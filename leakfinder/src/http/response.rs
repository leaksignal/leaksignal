use std::{io::Write, pin::Pin, sync::Arc, task::Poll};

use flate2::write::GzDecoder;
use futures::{task::waker, Future, FutureExt};
use leakpolicy::{ContentType, PathPolicy};
use rand::{thread_rng, Rng};

use crate::{
    config::TimestampSource,
    match_data::ParsedMatches,
    parsers::{grpc::parse_grpc, html::parse_html, json::parse_json, ParseResponse},
    perf::PerformanceMonitor,
    pipe::{pipe, DummyWaker, PipeReader, PipeWriter},
    policy::PolicyRef,
};

use super::ContentEncoding;
use anyhow::{bail, Result};

enum Decoder {
    None,
    Gzip(GzDecoder<Vec<u8>>),
}

pub struct ResponseBodyContext {
    task: Option<Pin<Box<dyn Future<Output = Result<ResponseOutputData>>>>>,
    writer: PipeWriter,
    decoder: Decoder,
}

impl ResponseBodyContext {
    pub(super) fn spawn(
        policy: PolicyRef,
        timestamp_source: TimestampSource,
        path_policy: Arc<PathPolicy>,
        content_type: ContentType,
        content_encoding: ContentEncoding,
    ) -> Self {
        let max_persistence = (policy.max_body_collection_mb * 1024.0 * 1024.0) as usize;
        let (reader, writer) = pipe(max_persistence);

        Self {
            task: Some(Box::pin(response_body_task(
                policy,
                timestamp_source,
                path_policy,
                content_type,
                reader,
            ))),
            writer,
            decoder: match content_encoding {
                ContentEncoding::Gzip => Decoder::Gzip(GzDecoder::new(vec![])),
                ContentEncoding::None | ContentEncoding::Unknown => Decoder::None,
            },
        }
    }

    fn decode_data(&mut self, body: Vec<u8>) -> Result<Vec<u8>> {
        match &mut self.decoder {
            Decoder::None => Ok(body),
            Decoder::Gzip(decompressor) => {
                decompressor.write_all(&body[..])?;
                Ok(decompressor.get_mut().drain(..).collect::<Vec<_>>())
            }
        }
    }

    fn decode_finish(&mut self) -> Result<Vec<u8>> {
        match std::mem::replace(&mut self.decoder, Decoder::None) {
            Decoder::None => Ok(vec![]),
            Decoder::Gzip(decoder) => Ok(decoder.finish()?),
        }
    }

    fn poll(&mut self) -> Poll<Result<ResponseOutputData>> {
        let waker = waker(Arc::new(DummyWaker));
        let mut context = std::task::Context::from_waker(&waker);
        match self.task.as_mut().unwrap().poll_unpin(&mut context) {
            Poll::Ready(Err(e)) => {
                self.task.take();
                Poll::Ready(Err(e))
            }
            Poll::Ready(Ok(data)) => {
                self.task.take();
                Poll::Ready(Ok(data))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    pub fn receive_chunk(&mut self, body: Vec<u8>) -> Result<ParseResponse> {
        if body.is_empty() {
            return Ok(ParseResponse::Continue);
        }
        let body = self.decode_data(body)?;
        self.writer.append(body);
        match self.poll() {
            Poll::Ready(Err(e)) => Err(e),
            Poll::Ready(Ok(_)) => {
                unreachable!("unexpected early EOF");
            }
            Poll::Pending => Ok(ParseResponse::Continue),
        }
    }

    pub fn end_stream(mut self) -> Result<ResponseOutputData> {
        let body = self.decode_finish()?;
        self.writer.append(body);
        self.writer.close();

        match self.poll() {
            Poll::Ready(Err(e)) => Err(e),
            Poll::Ready(Ok(output)) => Ok(output),
            Poll::Pending => {
                unreachable!("parsing future did not finish on time");
            }
        }
    }
}

pub struct ResponseOutputData {
    pub response: ParseResponse,
    pub matches: ParsedMatches,
}

async fn response_body_task(
    policy: PolicyRef,
    timestamp_source: TimestampSource,
    path_policy: Arc<PathPolicy>,
    content_type: ContentType,
    mut reader: PipeReader,
) -> Result<ResponseOutputData> {
    let mut matches = vec![];
    let performance = PerformanceMonitor::new(timestamp_source.clone());

    let response = match content_type {
        ContentType::Html => {
            match parse_html(
                &*policy,
                &mut reader,
                &path_policy.configuration,
                &mut matches,
                &performance,
            )
            .await
            {
                Ok(x) => x,
                Err(e) => {
                    bail!("failed to read html: {:?}", e);
                }
            }
        }
        ContentType::Json => {
            match parse_json(
                &*policy,
                &mut reader,
                &path_policy.configuration,
                &mut matches,
                &performance,
            )
            .await
            {
                Ok(x) => x,
                Err(e) => {
                    bail!("failed to read json: {:?}", e);
                }
            }
        }
        ContentType::Grpc => {
            match parse_grpc(
                &*policy,
                &mut reader,
                &path_policy.configuration,
                &mut matches,
                &performance,
            )
            .await
            {
                Ok(x) => x,
                Err(e) => {
                    bail!("failed to read grpc: {:?}", e);
                }
            }
        }
        // do no parsing here
        ContentType::Jpeg => ParseResponse::Continue, // parse_jpeg(&*body, &configuration),
        ContentType::Unknown => ParseResponse::Continue,
    };

    let body_size = reader.total_read() as u64;
    let body = if policy.body_collection_rate <= 0.0 || policy.max_body_collection_mb <= 0.0 {
        None
    } else {
        let chance: f64 = thread_rng().gen();
        if chance < policy.body_collection_rate {
            reader.fetch_full_content()
        } else {
            None
        }
    };

    let response_body_end = timestamp_source.epoch_ns();

    let matches = ParsedMatches {
        time_parse_end: response_body_end,
        matches,
        body_size,
        body,
        category_performance_us: performance.into_inner(),
    };

    Ok(ResponseOutputData { response, matches })
}
