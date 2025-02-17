#![allow(warnings)]

//! Maemio is a high-performance in-memory database system based on the Cicada design.
//! It provides serializable transactions with optimistic multi-version concurrency control.

mod error;
mod clock;
mod data;
mod transaction;
mod gc;
mod contention;
mod index;

pub use error::{MaemioError, Result};
pub use transaction::{Transaction, TransactionManager};
pub use gc::GarbageCollector;
pub use contention::ContentionManager;
pub use index::{Index, IndexType, IndexKey, IndexManager};

use std::sync::Arc;

/// Configuration options for the database instance
pub struct MaemioConfig {
    /// Number of worker threads
    pub thread_count: usize,
    /// Garbage collection interval in microseconds
    pub gc_interval: u64,
    /// Clock synchronization interval in microseconds
    pub clock_sync_interval: u64,
    /// Initial index capacity (for hash indexes)
    pub initial_index_capacity: usize,
}

impl Default for MaemioConfig {
    fn default() -> Self {
        Self {
            thread_count: num_cpus::get(),
            gc_interval: 10,  // 10 microseconds
            clock_sync_interval: 100,  // 100 microseconds
            initial_index_capacity: 1024,
        }
    }
}

/// The main database instance that coordinates all components
pub struct Maemio {
    // Core components
    transaction_manager: Arc<TransactionManager>,
    gc: Option<Arc<GarbageCollector>>,
    contention_manager: Arc<ContentionManager>,
    
    // Index management component
    index_manager: Arc<IndexManager>,
    
    // Configuration
    config: MaemioConfig,
}

impl Maemio {
    /// Creates a new database instance with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(MaemioConfig::default())
    }

    /// Creates a new database instance with custom configuration
    pub fn with_config(config: MaemioConfig) -> Result<Self> {
        // Initialize the clock manager first since other components depend on it
        let clock_manager = Arc::new(clock::ClockManager::new(
            config.thread_count, 
            config.clock_sync_interval
        )?);

        // Initialize the contention manager
        let contention_manager = Arc::new(ContentionManager::new(
            config.thread_count,
            contention::DEFAULT_HILL_CLIMB_INTERVAL,
            contention::DEFAULT_BACKOFF_STEP,
        ));

        // Create the transaction manager
        let transaction_manager = Arc::new(TransactionManager::new(
            clock_manager.clone(),
            config.thread_count,
        )?);

        // Create the garbage collector
        let gc = Some(Arc::new(GarbageCollector::new(
            clock_manager.clone(),
            config.gc_interval
        )));

        // Create the index manager
        let index_manager = Arc::new(IndexManager::new());

        Ok(Self {
            transaction_manager,
            gc,
            contention_manager,
            index_manager,
            config,
        })
    }

    /// Starts all background maintenance tasks
    pub fn start_maintenance(&self) -> Result<()> {
        // Start garbage collection if enabled
        if let Some(gc) = &self.gc {
            gc.start_collection();
        }
    
        // Start contention management
        self.transaction_manager.start_contention_management();
    
        Ok(())
    }

    /// Creates a new index for a table
    pub fn create_index(&self, table_id: u64, name: &str, index_type: IndexType) -> Result<()> {
        self.index_manager.create_index(table_id, name, index_type)
    }

    /// Drops an existing index
    pub fn drop_index(&self, table_id: u64, name: &str) -> Result<()> {
        self.index_manager.drop_index(table_id, name)
    }

    /// Begins a new transaction for the given thread
    pub fn begin_transaction(&self, thread_id: usize) -> Transaction {
        self.transaction_manager.begin_transaction(thread_id)
    }

    /// Execute a transaction with automatic retry and garbage collection
    pub fn execute<F, T>(&self, thread_id: usize, mut operation: F) -> Result<T>
    where
        F: FnMut(&mut Transaction) -> Result<T>
    {
        if let Some(ref gc) = self.gc {
            self.transaction_manager.execute_with_gc(thread_id, gc, operation)
        } else {
            let mut tx = self.begin_transaction(thread_id);
            operation(&mut tx)
        }
    }

    /// Creates a new record in the database
    pub fn create_record(&self, record_id: u64) -> Result<()> {
        self.transaction_manager.create_record(record_id)
    }

    /// Gets a reference to the index manager
    pub fn index_manager(&self) -> Arc<IndexManager> {
        self.index_manager.clone()
    }

    /// Stops all background tasks gracefully
    pub fn shutdown(&self) -> Result<()> {
        // Add shutdown logic for background tasks
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Maemio::new().unwrap();
        db.start_maintenance().unwrap();
        
        // Create a record first
        db.create_record(1).unwrap();
        
        // Create an index
        db.create_index(1, "test_idx", IndexType::BTree).unwrap();
        
        // Execute a transaction that writes data and uses the index
        db.execute(0, |tx| {
            // Write some data to the record
            tx.write(1, vec![1, 2, 3])?;
            
            // Use the index
            let index = db.index_manager().get_index(1, "test_idx")?;
            index.insert(IndexKey::Int(1), 1, tx.get_timestamp())?;
            
            Ok(())
        }).unwrap();
        
        // Verify the data was written correctly
        db.execute(0, |tx| {
            let version = tx.read(1)?;
            assert_eq!(version.data, vec![1, 2, 3]);
            
            let index = db.index_manager().get_index(1, "test_idx")?;
            let record_id = index.get(&IndexKey::Int(1), tx.get_timestamp())?;
            assert_eq!(record_id, Some(1));
            
            Ok(())
        }).unwrap();
        
        db.shutdown().unwrap();
    }

    #[test]
    fn test_custom_configuration() {
        let config = MaemioConfig {
            thread_count: 4,
            gc_interval: 20,
            clock_sync_interval: 200,
            initial_index_capacity: 2048,
        };
        
        let db = Maemio::with_config(config).unwrap();
        assert!(db.start_maintenance().is_ok());
        db.shutdown().unwrap();
    }

    #[test]
    fn test_concurrent_transactions() {
        let db = Maemio::new().unwrap();
        db.start_maintenance().unwrap();

        // Create a record
        db.create_record(1).unwrap();

        // First transaction writes initial value
        db.execute(0, |tx| {
            tx.write(1, vec![1])?;
            Ok(())
        }).unwrap();

        // Second transaction attempts to modify
        db.execute(1, |tx| {
            tx.write(1, vec![2])?;
            Ok(())
        }).unwrap();

        // Verify final state
        db.execute(2, |tx| {
            let version = tx.read(1)?;
            assert_eq!(version.data, vec![2]);
            Ok(())
        }).unwrap();

        db.shutdown().unwrap();
    }
}