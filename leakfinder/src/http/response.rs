use std::{io::Write, pin::Pin, sync::Arc, task::Poll};

use flate2::write::GzDecoder;
use futures::{task::waker, Future, FutureExt};
use leakpolicy::{ContentType, EndpointContext, PathPolicy};
use rand::{thread_rng, Rng};

use crate::{
    config::TimestampSource,
    match_data::ParsedMatches,
    parsers::{
        grpc::GrpcParser, json::JsonParser, plaintext::PlaintextParser, ParseResponse, Parser,
        ParserConfiguration,
    },
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

pub struct BodyContext {
    task: Option<Pin<Box<dyn Future<Output = Result<OutputData>>>>>,
    writer: PipeWriter,
    decoder: Decoder,
    early_return: Option<OutputData>,
}

impl BodyContext {
    pub(super) fn spawn(
        policy: PolicyRef,
        timestamp_source: TimestampSource,
        path_policy: Arc<PathPolicy>,
        active_context: EndpointContext,
        content_type: ContentType,
        content_encoding: ContentEncoding,
    ) -> Self {
        let max_persistence = (policy.max_body_collection_mb * 1024.0 * 1024.0) as usize;
        let (reader, writer) = pipe(max_persistence);

        Self {
            task: Some(Box::pin(body_task(
                policy,
                timestamp_source,
                path_policy,
                active_context,
                content_type,
                reader,
            ))),
            writer,
            decoder: match content_encoding {
                ContentEncoding::Gzip => Decoder::Gzip(GzDecoder::new(vec![])),
                ContentEncoding::None | ContentEncoding::Unknown => Decoder::None,
            },
            early_return: None,
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

    fn poll(&mut self) -> Poll<Result<OutputData>> {
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
        if let Some(early_return) = &self.early_return {
            return Ok(early_return.response);
        }
        let body = self.decode_data(body)?;
        self.writer.append(body);
        match self.poll() {
            Poll::Ready(Err(e)) => Err(e),
            Poll::Ready(Ok(value)) => {
                self.early_return = Some(value);
                Ok(self.early_return.as_ref().unwrap().response)
            }
            Poll::Pending => Ok(ParseResponse::Continue),
        }
    }

    pub fn end_stream(mut self) -> Result<OutputData> {
        if let Some(early_return) = self.early_return.take() {
            return Ok(early_return);
        }
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

pub struct OutputData {
    pub response: ParseResponse,
    pub matches: ParsedMatches,
}

async fn body_task(
    policy: PolicyRef,
    timestamp_source: TimestampSource,
    path_policy: Arc<PathPolicy>,
    active_context: EndpointContext,
    content_type: ContentType,
    mut reader: PipeReader,
) -> Result<OutputData> {
    let response_body_start = timestamp_source.epoch_ns();

    let mut matches = vec![];
    let performance = PerformanceMonitor::new(timestamp_source.clone());
    let configuration = ParserConfiguration {
        categories: &path_policy.configuration,
        active_context,
    };

    let response = 'response: {
        let parser: Box<dyn Parser> = match content_type {
            ContentType::Html => Box::new(PlaintextParser),
            //TODO: url decoding/parsing
            ContentType::UrlEncoded => Box::new(PlaintextParser),
            ContentType::Json => Box::new(JsonParser),
            ContentType::Grpc => Box::new(GrpcParser),
            ContentType::Jpeg => break 'response ParseResponse::Continue,
            ContentType::Unknown => break 'response ParseResponse::Continue,
        };

        match parser
            .parse(
                &policy,
                &mut reader,
                configuration,
                &mut matches,
                &performance,
            )
            .await
        {
            Ok(x) => x,
            Err(e) => {
                bail!("failed to parse body: {:?}", e);
            }
        }
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
        time_parse_start: response_body_start,
        matches,
        body_size,
        body,
        category_performance_us: performance.into_inner(),
    };

    Ok(OutputData { response, matches })
}
