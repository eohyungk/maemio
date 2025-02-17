// src/transaction/mod.rs
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use parking_lot::RwLock;
use crate::clock::Clock;
use crate::data::{Version, RecordHead};
use crate::error::{MaemioError, Result};
use crate::contention::ContentionManager;
mod manager;
pub use manager::TransactionManager;

#[derive(Clone)]
struct ValidationData {
    timestamp: u64,
    write_checks: HashMap<u64, Version>,
    read_checks: HashMap<u64, Arc<Version>>,
}

pub struct Transaction {
    timestamp: u64,
    read_set: HashMap<u64, Arc<Version>>,
    write_set: HashMap<u64, Version>,
    local_writes: HashMap<u64, Arc<Version>>,
    clock: Arc<Clock>,
    records: Arc<RwLock<HashMap<u64, Arc<RecordHead>>>>,
    contention_manager: Arc<ContentionManager>,
    thread_id: usize,
}

impl Transaction {
    pub fn new(
        clock: Arc<Clock>, 
        records: Arc<RwLock<HashMap<u64, Arc<RecordHead>>>>,
        contention_manager: Arc<ContentionManager>,
        thread_id: usize,
    ) -> Self {
        Self {
            timestamp: clock.generate_write_timestamp(),
            read_set: HashMap::new(),
            write_set: HashMap::new(),
            local_writes: HashMap::new(),
            clock,
            records,
            contention_manager,
            thread_id,
        }
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn read(&mut self, record_id: u64) -> Result<Arc<Version>> {
        if let Some(local_version) = self.local_writes.get(&record_id) {
            return Ok(local_version.clone());
        }
        let record = self.get_record(record_id)?;
        let visible_version = record.find_visible_version(self.timestamp)
            .ok_or(MaemioError::NoVisibleVersion)?;
        self.read_set.insert(record_id, visible_version.clone());
        Ok(visible_version)
    }

    pub fn write(&mut self, record_id: u64, data: Vec<u8>) -> Result<()> {
        let record = self.get_record(record_id)?;
        let new_version = Version::new(self.timestamp, data);
        self.write_set.insert(record_id, new_version.clone());
        self.local_writes.insert(record_id, Arc::new(new_version));
        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        self.validate()?;
        // To avoid overlapping borrows on self, take out the write_set.
        let write_set = std::mem::take(&mut self.write_set);
        for (record_id, version) in write_set {
            let record = self.get_record(record_id)?;
            version.commit();
            record.install_version(version.clone())?;
        }
        self.clock.reset_boost();
        Ok(())
    }

    fn prepare_validation_info(&self) -> Result<ValidationData> {
        Ok(ValidationData {
            timestamp: self.timestamp,
            write_checks: self.write_set.clone(),
            read_checks: self.read_set.clone(),
        })
    }
    fn prepare_validation_data(&self) -> Result<ValidationData> {
        Ok(ValidationData {
            timestamp: self.timestamp,
            write_checks: self.write_set.clone(),
            read_checks: self.read_set.clone(),
        })
    }

    // validate now takes &self because it only reads data.
    fn validate(&self) -> Result<()> {
        let validation_data = self.prepare_validation_data()?;
        {
            let records = self.records.read();
            for (record_id, _write_version) in &validation_data.write_checks {
                let record = records.get(record_id)
                    .ok_or(MaemioError::RecordNotFound(*record_id))?;
                if let Some(current_visible) = record.find_visible_version(validation_data.timestamp) {
                    if current_visible.wts > validation_data.timestamp {
                        return Err(MaemioError::Conflict);
                    }
                }
            }
            for (record_id, read_version) in &validation_data.read_checks {
                let record = records.get(record_id)
                    .ok_or(MaemioError::RecordNotFound(*record_id))?;
                let current_visible = record.find_visible_version(validation_data.timestamp)
                    .ok_or(MaemioError::ValidationFailed)?;
                if current_visible.wts != read_version.wts {
                    return Err(MaemioError::Conflict);
                }
            }
        }
        Ok(())
    }    
    
    fn get_record(&self, record_id: u64) -> Result<Arc<RecordHead>> {
        self.records.read()
            .get(&record_id)
            .cloned()
            .ok_or(MaemioError::RecordNotFound(record_id))
    }

    pub fn create_record(&mut self, record_id: u64) -> Result<()> {
        let record = Arc::new(RecordHead::new(self.timestamp));
        self.records.write().insert(record_id, record);
        Ok(())
    }

    pub fn prepare_gc_tracking(&self) -> Vec<(Arc<RecordHead>, u64)> {
        let records = self.records.read();
        self.write_set
            .iter()
            .filter_map(|(&id, version)| {
                records.get(&id)
                    .map(|record| (record.clone(), version.wts))
            })
            .collect()
    }
    pub fn start_contention_management(&self) -> std::thread::JoinHandle<()> {
        self.contention_manager.start_hill_climbing()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::ClockManager;

    fn setup_test_env() -> (Arc<Clock>, Arc<RwLock<HashMap<u64, Arc<RecordHead>>>>, Arc<ContentionManager>) {
        let clock_manager = Arc::new(ClockManager::new(1, 100).unwrap());
        let clock = clock_manager.get_clock(0);
        let records = Arc::new(RwLock::new(HashMap::new()));
        let contention_manager = Arc::new(ContentionManager::new(1, 1000, 5));
        (clock, records, contention_manager)
    }

    #[test]
    fn test_basic_transaction() {
        let (clock, records, contention_manager) = setup_test_env();
        let record = Arc::new(RecordHead::new(0));
        records.write().insert(1, record.clone());
        let mut tx1 = Transaction::new(clock.clone(), records.clone(), contention_manager.clone(), 0);
        tx1.write(1, vec![1, 2, 3]).unwrap();
        tx1.commit().unwrap();
        let mut tx2 = Transaction::new(clock.clone(), records.clone(), contention_manager.clone(), 0);
        let version = tx2.read(1).unwrap();
        assert_eq!(version.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_concurrent_transactions() {
        let (clock, records, contention_manager) = setup_test_env();
        let record = Arc::new(RecordHead::new(0));
        records.write().insert(1, record.clone());
        let mut tx1 = Transaction::new(clock.clone(), records.clone(), contention_manager.clone(), 0);
        tx1.write(1, vec![1]).unwrap();
        tx1.commit().unwrap();
        let mut tx2 = Transaction::new(clock.clone(), records.clone(), contention_manager.clone(), 1);
        tx2.write(1, vec![2]).unwrap();
        let result = tx2.commit();
        assert!(result.is_ok());
        let mut verify_tx = Transaction::new(clock, records, contention_manager, 2);
        let version = verify_tx.read(1).unwrap();
        assert_eq!(version.data, vec![2]);
    }
}
