// src/index/btree.rs
use std::sync::Arc;
use parking_lot::RwLock;
use super::{Index, IndexKey, IndexNode, MIN_DEGREE};
use crate::error::{MaemioError, Result};

struct BTreeNode {
    // Multi-version node metadata
    mv_node: Arc<IndexNode>,
    // Keys in sorted order
    keys: RwLock<Vec<IndexKey>>,
    // Child pointers
    children: RwLock<Vec<Arc<BTreeNode>>>,
    // Is this a leaf node?
    is_leaf: bool,
}

pub struct BTreeIndex {
    root: RwLock<Arc<BTreeNode>>,
}

impl BTreeNode {
    fn new(is_leaf: bool) -> Self {
        Self {
            mv_node: Arc::new(IndexNode::new()),
            keys: RwLock::new(Vec::with_capacity(2 * MIN_DEGREE - 1)),
            children: RwLock::new(if is_leaf {
                Vec::new()
            } else {
                Vec::with_capacity(2 * MIN_DEGREE)
            }),
            is_leaf,
        }
    }

    fn split_child(&self, child_idx: usize, ts: u64) -> Result<()> {
        let children = self.children.read();
        let child = &children[child_idx];
        
        // Create new node
        let mut new_node = BTreeNode::new(child.is_leaf);
        let mid = MIN_DEGREE - 1;
        
        // Copy keys and children
        {
            let child_keys = child.keys.read();
            let mut new_keys = new_node.keys.write();
            new_keys.extend_from_slice(&child_keys[mid + 1..]);
        }
        
        if !child.is_leaf {
            let child_children = child.children.read();
            let mut new_children = new_node.children.write();
            new_children.extend_from_slice(&child_children[mid + 1..]);
        }
        
        // Update parent
        {
            let child_keys = child.keys.read();
            let mut parent_keys = self.keys.write();
            let mut parent_children = self.children.write();
            
            parent_keys.insert(child_idx, child_keys[mid].clone());
            parent_children.insert(child_idx + 1, Arc::new(new_node));
        }
        
        // Update timestamps
        self.mv_node.wts.store(ts, std::sync::atomic::Ordering::Release);
        child.mv_node.wts.store(ts, std::sync::atomic::Ordering::Release);
        
        Ok(())
    }
}

impl BTreeIndex {
    pub fn new() -> Self {
        Self {
            root: RwLock::new(Arc::new(BTreeNode::new(true))),
        }
    }
}

impl Index for BTreeIndex {
    fn insert(&self, key: IndexKey, record_id: u64, ts: u64) -> Result<()> {
        let root = self.root.read();
        let mut current = root.clone();
        
        // Split root if full
        if current.keys.read().len() == 2 * MIN_DEGREE - 1 {
            let mut new_root = BTreeNode::new(false);
            new_root.children.write().push(current.clone());
            new_root.split_child(0, ts)?;
            *self.root.write() = Arc::new(new_root);
            current = self.root.read().clone();
        }
        
        // Insert non-full
        self.insert_non_full(current, key, record_id, ts)
    }
    
    fn remove(&self, key: &IndexKey, ts: u64) -> Result<()> {
        let root = self.root.read().clone();
        self.remove_key(root, key, ts)
    }
    
    fn get(&self, key: &IndexKey, ts: u64) -> Result<Option<u64>> {
        let root = self.root.read();
        self.search_key(&root, key, ts)
    }
    
    fn range_scan(&self, start: &IndexKey, end: &IndexKey, ts: u64) -> Result<Vec<u64>> {
        let mut result = Vec::new();
        let root = self.root.read();
        self.range_scan_internal(&root, start, end, ts, &mut result)?;
        Ok(result)
    }
    
    fn get_validation_nodes(&self, start: &IndexKey, end: &IndexKey) -> Vec<Arc<IndexNode>> {
        let mut nodes = Vec::new();
        let root = self.root.read();
        self.collect_validation_nodes(&root, start, end, &mut nodes);
        nodes
    }
    
