use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use rand::Rng;
use crate::error::Result;

/// Tracks commit counts and contention for each thread
struct ThreadStats {
    commit_count: AtomicU64,
    last_commit_count: AtomicU64,
}

impl ThreadStats {
    fn new() -> Self {
        Self {
            commit_count: AtomicU64::new(0),
            last_commit_count: AtomicU64::new(0),
        }
    }
}

/// Manages contention across all threads using hill climbing
pub struct ContentionManager {
    // Store thread stats in Arc to allow sharing across clones
    thread_stats: Arc<Vec<ThreadStats>>,
    
    // Global backoff coordination
    max_backoff_time: AtomicU64,
    
    // Hill climbing state
    last_throughput: AtomicU64,
    last_backoff: AtomicU64,
    positive_gradient: AtomicBool,
    
    // Configuration
    hill_climb_interval: Duration,
    backoff_step: u64,
}

impl ContentionManager {
    pub fn new(thread_count: usize, hill_climb_interval_micros: u64, backoff_step: u64) -> Self {
        // Initialize thread statistics
        let mut stats = Vec::with_capacity(thread_count);
        for _ in 0..thread_count {
            stats.push(ThreadStats::new());
        }

        Self {
            thread_stats: Arc::new(stats),
            max_backoff_time: AtomicU64::new(0),
            last_throughput: AtomicU64::new(0),
            last_backoff: AtomicU64::new(0),
            positive_gradient: AtomicBool::new(true),
            hill_climb_interval: Duration::from_micros(hill_climb_interval_micros),
            backoff_step,
        }
    }

    /// Records a successful commit for the given thread
    pub fn record_commit(&self, thread_id: usize) {
        if let Some(stats) = self.thread_stats.get(thread_id) {
            stats.commit_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Calculates the current throughput across all threads
    fn calculate_throughput(&self) -> u64 {
        let mut total_commits = 0;
        
        for stats in self.thread_stats.iter() {
            let current = stats.commit_count.load(Ordering::Relaxed);
            let previous = stats.last_commit_count.load(Ordering::Relaxed);
            total_commits += current - previous;
            stats.last_commit_count.store(current, Ordering::Relaxed);
        }

        total_commits
    }

    /// Updates the maximum backoff time using hill climbing
    fn update_max_backoff(&self) {
        let current_throughput = self.calculate_throughput();
        let current_backoff = self.max_backoff_time.load(Ordering::Relaxed);
        
        let last_throughput = self.last_throughput.load(Ordering::Relaxed);
        let last_backoff = self.last_backoff.load(Ordering::Relaxed);
        
        // Calculate gradient if we have historical data
        if last_throughput > 0 {
            let throughput_change = current_throughput as i64 - last_throughput as i64;
            let backoff_change = current_backoff as i64 - last_backoff as i64;
            
            let gradient = if backoff_change != 0 {
                throughput_change as f64 / backoff_change as f64
            } else {
                0.0
            };

            // Update direction based on gradient
            let positive_gradient = gradient >= 0.0;
            self.positive_gradient.store(positive_gradient, Ordering::Relaxed);
            
            // Adjust backoff time
            let new_backoff = if positive_gradient {
                current_backoff.saturating_add(self.backoff_step)
            } else {
                current_backoff.saturating_sub(self.backoff_step)
            };
            
            self.max_backoff_time.store(new_backoff, Ordering::Release);
        }
        
        // Store current values for next iteration
        self.last_throughput.store(current_throughput, Ordering::Relaxed);
        self.last_backoff.store(current_backoff, Ordering::Relaxed);
    }

    /// Gets the current maximum backoff time
    pub fn get_max_backoff(&self) -> Duration {
        Duration::from_micros(self.max_backoff_time.load(Ordering::Acquire))
    }

    /// Starts the background hill climbing thread
    pub fn start_hill_climbing(&self) -> std::thread::JoinHandle<()> {
        // Clone Arc for thread
        let thread_stats = Arc::clone(&self.thread_stats);
        let hill_climb_interval = self.hill_climb_interval;
        let manager = self.clone();
        
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(hill_climb_interval);
                manager.update_max_backoff();
            }
        })
    }

    /// Performs randomized backoff after an abort
    pub fn backoff(&self) {
        let max_backoff = self.get_max_backoff();
        if max_backoff.as_micros() > 0 {
            let random_duration = Duration::from_micros(
                rand::thread_rng().gen_range(0..=max_backoff.as_micros() as u64)
            );
            std::thread::sleep(random_duration);
        }
    }
}

impl Clone for ContentionManager {
    fn clone(&self) -> Self {
        Self {
            thread_stats: Arc::clone(&self.thread_stats),
            max_backoff_time: AtomicU64::new(self.max_backoff_time.load(Ordering::Relaxed)),
            last_throughput: AtomicU64::new(self.last_throughput.load(Ordering::Relaxed)),
            last_backoff: AtomicU64::new(self.last_backoff.load(Ordering::Relaxed)),
            positive_gradient: AtomicBool::new(self.positive_gradient.load(Ordering::Relaxed)),
            hill_climb_interval: self.hill_climb_interval,
            backoff_step: self.backoff_step,
        }
    }
}
// [Previous code remains the same until the test module]

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_tracking() {
        let manager = ContentionManager::new(2, 1000, 5);
        
        manager.record_commit(0);
        manager.record_commit(0);
        manager.record_commit(1);
        
        let throughput = manager.calculate_throughput();
        assert_eq!(throughput, 3);
    }

    #[test]
    fn test_hill_climbing() {
        let manager = ContentionManager::new(1, 1000, 5);
        
        // Simulate increasing throughput
        for _ in 0..100 {
            manager.record_commit(0);
        }
        manager.update_max_backoff();
        
        // Simulate decreased throughput
        std::thread::sleep(Duration::from_millis(1));
        for _ in 0..50 {
            manager.record_commit(0);
        }
        manager.update_max_backoff();
        
        // Should have adjusted backoff time
        assert!(manager.get_max_backoff().as_micros() > 0);
    }

    #[test]
    fn test_backoff_randomization() {
        let manager = ContentionManager::new(1, 1000, 5);
        let max_backoff_micros = 100;
        manager.max_backoff_time.store(max_backoff_micros, Ordering::Relaxed);
        
        // Run multiple trials to account for system timing variability
        let trials = 10;
        let mut all_within_bound = true;
        let mut max_observed = 0;

        for _ in 0..trials {
            let start = Instant::now();
            manager.backoff();
            let elapsed_micros = start.elapsed().as_micros() as u64;
            
            // Update maximum observed delay
            max_observed = max_observed.max(elapsed_micros);

            // Allow for some system overhead, but ensure it's not wildly off
            // A factor of 2 accounts for reasonable system scheduling overhead
            if elapsed_micros > max_backoff_micros * 2 {
                all_within_bound = false;
                break;
            }
        }

        // If any trial was outside our expected bounds, provide detailed diagnostics
        if !all_within_bound {
            panic!(
                "Backoff time exceeded bounds: max_observed={} µs, max_allowed={} µs",
                max_observed,
                max_backoff_micros * 2
            );
        }
        
        // Also verify that backoff is actually happening
        assert!(max_observed > 0, "Expected non-zero backoff time");
    }
}