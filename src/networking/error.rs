use thiserror::Error;

use crate::save;

pub const NETWORK_ERROR_MESSAGE: &'static str = "A network error has occurred";

#[derive(Debug, Error)]
pub enum NetworkError {
	#[error("error during serialization: {0}")]
	Serialization(#[from] bincode::Error),
	#[error("save error: {0}")]
	SaveError(#[from] save::error::SaveError),
	#[error("error querying entity: {0}")]
	QueryEntityError(#[from] bevy::ecs::query::QueryEntityError),
	#[error("tile event error: {0}")]
	TileEventError(#[from] crate::world::TileEventError),
}
