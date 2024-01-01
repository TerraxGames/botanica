use crate::utils::BevyHashMap;
use std::time::SystemTime;

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::{TilePos, Position};
use crate::identifier::Identifier;
use crate::networking::Username;
use crate::raw_id::RawId;
use crate::raw_id::tile::RawTileIds;
use crate::save::error::SaveError;
use crate::save::open_or_gen_world;
use crate::tile::{WorldTile, TileData};

#[derive(Resource, Default)]
pub struct ServerGameWorlds(BevyHashMap<String, ServerGameWorld>);

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
			let world = ServerGameWorld {
				id: WorldId(world_name.to_string()),
				tiles: save.tiles,
				players: default(),
				bans: save.bans,
				spawnpoint: Position { x: 0.0, y: 1.0 }, // todo: implement customizable spawnpoint
			};
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

/// An event that sets a tile.
#[derive(Event)]
pub struct SetTileEvent {
	pub pos: TilePos,
	pub id: Identifier,
	pub data: TileData,
}

#[derive(Debug, Error)]
pub enum TileEventError {
	#[error("raw id for {0} (at {1:?}) does not exist")]
	InvalidId(Identifier, TilePos),
	#[error("tile definition for {0} (at {1:?}) does not exist")]
	TileDefNotFound(Identifier, TilePos),
	#[error("{0} (at {1:?}) does not exist")]
	InvalidRawId(RawId, TilePos),
}

pub const TILE_EVENT_ERROR_MESSAGE: &'static str = "A TileEventError has occurred";

#[derive(Clone)]
pub struct ServerGameWorld {
	pub id: WorldId,
	pub tiles: BevyHashMap<TilePos, WorldTile>,
	pub players: Vec<Entity>,
	pub bans: BevyHashMap<Username, WorldBan>,
	pub spawnpoint: Position,
}

#[derive(Clone, Resource)]
pub struct ClientGameWorld {
	pub id: WorldId,
	pub tiles: BevyHashMap<TilePos, WorldTile>,
	pub tile_sprites: BevyHashMap<TilePos, Entity>,
}

impl ClientGameWorld {
	pub fn get_tile(&self, pos: &TilePos) -> Option<&WorldTile> {
		self.tiles.get(pos)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, DerefMut, Serialize, Deserialize, Component)]
pub struct WorldId(pub String);
