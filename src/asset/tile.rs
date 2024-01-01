use bevy::{asset::{AssetLoader, LoadedAsset, Asset, io::Reader, AsyncReadExt}, reflect::{TypeUuid, TypePath}};
use serde::Deserialize;

use crate::{registry::tile::settings::TileSettings, identifier::Identifier, i18n::Translatable};

fn id_default() -> Identifier {
	Identifier::from_str("null", "null")
}

fn trans_default() -> Translatable {
	Translatable::from_str("null")
}

#[derive(Debug, Deserialize, Asset, TypeUuid, TypePath)]
#[uuid = "e9916291-4fef-4058-8b1b-d4e8f8a23aaf"]
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
			name: Translatable::new(format!("{}:tile.name.{}", identifier.namespace(), identifier.path())),
			identifier,
			settings,
		}
	}
	
	pub fn settings(&self) -> &TileSettings {
		&self.settings
	}
	
	pub fn name(&self) -> &Translatable {
		&self.name
	}
	
	pub fn identifier(&self) -> &Identifier {
		&self.identifier
	}
	
	pub fn is_air(&self) -> bool {
		self.identifier.namespace() == "null" && self.identifier.path() == "air"
	}
	
	pub fn is_missingno(&self) -> bool {
		self.identifier.namespace() == "null" && self.identifier.path() == "null"
	}
}

#[derive(Default)]
pub struct TileDefLoader;

impl AssetLoader for TileDefLoader {
	type Asset = TileDef;
	type Settings = ();
	type Error = anyhow::Error;
	
	fn load<'a>(
		&'a self,
		reader: &'a mut Reader,
		settings: &'a Self::Settings,
		load_context: &'a mut bevy::asset::LoadContext,
	) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
		Box::pin(async move {
			let bytes = vec![];
			reader.read_to_end(&mut bytes);
			
			let mut def: TileDef = ron::de::from_bytes(&bytes)?;
			let default_path = load_context.path().file_name().expect("file path should not terminate with \"..\"").to_string_lossy().to_string().replace(".tile.ron", "");
			let default_namespace = load_context.path().ancestors().nth(2).expect("tile definition file should be in directory \"<namespace>/tiles\"").file_name().expect("directory name should not end in ..").to_string_lossy().to_string();
			if def.identifier.path() == "null" && def.identifier.namespace() == "null" {
				def.identifier = Identifier::new(default_namespace, default_path);
			}
			
			if def.name().key() == "null" {
				def.name = Translatable::new(format!("{}:tile.name.{}", def.identifier.namespace(), def.identifier.path()));
			}
			
			Ok(def)
		})
	}

	fn extensions(&self) -> &[&str] {
		&["tile.ron"]
	}
}
