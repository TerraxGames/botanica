use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::networking::protocol::ClientId;
use crate::raw_id::RawId;
use crate::registry::tile::TileData;
use crate::TilePos;

pub type WorldTile = (RawId, TileData);

#[derive(Resource, Default)]
pub struct GameWorlds(HashMap<String, WorldId>, HashMap<WorldId, GameWorld>);

// me when overcomplicated code that i will never use
impl GameWorlds {
	pub fn get_world_id(&self, world_name: &str) -> Option<WorldId> {
		self.0.get(world_name)
			.map(|x| *x)
	}
	
	pub fn get_world(&self, world_id: WorldId) -> Option<&GameWorld> {
		self.1.get(&world_id)
	}
	
	pub fn get_world_mut(&mut self, world_id: WorldId) -> Option<&mut GameWorld> {
		self.1.get_mut(&world_id)
	}
	
	pub fn get_world_from_name(&self, world_name: &str) -> Option<&GameWorld> {
		self.1.get(&self.get_world_id(world_name)?)
	}
	
	pub fn get_world_mut_from_name(&mut self, world_name: &str) -> Option<&mut GameWorld> {
		self.1.get_mut(&self.get_world_id(world_name)?)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct GameWorld {
	id: WorldId,
	name: &'static str,
	tiles: HashMap<TilePos, WorldTile>,
	players: Vec<ClientId>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldId(pub u64);
