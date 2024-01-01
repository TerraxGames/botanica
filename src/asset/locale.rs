use crate::utils::BevyHashMap;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset, Asset};
use bevy::reflect::{TypePath, TypeUuid};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Asset, TypeUuid, TypePath)]
#[uuid = "57ed7713-25b9-4f84-a961-238acca10d96"]
pub struct LocaleAsset {
	translations: BevyHashMap<String, String>,
	locale: String,
}

impl LocaleAsset {
	pub fn translate(&self, key: &str) -> Option<String> {
		Some(self.translations.get(key)?.clone())
	}
	
	pub fn translations(&self) -> &BevyHashMap<String, String> {
		&self.translations
	}
	
	pub fn locale(&self) -> &str {
		&self.locale
	}
}

pub struct LocaleAssetLoaderSettings {
	pub locale: String,
}

#[derive(Default)]
pub struct LocaleAssetLoader;

impl AssetLoader for LocaleAssetLoader {
	type Asset = LocaleAsset;
	type Settings = LocaleAssetLoaderSettings;
	type Error = anyhow::Error;
	
	fn load<'a>(
		&'a self,
		reader: &'a mut Reader,
		settings: &'a Self::Settings,
		load_context: &'a mut LoadContext
	) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
		Box::pin(async move {
			let bytes = vec![];
			reader.read_to_end(&mut bytes);
			
			let locale = ron::de::from_bytes::<BevyHashMap<String, String>>(&bytes)?;
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
