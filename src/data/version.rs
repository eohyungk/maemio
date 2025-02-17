//src/data/version.rs
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;

pub struct Version {
    pub(crate) wts: u64,
    pub(crate) rts: AtomicU64,
    pub(crate) status: AtomicU8,
    pub(crate) data: Vec<u8>,
    pub(crate) next: Option<Box<Version>>,
}

impl Version {
    pub fn new(wts: u64, data: Vec<u8>) -> Self {
        Self {
            wts,
            rts: AtomicU64::new(0),
            status: AtomicU8::new(super::VERSION_STATUS_PENDING),
            data,
            next: None,
        }
    }

    pub fn is_visible_to(&self, ts: u64) -> bool {
        let status = self.status.load(Ordering::Acquire);
        
        // A version is visible if:
        // 1. Its write timestamp is less than or equal to the transaction's timestamp
        // 2. It is committed
        let is_visible = self.wts <= ts && status == super::VERSION_STATUS_COMMITTED;

        tracing::debug!(
            "Checking visibility: version_ts={}, tx_ts={}, status={}, result={}",
            self.wts,
            ts,
            status,
            is_visible
        );
        
        is_visible
    }

    pub fn commit(&self) {
        tracing::debug!("Committing version with timestamp {}", self.wts);
        self.status.store(super::VERSION_STATUS_COMMITTED, Ordering::Release);
    }

    pub fn wait_pending(&self) -> bool {
        let mut status = self.status.load(Ordering::Acquire);
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 1000;  // Prevent infinite waiting

        while status == super::VERSION_STATUS_PENDING && attempts < MAX_ATTEMPTS {
            std::thread::yield_now();
            status = self.status.load(Ordering::Acquire);
            attempts += 1;
        }

        status == super::VERSION_STATUS_COMMITTED
    }

    pub fn update_rts(&self, ts: u64) {
        let current = self.rts.load(Ordering::Relaxed);
        if ts > current {
            self.rts.store(ts, Ordering::Release);
        }
    }

    pub fn abort(&self) {
        self.status.store(super::VERSION_STATUS_ABORTED, Ordering::Release);
    }
}

impl Clone for Version {
    fn clone(&self) -> Self {
        Self {
            wts: self.wts,
            rts: AtomicU64::new(self.rts.load(Ordering::Relaxed)),
            status: AtomicU8::new(self.status.load(Ordering::Relaxed)),
            data: self.data.clone(),
            next: self.next.clone(),
        }
    }
}