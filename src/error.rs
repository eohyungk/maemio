use thiserror::Error;

#[derive(Error, Debug)]
pub enum MaemioError {
    #[error("Transaction validation failed")]
    ValidationFailed,
    
    #[error("No visible version found for record")]
    NoVisibleVersion,
    
    #[error("Transaction conflict detected")]
    Conflict,
    
    #[error("Record not found: {0}")]
    RecordNotFound(u64),
    
    #[error("Table not found: {0}")]
    TableNotFound(String),
    
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    
    #[error("System error: {0}")]
    System(String),

    #[error("Version installation failed")]
    VersionInstallationFailed,
    
}
// Implementation to convert unit error () into MaemioError
impl From<()> for MaemioError {
    fn from(_: ()) -> Self {
        MaemioError::VersionInstallationFailed
    }
}
pub type Result<T> = std::result::Result<T, MaemioError>;