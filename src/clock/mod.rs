mod manager;
pub use manager::ClockManager;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use crate::error::{MaemioError, Result};

pub struct Clock {
    local_clock: AtomicU64,
    clock_boost: AtomicU64,
    thread_id: u8,
    last_timestamp: AtomicU64,
    read_timestamp: AtomicU64,
}

impl Clock {
    pub fn new(thread_id: u8) -> Result<Self> {
        if thread_id >= 255 {
            return Err(MaemioError::System("Thread ID must be less than 255".into()));
        }

        Ok(Self {
            local_clock: AtomicU64::new(0),
            clock_boost: AtomicU64::new(0),
            thread_id,
            last_timestamp: AtomicU64::new(0),
            read_timestamp: AtomicU64::new(0),
        })
    }

    pub fn generate_write_timestamp(&self) -> u64 {
        let now = Instant::now().elapsed().as_micros() as u64;
        self.local_clock.fetch_add(now, Ordering::Relaxed);
        
        let current_clock = self.local_clock.load(Ordering::Relaxed);
        let boosted_clock = current_clock + self.clock_boost.load(Ordering::Relaxed);
        let last_ts = self.last_timestamp.load(Ordering::Relaxed);
        let new_clock = std::cmp::max(boosted_clock, last_ts + 1);
        
        let timestamp = (new_clock << 8) | (self.thread_id as u64);
        self.last_timestamp.store(timestamp, Ordering::Relaxed);
        
        timestamp
    }

    pub fn generate_read_timestamp(&self, min_write_ts: u64) -> u64 {
        let read_ts = min_write_ts.saturating_sub(1);
        self.read_timestamp.store(read_ts, Ordering::Relaxed);
        read_ts
    }

    pub fn synchronize_with(&self, other: &Clock) {
        let remote_clock = other.local_clock.load(Ordering::Relaxed);
        let local_clock = self.local_clock.load(Ordering::Relaxed);
        
        if remote_clock > local_clock {
            self.local_clock.store(remote_clock, Ordering::Relaxed);
        }
    }

    pub fn apply_boost(&self, boost_amount: u64) {
        self.clock_boost.store(boost_amount, Ordering::Relaxed);
    }

    pub fn reset_boost(&self) {
        self.clock_boost.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_generation() {
        let clock = Clock::new(1).unwrap();
        let ts1 = clock.generate_write_timestamp();
        let ts2 = clock.generate_write_timestamp();
        assert!(ts2 > ts1, "Timestamps should be monotonically increasing");
        assert_eq!(ts1 & 0xFF, 1, "Thread ID should be preserved in timestamp");
    }

    #[test]
    fn test_clock_synchronization() {
        let clock1 = Clock::new(1).unwrap();
        let clock2 = Clock::new(2).unwrap();
        
        clock2.local_clock.store(1000, Ordering::Relaxed);
        clock1.synchronize_with(&clock2);
        
        assert!(clock1.local_clock.load(Ordering::Relaxed) >= 1000);
    }
}