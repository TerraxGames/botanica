use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::networking::protocol::ClientId;
use crate::raw_id::RawId;
use crate::registry::tile::TileData;
use crate::TilePos;

pub type WorldTile = (RawId, TileData);

#[derive(Resource, Default)]
pub struct GameWorlds(pub HashMap<String, WorldId>, pub HashMap<WorldId, GameWorld>);

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct GameWorld {
	id: WorldId,
	name: &'static str,
	tiles: HashMap<TilePos, WorldTile>,
	players: Vec<ClientId>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldId(pub u64);
