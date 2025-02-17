// src/index/hash.rs
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use super::{Index, IndexKey, IndexNode};
use crate::error::{MaemioError, Result};

pub struct HashIndex {
    buckets: Vec<RwLock<(Arc<IndexNode>, HashMap<IndexKey, u64>)>>,
    num_buckets: usize,
}

impl HashIndex {
    pub fn new(capacity: usize) -> Self {
        let num_buckets = capacity.next_power_of_two();
        let mut buckets = Vec::with_capacity(num_buckets);
        
        for _ in 0..num_buckets {
            buckets.push(RwLock::new((
                Arc::new(IndexNode::new()),
                HashMap::new()
            )));
        }
        
        Self {
            buckets,
            num_buckets,
        }
    }
    
    fn get_bucket_index(&self, key: &IndexKey) -> usize {
        match key {
            IndexKey::Int(i) => (*i as usize) & (self.num_buckets - 1),
            IndexKey::String(s) => {
                let hash: usize = s.as_bytes()
                    .iter()
                    .fold(0_usize, |acc, &x| acc.wrapping_add(x as usize));
                hash & (self.num_buckets - 1)
            },
            IndexKey::Bytes(b) => {
                let hash: usize = b.iter()
                    .fold(0_usize, |acc, &x| acc.wrapping_add(x as usize));
                hash & (self.num_buckets - 1)
            },
        }
    }
}

impl Index for HashIndex {
    fn insert(&self, key: IndexKey, record_id: u64, ts: u64) -> Result<()> {
        let bucket_idx = self.get_bucket_index(&key);
        let mut bucket = self.buckets[bucket_idx].write();
        
        bucket.0.wts.store(ts, std::sync::atomic::Ordering::Release);
        bucket.1.insert(key, record_id);
        
        Ok(())
    }
    
    fn remove(&self, key: &IndexKey, ts: u64) -> Result<()> {
        let bucket_idx = self.get_bucket_index(key);
        let mut bucket = self.buckets[bucket_idx].write();
        
        if bucket.1.remove(key).is_some() {
            bucket.0.wts.store(ts, std::sync::atomic::Ordering::Release);
            Ok(())
        } else {
            Err(MaemioError::RecordNotFound(0))
        }
    }
    
    fn get(&self, key: &IndexKey, ts: u64) -> Result<Option<u64>> {
        let bucket_idx = self.get_bucket_index(key);
        let bucket = self.buckets[bucket_idx].read();
        
        if bucket.0.wts.load(std::sync::atomic::Ordering::Acquire) <= ts {
            Ok(bucket.1.get(key).copied())
        } else {
            Ok(None)
        }
    }
    
    fn range_scan(&self, _start: &IndexKey, _end: &IndexKey, _ts: u64) -> Result<Vec<u64>> {
        Err(MaemioError::System("Range scan not supported on hash index".into()))
    }
    
    fn get_validation_nodes(&self, start: &IndexKey, end: &IndexKey) -> Vec<Arc<IndexNode>> {
        if start == end {
            vec![self.buckets[self.get_bucket_index(start)].read().0.clone()]
        } else {
            self.buckets.iter()
                .map(|bucket| bucket.read().0.clone())
                .collect()
        }
    }
    
    fn update_timestamps(&self, nodes: &[Arc<IndexNode>], ts: u64) {
        for node in nodes {
            node.update_rts(ts);
        }
    }
}