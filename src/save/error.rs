use thiserror::Error;

use crate::{raw_id::RawId, identifier::Identifier};

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
	#[error("bincode error: {0}")]
	BincodeError(#[from] bincode::Error),
    #[error("magic bytes 0x{0:X} are missing!")]
    MagicMissing(u32),
	#[error("invalid save version: 0x{0:X}; expected version 0x{1:X}")]
	InvalidVersion(u32, u32),
	#[error("world does not exist")]
	WorldNonexistent,
	#[error("the raw {0} ID {1} was not found in the save")]
	RawIdNotFoundInSave(String, RawId),
	#[error("the {0} ID {1} does not exist")]
	IdNonexistent(String, Identifier),
}
