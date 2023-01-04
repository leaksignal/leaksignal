use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use crate::TimestampProvider;

type ItemMap = HashMap<String, BlockItem>;

#[derive(Serialize, Deserialize, Default)]
pub struct BlockState {
    pub ips: ItemMap,
    pub tokens: ItemMap,
    pub services: ItemMap,
}

impl BlockState {
    pub fn merge(&mut self, other: Self) {
        let f = |dest: &mut ItemMap, src: ItemMap| {
            for (key, item) in src {
                if matches!(item.reason, BlockReason::Unblock) {
                    dest.remove(&key);
                } else {
                    dest.insert(key, item);
                }
            }
        };
        f(&mut self.ips, other.ips);
        f(&mut self.tokens, other.tokens);
        f(&mut self.services, other.services);
    }

    fn is_blocked(
        map: &ItemMap,
        time: &dyn TimestampProvider,
        key: &str,
    ) -> Option<BlockReason> {
        let entry = map.get(key)?;
        (entry.expire_at >= time.elapsed().as_millis() as u64).then_some(entry.reason)
    }

    pub fn is_ip_blocked(&self, time: &dyn TimestampProvider, ip: &str) -> Option<BlockReason> {
        Self::is_blocked(&self.ips, time, ip)
    }

    pub fn is_token_blocked(
        &self,
        time: &dyn TimestampProvider,
        token: &str,
    ) -> Option<BlockReason> {
        Self::is_blocked(&self.tokens, time, token)
    }

    pub fn is_service_blocked(
        &self,
        time: &dyn TimestampProvider,
        service: &str,
    ) -> Option<BlockReason> {
        Self::is_blocked(&self.services, time, service)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct BlockItem {
    pub expire_at: u64,
    pub reason: BlockReason,
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
pub enum BlockReason {
    #[default]
    Unspecified,
    // only present in delta BlockStates
    Unblock,
    Ratelimit,
    Violation,
}

impl fmt::Display for BlockReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockReason::Unspecified => write!(f, "unspecified"),
            BlockReason::Unblock => write!(f, "unblocked"),
            BlockReason::Ratelimit => write!(f, "ratelimited"),
            BlockReason::Violation => write!(f, "policy violation"),
        }
    }
}
