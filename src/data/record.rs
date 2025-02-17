use super::Version;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const MAX_INLINE_SIZE: usize = 216;

pub struct RecordHead {
    version_list: RwLock<Option<Box<Version>>>,
    inline_version: RwLock<Option<Version>>,
    min_wts: AtomicU64,
    gc_lock: parking_lot::Mutex<()>,
    creation_timestamp: u64,
}

impl RecordHead {
    pub fn new(creation_ts: u64) -> Self {
        // Instead of creating an initial version, start with no version installed.
        Self {
            version_list: RwLock::new(None),
            inline_version: RwLock::new(None),
            min_wts: AtomicU64::new(creation_ts),
            gc_lock: parking_lot::Mutex::new(()),
            creation_timestamp: creation_ts,
        }
    }

    /// Attempts to create an inline version
    pub fn try_inline_version(&self, version: Version) -> bool {
        // Only inline if the data is small enough.
        if version.data.len() > MAX_INLINE_SIZE {
            return false;
        }

        let mut inline = self.inline_version.write();
        if inline.is_some() {
            return false;
        }

        *inline = Some(version);
        true
    }

    /// Adds a new version to the version chain.
    pub fn add_version(&self, mut version: Version) {
        // First try to inline if possible.
        if !self.try_inline_version(version.clone()) {
            let mut list = self.version_list.write();
            version.next = list.take();
            *list = Some(Box::new(version));
        }
    }

    pub fn install_version(&self, version: Version) -> Result<(), ()> {
        // If the version's data is small enough, try to store it inline.
        if version.data.len() <= MAX_INLINE_SIZE {
            let mut inline = self.inline_version.write();
            if inline.is_none() {
                *inline = Some(version);
                return Ok(());
            } else if version.wts >= inline.as_ref().unwrap().wts {
                // New version is as new or newer than the current inline.
                // Move the old inline version into the version_list.
                let old_inline = inline.take().unwrap();
                {
                    let mut list = self.version_list.write();
                    let mut boxed_old = Box::new(old_inline);
                    boxed_old.next = list.take();
                    *list = Some(boxed_old);
                }
                *inline = Some(version);
                return Ok(());
            }
        }
        // Otherwise, add the version to the version_list.
        let mut list = self.version_list.write();
        let mut new_version = Box::new(version);
        new_version.next = list.take();
        *list = Some(new_version);
        Ok(())
    }
    
    pub fn get_current_version(&self) -> Option<Box<Version>> {
        self.version_list.read().clone()
    }
    
    /// Finds the latest visible version for a given timestamp.
    pub fn find_visible_version(&self, ts: u64) -> Option<Arc<Version>> {
        if ts < self.creation_timestamp {
            return None;
        }

        // Check inline version first.
        {
            let inline = self.inline_version.read();
            if let Some(ref version) = *inline {
                if version.is_visible_to(ts) {
                    return Some(Arc::new(version.clone()));
                }
            }
        }

        // Check version list.
        let list = self.version_list.read();
        let mut current = list.as_ref();
        while let Some(version) = current {
            if version.is_visible_to(ts) {
                return Some(Arc::new((**version).clone()));
            }
            current = version.next.as_ref();
        }

        None
    }

    /// Attempts to acquire the garbage collection lock.
    pub fn try_gc_lock(&self) -> bool {
        self.gc_lock.try_lock().is_some()
    }

    /// Updates the minimum write timestamp.
    pub fn update_min_wts(&self, ts: u64) {
        self.min_wts.store(ts, Ordering::Release);
    }

    pub fn debug_versions(&self) -> String {
        let mut info = String::new();
        let inline = self.inline_version.read();
        if let Some(ref version) = *inline {
            info.push_str(&format!("Inline version - wts: {}, status: {}\n", 
                version.wts,
                version.status.load(Ordering::Acquire)));
        }
        let list = self.version_list.read();
        if let Some(ref version) = *list {
            info.push_str(&format!("List version - wts: {}, status: {}\n",
                version.wts,
                version.status.load(Ordering::Acquire)));
        }
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_visibility() {
        let record = RecordHead::new(0);
        
        // Create two versions.
        let v1 = Version::new(100, vec![1]);
        let v2 = Version::new(200, vec![2]);
        
        // DO NOT commit versions yet.
        // Install versions into the record (newer version installed first).
        record.install_version(v2).unwrap();
        record.install_version(v1).unwrap();
        
        // No version should be visible before commit.
        assert!(record.find_visible_version(150).is_none(), 
            "Uncommitted version should not be visible");
        
        // Now commit v1 (which is stored in version_list).
        if let Some(ref version_in_list) = *record.version_list.read() {
            version_in_list.commit();
        }
        
        // v1 should now be visible.
        let visible = record.find_visible_version(150).unwrap();
        assert_eq!(visible.data, vec![1]);
        
        // v2 is still not visible because it is not committed.
        assert_eq!(record.find_visible_version(250).unwrap().data, vec![1]);
    }
}
