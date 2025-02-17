// src/transaction/manager.rs
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use super::Transaction;
use crate::clock::ClockManager;
use crate::error::{MaemioError, Result};
use crate::data::RecordHead;
use crate::gc::GarbageCollector;
use crate::contention::ContentionManager;

pub struct TransactionManager {
    clock_manager: Arc<ClockManager>,
    records: Arc<RwLock<HashMap<u64, Arc<RecordHead>>>>,
    contention_manager: Arc<ContentionManager>,
}

impl TransactionManager {
    pub fn new(
        clock_manager: Arc<ClockManager>,
        thread_count: usize,
    ) -> Result<Self> {
        let contention_manager = Arc::new(ContentionManager::new(
            thread_count,
            crate::contention::DEFAULT_HILL_CLIMB_INTERVAL,
            crate::contention::DEFAULT_BACKOFF_STEP,
        ));

        Ok(Self {
            clock_manager,
            records: Arc::new(RwLock::new(HashMap::new())),
            contention_manager,
        })
    }

    pub fn execute_with_gc<F, T>(&self, thread_id: usize, gc: &GarbageCollector, mut operation: F) -> Result<T>
    where
        F: FnMut(&mut Transaction) -> Result<T>  // Note: parameter is now marked as mut
    {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 10;

        loop {
            attempts += 1;
            if attempts > MAX_ATTEMPTS {
                return Err(MaemioError::System("Max retry attempts exceeded".into()));
            }

            let mut tx = self.begin_transaction(thread_id);
            
            match operation(&mut tx) {
                Ok(value) => {
                    let gc_info = tx.prepare_gc_tracking();
                    
                    match tx.commit() {
                        Ok(()) => {
                            self.contention_manager.record_commit(thread_id);
                            for (record, wts) in gc_info {
                                gc.track_version(record, wts);
                            }
                            return Ok(value);
                        }
                        Err(MaemioError::Conflict) => {
                            self.contention_manager.backoff();
                            continue;
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(MaemioError::Conflict) => {
                    self.contention_manager.backoff();
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn begin_transaction(&self, thread_id: usize) -> Transaction {
        let clock = self.clock_manager.get_clock(thread_id);
        Transaction::new(
            clock,
            self.records.clone(),
            self.contention_manager.clone(),
            thread_id,
        )
    }

    pub fn create_record(&self, record_id: u64) -> Result<()> {
        let mut records = self.records.write();
        
        if records.contains_key(&record_id) {
            return Err(MaemioError::System(
                format!("Record {} already exists", record_id)
            ));
        }

        // Get a new timestamp for this record creation
        let creation_ts = self.clock_manager.get_min_write_ts();
        
        // Create the record with this timestamp
        records.insert(record_id, Arc::new(RecordHead::new(creation_ts)));
        Ok(())
    }

    pub fn get_record(&self, record_id: u64) -> Result<Arc<RecordHead>> {
        self.records.read()
            .get(&record_id)
            .cloned()
            .ok_or(MaemioError::RecordNotFound(record_id))
    }
    pub fn start_contention_management(&self) -> std::thread::JoinHandle<()> {
        // Delegate to the contention manager
        self.contention_manager.start_hill_climbing()
    }

}