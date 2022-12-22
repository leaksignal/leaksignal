use futures::{AsyncRead, AsyncReadExt};
use leakpolicy::{ContentType, Policy};
use log::warn;
use protobuf::{rt::WireType, CodedInputStream};

use crate::{evaluator::MatcherState, perf::PerformanceMonitor, pipe::PipeReader, Match};

use super::{ParseResponse, Parser, ParserConfiguration};
use anyhow::{Context, Result};

async fn read<const N: usize>(reader: &mut (impl AsyncRead + Unpin)) -> Result<[u8; N]> {
    let mut output = [0u8; N];
    reader.read_exact(&mut output[..]).await?;
    Ok(output)
}

fn parse_message(
    input: &[u8],
    offset: usize,
    matcher: &MatcherState,
    performance: &PerformanceMonitor,
) -> Result<(Vec<Match>, ParseResponse)> {
    let mut reader = CodedInputStream::from_bytes(input);
    let mut out_response = ParseResponse::Continue;
    let mut out = vec![];
    while !reader.eof()? {
        let tag = reader.read_raw_varint32()?;
        let _field_number = tag >> 3;
        let wire_type = WireType::new(tag & 0b111).context("invalid wire type")?;
        match wire_type {
            WireType::LengthDelimited => {
                let len = reader.read_raw_varint32()?;
                let position = offset + reader.pos() as usize;
                let bytes = reader.read_raw_bytes(len)?;
                match String::from_utf8(bytes) {
                    Ok(string) => {
                        // we found a UTF-8 string, scan it
                        matcher.do_matching(position, 0, &string, &mut out, performance);
                    }
                    Err(e) => {
                        // not a UTF-8 string, try to parse as a message
                        let bytes = e.into_bytes();
                        if let Ok((results, response)) =
                            parse_message(&bytes[..], position, matcher, performance)
                        {
                            out.extend(results);
                            if response != ParseResponse::Continue {
                                out_response = response;
                            }
                        }
                    }
                }
            }
            _ => reader.skip_field(wire_type)?,
        }
    }
    Ok((out, out_response))
}

pub struct GrpcParser;

#[async_trait::async_trait(?Send)]
impl Parser for GrpcParser {
    async fn parse(
        &self,
        policy: &Policy,
        body: &mut PipeReader,
        configuration: ParserConfiguration<'_>,
        matches: &mut Vec<Match>,
        performance: &PerformanceMonitor,
    ) -> Result<ParseResponse> {
        let matcher = super::prepare_match_state(policy, configuration, ContentType::Grpc);

        let compressed_flag = read::<1>(body).await?[0] != 0;
        if compressed_flag {
            warn!("gRPC compressed not supported");
            return Ok(ParseResponse::Continue);
        }
        let length = u32::from_be_bytes(read::<4>(body).await?);

        // we are not expecting more than one message per response

        let mut raw_body = Vec::with_capacity(length as usize);
        body.read_exact(unsafe {
            std::slice::from_raw_parts_mut(raw_body.as_mut_ptr(), length as usize)
        })
        .await?;
        unsafe { raw_body.set_len(length as usize) };

        let (new_matches, response) = parse_message(&raw_body[..], 0, &matcher, performance)?;
        matches.extend(new_matches);

        Ok(response)
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use crate::config::StdTimestampProvider;

    use super::*;
    const RAW_BODY: &str =
        "0A2434386430313236662D393836632D343234372D383634342D323231653062303566663762";

    #[test]
    fn parse_body() {
        let raw_body = hex::decode(RAW_BODY).unwrap();
        let mut matcher = MatcherState::default();
        matcher.push_raw("test_raw", "6c-42");
        let (matches, _) = parse_message(
            &raw_body[..],
            0,
            &matcher,
            &PerformanceMonitor::new(std::sync::Arc::new(StdTimestampProvider::default())),
        )
        .unwrap();
        println!("{matches:?}");
        assert_eq!(matches.len(), 1);
    }
}
