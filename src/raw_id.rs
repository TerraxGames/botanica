use std::collections::HashMap;

use bevy::{prelude::*, reflect::{TypePath, TypeUuid}, asset::{AssetLoader, LoadedAsset}};
use serde::{Deserialize, Serialize};

use crate::identifier::Identifier;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct RawId(pub i32);

impl RawId {
	#[inline]
	pub fn is_air(&self) -> bool {
		self.0 == -1
	}
	
	#[inline]
	pub fn is_missingno(&self) -> bool {
		self.0 == -2
	}
}

// fixme: make this implement Debug rather than Display
impl std::fmt::Display for RawId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("RawId(0x{:X})", self.0))
	}
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, TypeUuid, TypePath)]
#[uuid = "7e34fcef-ef6e-442b-8ec2-576fc73620bf"]
pub struct RawIds(HashMap<Identifier, RawId>);

impl RawIds {
	pub fn get_id(&self, raw_id: RawId) -> Option<&Identifier> {
		self.0.iter()
			.find_map(|(key, &val)| if val == raw_id { Some(key) } else { None })
	}
	
	pub fn get_raw_id(&self, id: &Identifier) -> Option<RawId> {
		Some(*self.0.get(id)?)
	}
	
	pub fn get_ids(&self) -> std::collections::hash_map::Keys<Identifier, RawId> {
		self.0.keys()
	}
	
	pub fn get_raw_ids(&self) -> std::collections::hash_map::Values<Identifier, RawId> {
		self.0.values()
	}
	
	pub fn register(&mut self, id: Identifier, raw_id: RawId) {
		self.0.insert(id, raw_id);
	}
}

pub mod tile {
    use bevy::prelude::*;
    use serde::{Serialize, Deserialize};

    use super::RawIds;

	#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Deref, DerefMut, Resource)]
	pub struct RawTileIds(pub RawIds);
}

#[derive(Default)]
pub struct RawIdsLoader;

impl AssetLoader for RawIdsLoader {
	fn load<'a>(
		&'a self,
		bytes: &'a [u8],
		load_context: &'a mut bevy::asset::LoadContext,
	) -> bevy::utils::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
		Box::pin(async move {
			let default_namespace = load_context.path().ancestors().nth(2).expect("raw ID file should be in directory \"<namespace>/ids\"").file_name().expect("path should not contain \"..\"").to_string_lossy().to_string();
			let id_vec: Vec<Identifier> = ron::de::from_bytes(bytes)?;
			let mut ids = HashMap::new();
			for (i, mut path) in id_vec.into_iter().enumerate() {
				if path.namespace() == "null" {
					path = Identifier::new(default_namespace.clone(), path.path().to_string());
				} // TODO: add "null" check for ID
				
				ids.insert(path, RawId(i as i32));
			}
			
			load_context.set_default_asset(LoadedAsset::new(RawIds(ids)));
			
			Ok(())
		})
	}

	fn extensions(&self) -> &[&str] {
		&["ids.ron"]
	}
}
