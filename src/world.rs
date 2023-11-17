use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};

use bevy::prelude::*;
use bevy::utils::label::DynHash;
use serde::{Deserialize, Serialize};

use crate::networking::protocol::ClientId;
use crate::tile::WorldTile;
use crate::TilePos;

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
	
	pub fn add_world(&mut self, world_name: String, world: GameWorld) {
		let mut hasher = self.0.hasher().build_hasher();
		world_name.dyn_hash(&mut hasher);
		let id = WorldId(hasher.finish());
		self.0.insert(world_name, id);
		self.1.insert(id, world);
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct GameWorld {
	id: WorldId,
	name: String,
	tiles: HashMap<TilePos, WorldTile>,
	players: Vec<ClientId>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldId(pub u64);
