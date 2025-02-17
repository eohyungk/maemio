// src/index/mod.rs

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;
use crate::error::{MaemioError, Result};

/// Represents the type of index
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexType {
    BTree,
    Hash,
}

/// Generic key type that can be used in indexes
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]  // Added Hash trait
pub enum IndexKey {
    Int(i64),
    String(String),
    Bytes(Vec<u8>),
}


/// Node structure used in multi-version indexes
#[derive(Debug)]
pub struct IndexNode {
    // Write timestamp when this node was last modified
    pub wts: AtomicU64,
    // Read timestamp tracking
    pub rts: AtomicU64,
    // For B-tree: range of keys in this node
    pub min_key: Option<IndexKey>,
    pub max_key: Option<IndexKey>,
    // Record IDs stored in this node
    pub records: RwLock<Vec<u64>>,
}

impl IndexNode {
    pub fn new() -> Self {
        Self {
            wts: AtomicU64::new(0),
            rts: AtomicU64::new(0),
            min_key: None,
            max_key: None,
            records: RwLock::new(Vec::new()),
        }
    }

    pub fn update_rts(&self, ts: u64) {
        self.rts.fetch_max(ts, Ordering::AcqRel);
    }
}

/// Trait defining the interface for all index types
pub trait Index: Send + Sync {
    /// Inserts a key-value pair into the index
    fn insert(&self, key: IndexKey, record_id: u64, ts: u64) -> Result<()>;
    
    /// Removes a key-value pair from the index
    fn remove(&self, key: &IndexKey, ts: u64) -> Result<()>;
    
    /// Looks up a single key
    fn get(&self, key: &IndexKey, ts: u64) -> Result<Option<u64>>;
    
    /// Performs a range scan
    fn range_scan(&self, start: &IndexKey, end: &IndexKey, ts: u64) -> Result<Vec<u64>>;
    
    /// Returns nodes that need validation for a given key range
    fn get_validation_nodes(&self, start: &IndexKey, end: &IndexKey) -> Vec<Arc<IndexNode>>;
    
    /// Updates timestamps after successful validation
    fn update_timestamps(&self, nodes: &[Arc<IndexNode>], ts: u64);
}

// Common constants for index management
pub const MAX_KEY_SIZE: usize = 1024;
pub const MIN_DEGREE: usize = 4; // For B-tree nodes

// Status codes for index operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexStatus {
    Success,
    KeyNotFound,
    KeyExists,
    NodeFull,
    Error,
}

// Now include our submodules
mod btree;
mod hash;
mod manager;

// And re-export the public interface
pub use self::btree::BTreeIndex;
pub use self::hash::HashIndex;
pub use self::manager::IndexManager;