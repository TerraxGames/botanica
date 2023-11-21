use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};
use std::time::{Duration, SystemTime};

use bevy::prelude::*;
use bevy::utils::label::DynHash;
use serde::{Serialize, Deserialize};

use crate::TilePos;
use crate::networking::Username;
use crate::networking::protocol::ClientId;
use crate::raw_id::RawIds;
use crate::raw_id::tile::RawTileIds;
use crate::save::error::SaveError;
use crate::save::format::WorldSave;
use crate::save::open_or_gen_world;
use crate::tile::WorldTile;

#[derive(Resource, Default)]
pub struct GameWorlds(HashMap<String, GameWorld>);

impl GameWorlds {
	pub fn get_world(&self, world_name: &str) -> Option<&GameWorld> {
		self.0.get(world_name)
	}
	
	pub fn get_world_mut(&mut self, world_name: &str) -> Option<&mut GameWorld> {
		self.0.get_mut(world_name)
	}
	
	pub fn get_or_gen_world_mut(&mut self, world_name: &str, raw_tile_ids: &RawTileIds) -> Result<&mut GameWorld, SaveError> {
		if self.0.contains_key(world_name) {
			Ok(self.0.get_mut(world_name).unwrap())
		} else {
			let save = open_or_gen_world(world_name, raw_tile_ids)?;
			let world = GameWorld::new(world_name.to_string(), save);
			self.add_world(world_name.to_string(), world);
			Ok(self.0.get_mut(world_name).unwrap())
		}
	}
	
	pub fn add_world(&mut self, world_name: String, world: GameWorld) {
		self.0.insert(world_name, world);
	}
	
	pub fn remove_world(&mut self, world_name: &str) {
		self.0.remove(world_name);
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorldBanUntil(SystemTime);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBan {
	reason: String,
	until: WorldBanUntil,
}

impl WorldBan {
	pub fn new(reason: String, until: WorldBanUntil) -> Self {
		Self {
			reason,
			until,
		}
	}
	
	pub fn reason(&self) -> &str {
		&self.reason
	}
	
	pub fn until(&self) -> WorldBanUntil {
		self.until
	}
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Component)]
pub struct GameWorld {
	name: String,
	tiles: HashMap<TilePos, WorldTile>,
	players: Vec<ClientId>,
	bans: HashMap<Username, WorldBan>,
}

impl GameWorld {
	pub fn new(name: String, save: WorldSave) -> Self {
		Self {
			name,
			tiles: save.tiles,
			bans: save.bans,
			..default()
		}
	}
	
	pub fn name(&self) -> &str {
		self.name.as_str()
	}
	
	pub fn bans(&self) -> &HashMap<Username, WorldBan> {
		&self.bans
	}
	
	pub fn tiles(&self) -> &HashMap<TilePos, WorldTile> {
		&self.tiles
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldId(pub u64);