    fn update_timestamps(&self, nodes: &[Arc<IndexNode>], ts: u64) {
        for node in nodes {
            node.update_rts(ts);
        }
    }
}

// Internal implementation methods
impl BTreeIndex {
    fn insert_non_full(&self, node: Arc<BTreeNode>, key: IndexKey, record_id: u64, ts: u64) -> Result<()> {
        let mut i = node.keys.read().len();
        
        if node.is_leaf {
            let mut keys = node.keys.write();
            let mut records = node.mv_node.records.write();
            
            while i > 0 && key < keys[i - 1] {
                i -= 1;
            }
            
            keys.insert(i, key);
            records.push(record_id);
            node.mv_node.wts.store(ts, std::sync::atomic::Ordering::Release);
            Ok(())
        } else {
            let keys = node.keys.read();
            while i > 0 && key < keys[i - 1] {
                i -= 1;
            }
            
            let children = node.children.read();
            let child = children[i].clone();
            
            if child.keys.read().len() == 2 * MIN_DEGREE - 1 {
                node.split_child(i, ts)?;
                let keys = node.keys.read();
                if key > keys[i] {
                    i += 1;
                }
            }
            
            let children = node.children.read();
            self.insert_non_full(children[i].clone(), key, record_id, ts)
        }
    }
    
    fn remove_key(&self, node: Arc<BTreeNode>, key: &IndexKey, ts: u64) -> Result<()> {
        let mut i = 0;
        let keys = node.keys.read();
        
        while i < keys.len() && key > &keys[i] {
            i += 1;
        }
        
        if node.is_leaf {
            if i < keys.len() && key == &keys[i] {
                let mut keys = node.keys.write();
                let mut records = node.mv_node.records.write();
                keys.remove(i);
                records.remove(i);
                node.mv_node.wts.store(ts, std::sync::atomic::Ordering::Release);
                Ok(())
            } else {
                Err(MaemioError::RecordNotFound(0))
            }
        } else {
            let children = node.children.read();
            self.remove_key(children[i].clone(), key, ts)
        }
    }
    
    fn search_key(&self, node: &BTreeNode, key: &IndexKey, ts: u64) -> Result<Option<u64>> {
        let mut i = 0;
        let keys = node.keys.read();
        
        while i < keys.len() && key > &keys[i] {
            i += 1;
        }
        
        if i < keys.len() && key == &keys[i] {
            let records = node.mv_node.records.read();
            Ok(Some(records[i]))
        } else if node.is_leaf {
            Ok(None)
        } else {
            let children = node.children.read();
            self.search_key(&children[i], key, ts)
        }
    }
    
    fn range_scan_internal(
        &self,
        node: &BTreeNode,
        start: &IndexKey,
        end: &IndexKey,
        ts: u64,
        result: &mut Vec<u64>,
    ) -> Result<()> {
        let keys = node.keys.read();
        let records = node.mv_node.records.read();
        
        for i in 0..keys.len() {
            if !node.is_leaf {
                let children = node.children.read();
                self.range_scan_internal(&children[i], start, end, ts, result)?;
            }
            
            if &keys[i] >= start && &keys[i] <= end {
                result.push(records[i]);
            }
        }
        
        if !node.is_leaf {
            let children = node.children.read();
            if let Some(last_child) = children.last() {
                self.range_scan_internal(last_child, start, end, ts, result)?;
            }
        }
        
        Ok(())
    }
    
    fn collect_validation_nodes(
        &self,
        node: &BTreeNode,
        start: &IndexKey,
        end: &IndexKey,
        nodes: &mut Vec<Arc<IndexNode>>,
    ) {
        nodes.push(node.mv_node.clone());
        
        if !node.is_leaf {
            let children = node.children.read();
            for child in children.iter() {
                let child_keys = child.keys.read();
                if let (Some(min), Some(max)) = (child_keys.first(), child_keys.last()) {
                    if min <= end && max >= start {
                        self.collect_validation_nodes(child, start, end, nodes);
                    }
                }
            }
        }
    }
}