use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use crate::TimestampProvider;

#[derive(Serialize, Deserialize, Default)]
pub struct BlockState {
    pub ips: HashMap<String, BlockItem>,
    pub tokens: HashMap<String, BlockItem>,
    pub services: HashMap<String, BlockItem>,
}

impl BlockState {
    pub fn merge(&mut self, other: Self) {
        for (ip, item) in other.ips {
            if matches!(item.reason, BlockReason::Unblock) {
                self.ips.remove(&ip);
            } else {
                self.ips.insert(ip, item);
            }
        }
        for (token, item) in other.tokens {
            if matches!(item.reason, BlockReason::Unblock) {
                self.tokens.remove(&token);
            } else {
                self.tokens.insert(token, item);
            }
        }
        for (service, item) in other.services {
            if matches!(item.reason, BlockReason::Unblock) {
                self.services.remove(&service);
            } else {
                self.services.insert(service, item);
            }
        }
    }

    pub fn is_ip_blocked(&self, time: &dyn TimestampProvider, ip: &str) -> Option<BlockReason> {
        let entry = self.ips.get(ip)?;
        if entry.expire_at < time.elapsed().as_millis() as u64 {
            None
        } else {
            Some(entry.reason)
        }
    }

    pub fn is_token_blocked(
        &self,
        time: &dyn TimestampProvider,
        token: &str,
    ) -> Option<BlockReason> {
        let entry = self.tokens.get(token)?;
        if entry.expire_at < time.elapsed().as_millis() as u64 {
            None
        } else {
            Some(entry.reason)
        }
    }

    pub fn is_service_blocked(
        &self,
        time: &dyn TimestampProvider,
        service: &str,
    ) -> Option<BlockReason> {
        let entry = self.services.get(service)?;
        if entry.expire_at < time.elapsed().as_millis() as u64 {
            None
        } else {
            Some(entry.reason)
        }
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
