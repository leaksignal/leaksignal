use std::cmp::Ordering;

use anyhow::Result;
use futures::AsyncReadExt;

use crate::{
    perf::PerformanceMonitor,
    pipe::PipeReader,
    policy::{ContentType, Policy, PolicyAction},
    Match,
};

use super::{ParseResponse, Parser, ParserConfiguration};

#[derive(Clone, Copy, PartialEq, Eq)]
struct PolicyMatch<'a> {
    category_name: &'a str,
    action: &'a PolicyAction,
    start: usize,
    length: usize,
}

impl<'a> PartialOrd for PolicyMatch<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.start.cmp(&other.start))
    }
}

impl<'a> Ord for PolicyMatch<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

const CHUNK_SIZE: usize = 1024 * 64;
const CHUNK_OVERLAP: usize = 512;

pub struct PlaintextParser;

#[async_trait::async_trait(?Send)]
impl Parser for PlaintextParser {
    async fn parse(
        &self,
        policy: &Policy,
        body: &mut PipeReader,
        configuration: ParserConfiguration<'_>,
        matches: &mut Vec<Match>,
        performance: &PerformanceMonitor,
    ) -> Result<ParseResponse> {
        let mut chunk = Vec::<u8>::with_capacity(CHUNK_SIZE);
        let mut overlap_index = 0usize;
        let mut chunk_len = 0usize;
        //TODO: safety, use ManuallyUninit?
        unsafe { chunk.set_len(CHUNK_SIZE) };

        let match_state = super::prepare_match_state(policy, configuration, ContentType::Html);

        loop {
            let minimum_end_index = body.total_read();
            let index = minimum_end_index - overlap_index;
            chunk.copy_within(chunk_len - overlap_index..chunk_len, 0);
            chunk_len = body.read(&mut chunk[overlap_index..]).await?;
            if chunk_len == 0 {
                break;
            }
            chunk_len += overlap_index;
            let data = &chunk[..chunk_len];

            overlap_index = CHUNK_OVERLAP.min(chunk_len);
            //TODO: make a better way to trim truncated utf8 characters to avoid allocation
            match match_state.do_matching(
                index,
                minimum_end_index,
                &*String::from_utf8_lossy(&data[..]),
                matches,
                performance,
            ) {
                ParseResponse::Continue => (),
                ParseResponse::Block => return Ok(ParseResponse::Block),
            }
        }
        Ok(ParseResponse::Continue)
    }
}
