use bevy::prelude::{Assets, AssetServer, Handle, Res};
use serde::de::Visitor;
use serde::Deserialize;

use crate::asset::locale::LocaleAsset;
use crate::asset::parse_namespaced;

#[derive(Debug)]
pub struct Translatable {
	key: String,
}

impl<'de> Deserialize<'de> for Translatable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
		struct StringVisitor;

		impl<'de> Visitor<'de> for StringVisitor {
			type Value = Translatable;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("String")
			}
			
			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
				where
					E: serde::de::Error, {
				Ok(
					Translatable {
						key: v,
					}
				)
			}
		}
		
        deserializer.deserialize_string(StringVisitor)
    }
}

impl Translatable {
	pub fn new(key: String) -> Self {
		Self {
			key,
		}
	}

	pub fn from_str(key: &str) -> Self {
		Self {
			key: key.to_string(),
		}
	}

	pub fn key(&self) -> &str {
		self.key.as_str()
	}

	/// Translates the `&str` translation key once.<br>
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
