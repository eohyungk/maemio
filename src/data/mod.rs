mod version;
mod record;

pub use version::Version;
pub use record::RecordHead;

// Export common constants
pub const VERSION_STATUS_UNUSED: u8 = 0;
pub const VERSION_STATUS_PENDING: u8 = 1;
pub const VERSION_STATUS_COMMITTED: u8 = 2;
pub const VERSION_STATUS_ABORTED: u8 = 3;
pub const VERSION_STATUS_DELETED: u8 = 4;