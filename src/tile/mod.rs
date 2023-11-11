use serde::{Deserialize, Serialize};
use bevy::prelude::Component;
use crate::raw_id::RawId;

pub mod bundle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldTile(pub RawId, pub TileData);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Component)]
pub struct TileData(pub u8); // todo: advanced tile data
