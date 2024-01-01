use bevy::asset::{LoadState, LoadedFolder};
use bevy::prelude::*;

use crate::DEFAULT_LOCALE;
use crate::asset::locale::LocaleAsset;
use crate::asset::tile::TileDef;
use crate::i18n::{TranslationServer, CurrentLocale};
use crate::identifier::Identifier;
use crate::raw_id::RawId;
use crate::registry::tile::TileRegistry;
use crate::utils::fatal_error_systems;
use crate::{EnvType, from_asset_loc, GameState, NAMESPACE, raw_id::{RawIds, tile::RawTileIds}};

#[derive(Deref, DerefMut, Resource)]
pub struct AssetPaths(Vec<String>);

impl Default for AssetPaths {
    fn default() -> Self {
        Self(vec![format!("{}", NAMESPACE)])
    }
}

const LOADING_ERROR: &'static str = "an error occurred during loading";

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<AssetPaths>()
			.add_systems(
				OnEnter(GameState::LoadingAssets),
				fatal_error_systems!(LOADING_ERROR, anyhow::Error, load_assets),
			)
			.add_systems(
				Update,
				fatal_error_systems!(LOADING_ERROR, anyhow::Error, check_assets_ready)
					.run_if(in_state(GameState::LoadingAssets))
			);
	}
}

#[derive(Default, Resource)]
pub struct AssetsLoading {
	assets: Vec<Handle<LoadedFolder>>,
	finished: bool,
}

impl AssetsLoading {
	pub fn finished(&self) -> bool {
		self.finished
	}
}

fn load_assets(
	asset_server: Res<AssetServer>,
	mut loading: ResMut<AssetsLoading>,
	asset_paths: Res<AssetPaths>,
	mut tile_registry: ResMut<TileRegistry>,
	loaded_folder_assets: Res<Assets<LoadedFolder>>,
) -> anyhow::Result<()> {
	println!("Loading assets");
	
	for asset_path in asset_paths.iter() {
		// add handles to tile definitions in tile registry
		let tiles_handle = asset_server.load_folder(from_asset_loc(asset_path, "tiles"));
		let tiles_folder: Option<&LoadedFolder> =  loaded_folder_assets.get(tiles_handle);
		if let Some(tiles_folder) = tiles_folder {
			for handle in tiles_folder.handles {
				let handle: Handle<TileDef> = handle.typed();
				let path = asset_server.get_path(handle);
				if let Some(path) = path {
					let mut path_buf = path.path().to_path_buf();
					path_buf.set_extension("");
					path_buf.set_extension("");
					let name = path_buf.file_name();
					if let Some(name) = name {
						let name = name.to_string_lossy().to_string();
						tile_registry.register(handle, Identifier::new(asset_path.clone(), name));
					}
				}
			}
		}
		
		// load all locale files
		let locale = asset_server.load_folder(from_asset_loc(asset_path, "locale"));
		// load all fonts
		let fonts = asset_server.load_folder(from_asset_loc(asset_path, "fonts"));
		// load all textures
		let textures = asset_server.load_folder(from_asset_loc(asset_path, "textures"));
		// load raw ids
		let ids = asset_server.load_folder(from_asset_loc(asset_path, "ids"));
		// load all tiles
		let tiles = asset_server.load_folder(from_asset_loc(asset_path, "tiles"));
		
		// add all assets to tracker
		for folder in [locale, fonts, textures, ids, tiles] {
			loading.assets.push(folder.clone());
		}
	}
	
	Ok(())
}

fn check_assets_ready(
	mut next_state: ResMut<NextState<GameState>>,
	asset_server: Res<AssetServer>,
	mut loading: ResMut<AssetsLoading>,
	env: Res<EnvType>,
	mut commands: Commands,
	asset_paths: Res<AssetPaths>,
	locale_assets: Res<Assets<LocaleAsset>>,
	raw_ids_assets: Res<Assets<RawIds>>,
	loaded_folder_assets: Res<Assets<LoadedFolder>>,
) -> anyhow::Result<()> {
	let path_expect = "asset handle should have path";
	
	for handle in loading.assets.iter() {
		let load_state = asset_server.load_state(handle);
		match load_state {
			LoadState::Failed => {
				let path = asset_server.get_path(handle).expect(path_expect);
				warn!("Failed to load asset at {:?}", path.path());
			},
			_ => {},
		}
	}
	
	let mut loaded = false;
	for folder in loading.assets {
		match asset_server.load_state(folder) {
			LoadState::Loaded => {
				// signal we may have loaded, and continue
				loaded = true;
				continue
			},
			LoadState::Failed => {
				panic!("asset load state failed");
			},
			_ => {
				// signal we're not loaded yet, and break
				loaded = false;
				break
			},
		}
	}
	
	if loaded {
		loading.finished = true;
		
		if *env == EnvType::Server {
			let mut raw_tile_ids: RawTileIds = default();
			for asset_path in asset_paths.iter() {
				let raw_ids_folder_handle: Handle<LoadedFolder> = asset_server.load(format!("{asset_path}/ids"));
				let raw_ids_folder: Option<&LoadedFolder> = loaded_folder_assets.get(raw_ids_folder_handle);
				if let Some(raw_ids_folder) = raw_ids_folder {
					for handle in raw_ids_folder.handles {
						let raw_tile_ids_asset = raw_ids_assets.get(handle.clone()).unwrap();
						let file_name = asset_server.get_path(handle.typed::<RawIds>()).unwrap().path().file_name().unwrap().to_string_lossy().to_string();
						for id in raw_tile_ids_asset.get_ids() {
							let raw_id = raw_tile_ids_asset.get_raw_id(id).unwrap();
							raw_tile_ids.register(id.clone(), raw_id);
						}
					}
				}
			}
			
			// add air tile
			raw_tile_ids.register(Identifier::from_str("null", "air"), RawId(-1));
			// add missingno tile
			raw_tile_ids.register(Identifier::from_str("null", "null"), RawId(-2));
			
			commands.insert_resource(raw_tile_ids);
		}
		
		let current_locale = CurrentLocale::new(DEFAULT_LOCALE.to_string()); // todo: change locale based on settings
		
		let mut translation_server = TranslationServer::new(asset_server.clone());
		
		let locale_assets = locale_assets.into_inner();
		for asset_path in asset_paths.iter() {
			translation_server.load_all(&asset_path, current_locale.locale(), locale_assets)?;
		}
		
		commands.insert_resource(translation_server);
		commands.insert_resource(current_locale);
		
		next_state.set(
			if *env == EnvType::Client {
				GameState::BevySplash
			} else {
				GameState::ServerLoading
			}
		)
	}
	
	Ok(())
}
