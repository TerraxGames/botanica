use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use bevy::prelude::*;
use serde::Deserialize;
use crate::i18n::Translatable;
use crate::identifier::Identifier;
use crate::Registry;
use crate::registry::def::Definition;

use self::settings::TileSettings;

use super::error::RegistryError;

pub mod settings;

#[derive(Resource)]
pub struct TileRegistry(pub Registry<TileDef>);

impl<'a> Default for TileRegistry {
	fn default() -> Self {
		Self(Registry::new())
	}
}

fn id_default() -> Identifier {
	Identifier::from_str("null", "null")
}

fn trans_default() -> Translatable {
	Translatable::from_str("null")
}

#[derive(Debug, Deserialize)]
pub struct TileDef {
	#[serde(skip, default = "id_default")]
	identifier: Identifier,
	#[serde(default)]
	settings: TileSettings,
	#[serde(default = "trans_default")]
	name: Translatable,
}

impl TileDef {
	pub fn new(identifier: Identifier, settings: TileSettings) -> Self {
		Self {
			name: Translatable::new(format!("{}:tile.name.{}", identifier.namespace(), identifier.id())),
			identifier,
			settings,
		}
	}
	
	pub fn open(path: PathBuf) -> Result<Self, RegistryError> {
		if path.exists() {
			let id = path.file_name().unwrap().to_os_string().into_string()?;
			let namespace = path.ancestors().nth(1).unwrap().as_os_str().to_os_string().into_string()?;

			let mut file = File::open(path)?;
			let mut buf = vec![];
			file.read_to_end(&mut buf)?;
			let str = String::from_utf8(buf)?;
			let mut def: Self = ron::from_str(str.as_str())?;
			if def.identifier.id() == "null" && def.identifier.namespace() == "null" {
				def.identifier = Identifier::new(namespace, id);
			}
			if def.name.key() == "null" {
				def.name = Translatable::new(format!("{}:tile.name.{}", def.identifier.namespace(), def.identifier.id()));
			}
			Ok(def)
		} else {
			Err(std::io::Error::from(std::io::ErrorKind::NotFound).into())
		}
	}

	pub fn name(&self) -> &Translatable {
		&self.name
	}
}

impl Definition for TileDef {
	fn identifier(&self) -> &Identifier {
		&self.identifier
	}
}
