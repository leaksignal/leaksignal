use std::{cell::RefCell, collections::HashMap, time::Duration};

use crate::config::TimestampSource;

pub struct PerformanceMonitor {
    inner: RefCell<HashMap<String, u64>>,
    timestamp_source: TimestampSource,
}

impl PerformanceMonitor {
    pub fn new(timestamp_source: TimestampSource) -> Self {
        Self {
            inner: Default::default(),
            timestamp_source,
        }
    }

    pub fn measure<'a>(&'a self, name: &'a str) -> PerformanceHandle<'a> {
        PerformanceHandle {
            monitor: self,
            name,
            start: self.timestamp_source.elapsed(),
        }
    }

    pub fn into_inner(self) -> HashMap<String, u64> {
        self.inner.into_inner()
    }

    #[allow(unused)]
    /// convenience function for quick temporary benchmarking
    pub fn elapsed(&self) -> Duration {
        self.timestamp_source.elapsed()
    }
}

pub struct PerformanceHandle<'a> {
    monitor: &'a PerformanceMonitor,
    name: &'a str,
    start: Duration,
}

impl<'a> PerformanceHandle<'a> {
    /// Same effect as dropping and recreating a PerformanceHandle with a new name, but with one less clock fetch
    pub fn chain(&mut self, name: &'a str) {
        let now = self.monitor.timestamp_source.elapsed();
        self.submit(now);
        self.name = name;
        self.start = now;
    }

    fn submit(&self, now: Duration) {
        let total = (now - self.start).as_micros() as u64;
        let mut category_performance_us = self.monitor.inner.borrow_mut();
        if let Some(x) = category_performance_us.get_mut(self.name) {
            *x += total;
        } else {
            category_performance_us.insert(self.name.to_string(), total);
        }
    }
}

impl<'a> Drop for PerformanceHandle<'a> {
    fn drop(&mut self) {
        let now = self.monitor.timestamp_source.elapsed();
        self.submit(now);
    }
}
