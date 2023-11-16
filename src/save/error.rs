use thiserror::Error;

#[derive(Debug, Error)]
pub enum SaveReadError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Magic bytes 0x{0:x} are missing!")]
    MagicMissing(u32),
}
