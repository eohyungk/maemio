// src/gc/collector.rs
use std::sync::Arc;
use std::collections::VecDeque;
use std::time::Duration;
use parking_lot::Mutex;
use crate::error::Result;
use crate::clock::ClockManager;
use crate::data::{Version, RecordHead};

pub struct GarbageCollector {
    queue: Mutex<VecDeque<(Arc<RecordHead>, u64)>>,
    clock_manager: Arc<ClockManager>,
    gc_interval: Duration,
}

impl GarbageCollector {
    pub fn new(clock_manager: Arc<ClockManager>, gc_interval_micros: u64) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            clock_manager,
            gc_interval: Duration::from_micros(gc_interval_micros),
        }
    }

    pub fn track_version(&self, record: Arc<RecordHead>, wts: u64) {
        let mut queue = self.queue.lock();
        queue.push_back((record, wts));
    }

    pub fn collect_garbage(&self) -> Result<()> {
        let min_rts = self.clock_manager.get_min_read_ts();
        let mut queue = self.queue.lock();

        let mut remaining = VecDeque::new();
        while let Some((record, wts)) = queue.pop_front() {
            if wts >= min_rts {
                remaining.push_back((record.clone(), wts));
                continue;
            }

            if !record.try_gc_lock() {
                remaining.push_back((record.clone(), wts));
                continue;
            }

            self.collect_record_versions(&record, min_rts);
        }

        *queue = remaining;
        Ok(())
    }

    fn collect_record_versions(&self, record: &RecordHead, min_rts: u64) {
        record.update_min_wts(min_rts);

        // Build new chain from old versions
        let mut versions = Vec::new();
        let mut current = record.get_current_version();

        // First collect all versions we want to keep
        while let Some(version) = current {
            if version.wts >= min_rts {
                versions.push(Version::new(
                    version.wts,
                    version.data.clone()
                ));
            }
            current = version.next;
        }

        // Then rebuild the chain in reverse order
        let mut new_chain = None;
        for version in versions.into_iter().rev() {
            let mut boxed_version = Box::new(version);
            boxed_version.next = new_chain;
            new_chain = Some(boxed_version);
        }

        // Install the new chain if we have one
        if let Some(chain) = new_chain {
            let _ = record.install_version(*chain);
        }
    }

    pub fn start_collection(&self) -> std::thread::JoinHandle<()> {
        let gc = self.clone();
        std::thread::spawn(move || {
            loop {
                let _ = gc.collect_garbage();
                std::thread::sleep(gc.gc_interval);
            }
        })
    }
}

impl Clone for GarbageCollector {
    fn clone(&self) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            clock_manager: self.clock_manager.clone(),
            gc_interval: self.gc_interval,
        }
    }
}