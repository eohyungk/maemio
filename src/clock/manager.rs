use super::Clock;
use crate::error::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct ClockManager {
    clocks: Vec<Arc<Clock>>,
    min_write_ts: AtomicU64,
    min_read_ts: AtomicU64,
    sync_interval: Duration,
}

impl ClockManager {
    pub fn new(thread_count: usize, sync_interval_micros: u64) -> Result<Self> {
        let mut clocks = Vec::with_capacity(thread_count);
        for id in 0..thread_count {
            clocks.push(Arc::new(Clock::new(id as u8)?));
        }

        Ok(Self {
            clocks,
            min_write_ts: AtomicU64::new(0),
            min_read_ts: AtomicU64::new(0),
            sync_interval: Duration::from_micros(sync_interval_micros),
        })
    }

    pub fn get_clock(&self, thread_id: usize) -> Arc<Clock> {
        self.clocks[thread_id].clone()
    }

    pub fn start_synchronization(&self) -> thread::JoinHandle<()> {
        let clocks = self.clocks.clone();
        let sync_interval = self.sync_interval;

        thread::spawn(move || {
            loop {
                thread::sleep(sync_interval);
                
                for i in 0..clocks.len() {
                    let next_idx = (i + 1) % clocks.len();
                    clocks[i].synchronize_with(&clocks[next_idx]);
                }
            }
        })
    }

    pub fn update_min_timestamps(&self) {
        let min_wts = self.clocks.iter()
            .map(|c| c.last_timestamp.load(Ordering::Relaxed))
            .min()
            .unwrap_or(0);
            
        let min_rts = self.clocks.iter()
            .map(|c| c.read_timestamp.load(Ordering::Relaxed))
            .min()
            .unwrap_or(0);
            
        self.min_write_ts.store(min_wts, Ordering::Release);
        self.min_read_ts.store(min_rts, Ordering::Release);
    }

    pub fn get_min_write_ts(&self) -> u64 {
        self.min_write_ts.load(Ordering::Acquire)
    }

    pub fn get_min_read_ts(&self) -> u64 {
        self.min_read_ts.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_clock_manager() {
        let manager = ClockManager::new(4, 100).unwrap();
        let clock0 = manager.get_clock(0);
        let clock1 = manager.get_clock(1);

        let ts1 = clock0.generate_write_timestamp();
        let ts2 = clock1.generate_write_timestamp();

        manager.update_min_timestamps();
        assert!(manager.get_min_write_ts() <= ts1.min(ts2));
    }
}