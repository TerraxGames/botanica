use bevy::{asset::{AssetLoader, LoadedAsset}, reflect::{TypeUuid, TypePath}};
use serde::Deserialize;

use crate::{registry::tile::settings::TileSettings, identifier::Identifier, i18n::Translatable};

fn id_default() -> Identifier {
	Identifier::from_str("null", "null")
}

fn trans_default() -> Translatable {
	Translatable::from_str("null")
}

#[derive(Debug, Deserialize, TypeUuid, TypePath)]
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
			name: Translatable::new(format!("{}:tile.name.{}", identifier.namespace(), identifier.id())),
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
}

#[derive(Default)]
pub struct TileDefLoader;

impl AssetLoader for TileDefLoader {
	fn load<'a>(
		&'a self,
		bytes: &'a [u8],
		load_context: &'a mut bevy::asset::LoadContext,
	) -> bevy::utils::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
		Box::pin(async move {
			let mut def: TileDef = ron::de::from_bytes(bytes)?;
			let default_id = load_context.path().file_name().expect("file path should not terminate with \"..\"").to_string_lossy().to_string().replace(".tile.ron", "");
			let default_namespace = load_context.path().ancestors().nth(1).expect("tile definition file should be in directory \"<namespace>/tiles\"").to_string_lossy().to_string();
			if def.identifier.id() == "null" && def.identifier.namespace() == "null" {
				def.identifier = Identifier::new(default_namespace, default_id);
			}
			
			if def.name().key() == "null" {
				def.name = Translatable::new(format!("{}:tile.name.{}", def.identifier.namespace(), def.identifier.id()));
			}
			
			load_context.set_labeled_asset(&def.identifier.to_string(), LoadedAsset::new(def));
			
			Ok(())
		})
	}

	fn extensions(&self) -> &[&str] {
		&["tile.ron"]
	}
}
