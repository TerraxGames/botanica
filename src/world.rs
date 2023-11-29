use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};
use std::time::SystemTime;

use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::TilePos;
use crate::networking::Username;
use crate::raw_id::tile::RawTileIds;
use crate::save::error::SaveError;
use crate::save::format::WorldSave;
use crate::save::open_or_gen_world;
use crate::tile::WorldTile;

#[derive(Resource, Default)]
pub struct ServerGameWorlds(HashMap<String, ServerGameWorld>);

impl ServerGameWorlds {
	/// Returns [Some]\(&[GameWorld]) if the world has been loaded. Returns [None] if the world is unloaded.
	pub fn get_world(&self, world_name: &str) -> Option<&ServerGameWorld> {
		self.0.get(world_name)
	}
	
	/// Returns [Some]\(&mut [GameWorld]) if the world has been loaded. Returns [None] if the world is unloaded.
	pub fn get_world_mut(&mut self, world_name: &str) -> Option<&mut ServerGameWorld> {
		self.0.get_mut(world_name)
	}
	
	/// Gets, loads, or generates the specified [GameWorld].
	pub fn get_or_gen_world_mut(&mut self, world_name: &str, raw_tile_ids: &RawTileIds) -> Result<&mut ServerGameWorld, SaveError> {
		if self.0.contains_key(world_name) {
			Ok(self.0.get_mut(world_name).unwrap())
		} else {
			let save = open_or_gen_world(world_name, raw_tile_ids)?;
			let world = ServerGameWorld::new(world_name.to_string(), save, self.get_world_id(world_name));
			self.add_world(world_name.to_string(), world);
			Ok(self.0.get_mut(world_name).unwrap())
		}
	}
	
	pub fn add_world(&mut self, world_name: String, world: ServerGameWorld) {
		self.0.insert(world_name, world);
	}
	
	pub fn remove_world(&mut self, world_name: &str) {
		self.0.remove(world_name);
	}
	
	pub fn get_world_id(&self, world_name: &str) -> WorldId {
		let mut hasher = self.0.hasher().build_hasher();
		hasher.write(world_name.as_bytes());
		WorldId(hasher.finish())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGameWorld {
	name: String,
	tiles: HashMap<TilePos, WorldTile>,
	players: Vec<Entity>,
	bans: HashMap<Username, WorldBan>,
	id: WorldId,
}

impl ServerGameWorld {
	pub fn new(name: String, save: WorldSave, id: WorldId) -> Self {
		Self {
			name,
			tiles: save.tiles,
			bans: save.bans,
			players: default(),
			id,
		}
	}
	
	pub fn name(&self) -> &str {
		self.name.as_str()
	}
	
	pub fn tiles(&self) -> &HashMap<TilePos, WorldTile> {
		&self.tiles
	}
	
	pub fn players(&self) -> &Vec<Entity> {
		&self.players
	}
	
	pub fn players_mut(&mut self) -> &mut Vec<Entity> {
		&mut self.players
	}
	
	pub fn bans(&self) -> &HashMap<Username, WorldBan> {
		&self.bans
	}
	
	pub fn id(&self) -> WorldId {
		self.id
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct ClientGameWorld {
	pub name: String,
	pub id: WorldId,
	pub tiles: HashMap<TilePos, WorldTile>,
}

impl ClientGameWorld {
	pub fn set_tile(&mut self, pos: TilePos, tile: WorldTile) {
		self.tiles.insert(pos, tile);
	}
	
	pub fn get_tile(&self, pos: &TilePos) -> Option<&WorldTile> {
		self.tiles.get(pos)
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub struct WorldId(pub u64);
