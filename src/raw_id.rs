use std::collections::HashMap;

use bevy::{prelude::*, reflect::{TypePath, TypeUuid}, asset::{AssetLoader, LoadedAsset}};
use serde::{Deserialize, Serialize};

use crate::identifier::Identifier;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct RawId(pub u32);

#[derive(Debug, TypeUuid, TypePath)]
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
}

pub mod tile {
    use bevy::prelude::*;

    use super::RawIds;

	#[derive(Deref, DerefMut, Resource)]
	pub struct RawTileIds(RawIds);
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
			let default_namespace = load_context.path().ancestors().nth(1).expect("tile id file should be in directory \"<namespace>/tiles\"").to_string_lossy().to_string();
			let id_vec: Vec<Identifier> = ron::de::from_bytes(bytes)?;
			let mut ids = HashMap::new();
			for (i, mut id) in id_vec.into_iter().enumerate() {
				if id.namespace() == "null" {
					id = Identifier::new(default_namespace.clone(), id.id().to_string());
				}
				
				ids.insert(id, RawId(i as u32));
			}
			
			load_context.set_labeled_asset(&default_namespace, LoadedAsset::new(RawIds(ids)));
			
			Ok(())
		})
	}

	fn extensions(&self) -> &[&str] {
		&["ids.ron"]
	}
}
