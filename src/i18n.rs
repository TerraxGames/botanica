use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use serde::de::Visitor;
use serde::Deserialize;
use thiserror::Error;

use crate::asset::locale::LocaleAsset;
use crate::asset::parse_namespaced;

/// Represents the currently loaded locale.
#[derive(Debug, Resource)]
pub struct CurrentLocale(String);

impl CurrentLocale {
	pub fn new(locale: String) -> Self {
		Self(locale)
	}
	
	/// Returns the locale identifier.
	pub fn locale(&self) -> &str {
		&self.0
	}
}

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
}

#[derive(Resource)]
pub struct TranslationServer {
	asset_server: AssetServer,
	/// A map of locales to maps of namespaces to maps of translation keys to translations.
	locales: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

impl TranslationServer {
	pub fn new(asset_server: AssetServer) -> Self {
		Self {
			asset_server,
			locales: HashMap::new(),
		}
	}
	
	/// Translates the given key.
	/// 
	/// This method will retrieve the specified key if it has been loaded. Otherwise, it returns [None](std::option::Option::None).
	pub fn translate(&self, namespace: &str, key: &str, locale: &CurrentLocale) -> Option<&String> {
		self.locales.get(locale.locale())?.get(namespace)?.get(key)
	}
	
	/// Translates the given key or loads it.
	/// 
	/// Useful when you're using lazily loaded translation keys for some godawful reason.
	pub fn translate_or_load(&mut self, namespace: &str, key: &str, locale: &CurrentLocale, locale_assets: &Assets<LocaleAsset>) -> Option<String> {
		let namespace_map = self.locales.get(locale.locale());
		if namespace_map.is_none() {
			let translation = self.load(namespace, key, locale.locale(), locale_assets)?;
			let mut translation_maps = HashMap::new();
			let mut translations = HashMap::new();
			translations.insert(key.to_string(), translation.clone());
			translation_maps.insert(locale.locale().to_string(), translations);
			self.locales.insert(namespace.to_string(), translation_maps);
			return Some(translation)
		}
		
		let translations = namespace_map.unwrap().get(namespace);
		if translations.is_none() {
			let translation = self.load(namespace, key, locale.locale(), locale_assets)?;
			let translation_maps = self.locales.get_mut(locale.locale()).unwrap();
			let mut translations = HashMap::new();
			translations.insert(key.to_string(), translation.clone());
			translation_maps.insert(namespace.to_string(), translations);
			return Some(translation)
		}
		
		let translation = translations.unwrap().get(key);
		if translation.is_none() {
			let translation = self.load(namespace, key, locale.locale(), locale_assets)?;
			let translations = self.locales.get_mut(locale.locale()).unwrap().get_mut(namespace).unwrap();
			translations.insert(key.to_string(), translation.clone());
			return Some(translation)
		} else { Some(translation?.clone()) }
	}
	
	fn load(&self, namespace: &str, key: &str, locale: &str, locale_assets: &Assets<LocaleAsset>) -> Option<String> {
		let locale_asset = locale_assets.get(&self.asset_server.load(format!("{}/locale/{}.locale.ron", namespace, locale).as_str()))?;
		locale_asset.translate(key)
	}
	
	/// Loads all of the translations in a given locale and namespace.
	pub fn load_all(&mut self, namespace: &str, locale: &str, locale_assets: &Assets<LocaleAsset>) -> Result<(), TranslationError> {
		let namespace_map = match self.locales.get_mut(locale) {
			Some(namespace_map) => namespace_map,
			None => {
				self.locales.insert(locale.to_string(), HashMap::new());
				self.locales.get_mut(locale).unwrap()
			},
		};
		
		let translations = match namespace_map.get_mut(namespace) {
			Some(translations) => translations,
			None => {
				namespace_map.insert(namespace.to_string(), HashMap::new());
				namespace_map.get_mut(namespace).unwrap()
			}
		};
		
		let locale_asset_path = format!("{}/locale/{}.locale.ron", namespace, locale);
		let locale_asset = match locale_assets.get(&self.asset_server.load(locale_asset_path.as_str())) {
			Some(locale_asset) => locale_asset,
			None => return Err(TranslationError::HandleNotFound(locale_asset_path)),
		};
		for (key, translation) in locale_asset.translations() {
			translations.insert(key.clone(), translation.clone());
		}
		
		Ok(())
	}
}

#[derive(Debug, Error)]
pub enum TranslationError {
	#[error("handle not found: {0}")]
	HandleNotFound(String),
}
