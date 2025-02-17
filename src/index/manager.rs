// src/index/manager.rs
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use super::{
    Index, IndexType, IndexKey, IndexNode,
    BTreeIndex, HashIndex,
};
use crate::error::{MaemioError, Result};

/// Manages all indexes in the database system
pub struct IndexManager {
    // Maps (table_id, index_name) to the index implementation
    indexes: RwLock<HashMap<(u64, String), (IndexType, Arc<dyn Index>)>>,
}

impl IndexManager {
    pub fn new() -> Self {
        Self {
            indexes: RwLock::new(HashMap::new()),
        }
    }
    
    /// Creates a new index of the specified type
    pub fn create_index(&self, table_id: u64, name: &str, index_type: IndexType) -> Result<()> {
        let mut indexes = self.indexes.write();
        
        // Check if index already exists
        if indexes.contains_key(&(table_id, name.to_string())) {
            return Err(MaemioError::System(format!(
                "Index {} already exists for table {}", name, table_id
            )));
        }
        
        // Create the appropriate index type
        let index: Arc<dyn Index> = match index_type {
            IndexType::BTree => Arc::new(BTreeIndex::new()),
            IndexType::Hash => Arc::new(HashIndex::new(1024)), // Default initial capacity
        };
        
        indexes.insert(
            (table_id, name.to_string()),
            (index_type, index)
        );
        
        Ok(())
    }
    
    /// Gets an existing index
    pub fn get_index(&self, table_id: u64, name: &str) -> Result<Arc<dyn Index>> {
        let indexes = self.indexes.read();
        
        indexes.get(&(table_id, name.to_string()))
            .map(|(_, index)| index.clone())
            .ok_or_else(|| MaemioError::System(format!(
                "Index {} not found for table {}", name, table_id
            )))
    }
    
    /// Drops an existing index
    pub fn drop_index(&self, table_id: u64, name: &str) -> Result<()> {
        let mut indexes = self.indexes.write();
        
        indexes.remove(&(table_id, name.to_string()))
            .map(|_| ())
            .ok_or_else(|| MaemioError::System(format!(
                "Index {} not found for table {}", name, table_id
            )))
    }
    
    /// Validates all affected index nodes for a given operation
    pub fn validate_index_access(
        &self,
        table_id: u64,
        name: &str,
        start_key: &IndexKey,
        end_key: &IndexKey,
        ts: u64
    ) -> Result<Vec<Arc<IndexNode>>> {
        let index = self.get_index(table_id, name)?;
        let nodes = index.get_validation_nodes(start_key, end_key);
        
        // Validate each node's timestamps
        for node in &nodes {
            let node_wts = node.wts.load(std::sync::atomic::Ordering::Acquire);
            let node_rts = node.rts.load(std::sync::atomic::Ordering::Acquire);
            
            // Check if this transaction's timestamp is valid for this node
            if node_wts > ts || node_rts > ts {
                return Err(MaemioError::ValidationFailed);
            }
        }
        
        Ok(nodes)
    }
    
    /// Updates timestamps for validated nodes after successful transaction commit
    pub fn update_index_timestamps(
        &self,
        nodes: &[Arc<IndexNode>],
        ts: u64
    ) {
        for node in nodes {
            node.update_rts(ts);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_index_creation() {
        let manager = IndexManager::new();
        
        // Create B-tree index
        assert!(manager.create_index(1, "btree_idx", IndexType::BTree).is_ok());
        
        // Create hash index
        assert!(manager.create_index(1, "hash_idx", IndexType::Hash).is_ok());
        
        // Try to create duplicate index
        assert!(manager.create_index(1, "btree_idx", IndexType::BTree).is_err());
    }
    
    #[test]
    fn test_index_operations() {
        let manager = IndexManager::new();
        
        // Create index
        manager.create_index(1, "test_idx", IndexType::BTree).unwrap();
        
        // Get index
        let index = manager.get_index(1, "test_idx").unwrap();
        
        // Insert and retrieve
        index.insert(IndexKey::Int(1), 100, 1).unwrap();
        assert_eq!(index.get(&IndexKey::Int(1), 2).unwrap(), Some(100));
        
        // Drop index
        assert!(manager.drop_index(1, "test_idx").is_ok());
        assert!(manager.get_index(1, "test_idx").is_err());
    }
    
    #[test]
    fn test_index_validation() {
        let manager = IndexManager::new();
        
        // Create index
        manager.create_index(1, "test_idx", IndexType::BTree).unwrap();
        let index = manager.get_index(1, "test_idx").unwrap();
        
        // Insert some data
        index.insert(IndexKey::Int(1), 100, 1).unwrap();
        index.insert(IndexKey::Int(2), 200, 1).unwrap();
        
        // Validate range access
        let nodes = manager.validate_index_access(
            1,
            "test_idx",
            &IndexKey::Int(1),
            &IndexKey::Int(2),
            2
        ).unwrap();
        
        assert!(!nodes.is_empty());
        
        // Update timestamps
        manager.update_index_timestamps(&nodes, 2);
    }
}