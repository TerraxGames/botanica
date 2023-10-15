use bevy::prelude::{Assets, AssetServer, Handle, Res};

use crate::asset::locale::LocaleAsset;
use crate::asset::parse_namespaced;

pub struct Translatable {
	key: String,
	translated: Option<String>,
}

impl Translatable {
	pub fn new(key: &str) -> Self {
		Self {
			key: String::from(key),
			translated: None,
		}
	}

	pub fn key(&self) -> &str {
		self.key.as_str()
	}

	pub fn translated(&self) -> Option<&String> {
		match &self.translated {
			Some(translated) => Some(&translated),
			_ => None,
		}
	}

	/// Translates the `Translatable` if it isn't already, otherwise returning a cached translation.<br>
	/// `locale` - The locale string identifier of which the translation is in.<br>
	/// `asset_server` - The `Res<AssetServer>` from which to load the translation from.<br>
	/// `locale_assets` - The `Res<Assets<LocaleAsset>>` from which to retrieve the asset from.<br>
	/// ***Note:** This method requires that the `locale` folder be loaded *before* calling.*
	pub fn translate(&self,
	                 locale: &str,
	                 asset_server: &Res<AssetServer>,
	                 locale_assets: &Res<Assets<LocaleAsset>>,
	) -> String {
		if let Some(translated) = self.translated() {
			translated.clone()
		} else {
			Translatable::translate_once(self.key.as_str(), locale, asset_server, locale_assets)
		}
	}

	/// Translates the `&str` translation key once.,<br>
	/// `key` - The namespaced translation key.<br>
	/// `locale` - The locale string identifier of which the translation is in.<br>
	/// `asset_server` - The `Res<AssetServer>` from which to load the translation from.<br>
	/// `locale_assets` - The `Res<Assets<LocaleAsset>>` from which to retrieve the asset from.<br>
	/// ***Note:** This method requires that the `locale` folder be loaded *before* calling.*
	pub fn translate_once(
		key: &str,
		locale: &str,
		asset_server: &Res<AssetServer>,
		locale_assets: &Assets<LocaleAsset>,
	) -> String {
		let (namespace, key) = parse_namespaced(key);
		let location = format!("{}/locale/{}.locale.ron", namespace, locale);
		let handle: Handle<LocaleAsset> = asset_server.get_handle(location.as_str());
		let locale = locale_assets.get(&handle).expect(format!("Failed to find locale \"{}\" ({}): not loaded.", locale, location).as_str()).clone();
		locale.translate(key)
	}
}
