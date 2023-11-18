use std::collections::HashMap;

use bevy::utils::default;

use crate::{save::format::WorldSave, registry::tile::TileRegistry, tile::WorldTile, TilePos};

use self::error::SaveError;

pub mod format;
pub mod error;

pub const SAVE_DIR: &'static str = "/saves/worlds";

/// ## Warning
/// You **must** ensure that the name is sanitized!
pub fn open_world(name: &str) -> Result<WorldSave, SaveError> {
	let mut path = std::env::current_dir().unwrap();
	path.push(format!("{}/{}.dat", SAVE_DIR, name));
	
	if !path.exists() {
		return Err(SaveError::WorldNonexistent)
	}
	
	WorldSave::deserialize(std::fs::read(path)?)
}

/// ## Warning
/// You **must** ensure that the name is sanitized!
pub fn open_or_gen_world(name: &str, registry: &TileRegistry) -> Result<WorldSave, SaveError> {
	let world = open_world(name);
	if let Err(err) = world {
		match err {
			SaveError::WorldNonexistent => {
				let mut tiles = HashMap::default();
				let grass = WorldTile::new(&crate::id("grass"), registry).unwrap();
				for x in -16..=16 {
					tiles.insert(TilePos(x, 8), grass.clone());
				}
				
				Ok(WorldSave {
					tiles,
					..default()
				})
			},
			_ => Err(err),
		}
	} else { world }
}

/// ## Warning
/// You **must** ensure that the name is sanitized!
pub fn save_world(name: &str, save: &WorldSave) -> Result<(), SaveError> {
	let mut path = std::env::current_dir().unwrap();
	path.push(format!("{}/{}.dat", SAVE_DIR, name));
	
	if !path.exists() {
		return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("save not found at {}", path.as_os_str().to_string_lossy().to_string())).into())
	}
	
	std::fs::write(path, save.serialize()?)?;
	Ok(())
} 
