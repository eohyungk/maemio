// src/contention/mod.rs
mod manager;
pub use manager::ContentionManager;

pub const DEFAULT_HILL_CLIMB_INTERVAL: u64 = 5000; // 5ms in microseconds
pub const DEFAULT_BACKOFF_STEP: u64 = 5; // 5 microseconds
pub const DEFAULT_MAX_BACKOFF_TIME: u64 = 100; // 100 microseconds