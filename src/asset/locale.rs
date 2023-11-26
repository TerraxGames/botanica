use std::collections::HashMap;

use anyhow::Error;
use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::reflect::{TypePath, TypeUuid};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, TypeUuid, TypePath)]
#[uuid = "57ed7713-25b9-4f84-a961-238acca10d96"]
pub struct LocaleAsset {
	translations: HashMap<String, String>,
	locale: String,
}

impl LocaleAsset {
	pub fn translate(&self, key: &str) -> Option<String> {
		Some(self.translations.get(key)?.clone())
	}
	
	pub fn translations(&self) -> &HashMap<String, String> {
		&self.translations
	}
	
	pub fn locale(&self) -> &str {
		&self.locale
	}
}

#[derive(Default)]
pub struct LocaleAssetLoader;

impl AssetLoader for LocaleAssetLoader {
	fn load<'a>(&'a self, bytes: &'a [u8], load_context: &'a mut LoadContext) -> BoxedFuture<'a, anyhow::Result<(), Error>> {
		Box::pin(async move {
			let locale = ron::de::from_bytes::<HashMap<String, String>>(bytes)?;
			load_context.set_default_asset(LoadedAsset::new(LocaleAsset {
				translations: locale,
				locale: load_context.path().to_str().unwrap().to_string(),
			}));
			Ok(())
		})
	}

	fn extensions(&self) -> &[&str] {
		&["locale.ron"]
	}
}
