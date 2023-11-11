use thiserror::Error;

pub const NETWORK_ERROR_MESSAGE: &'static str = "A network error has occurred";

#[derive(Debug, Error)]
pub enum NetworkError {
	#[error("error during serialization: {0}")]
	Serialization(#[from] bincode::Error),
}
