use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::{EnvType, from_asset_loc, GameState, NAMESPACE};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(OnEnter(GameState::Loading), load_assets)
			.add_systems(
				Update,
				check_assets_ready
					.run_if(in_state(GameState::Loading))
			);
	}
}

#[derive(Default, Resource)]
pub struct AssetsLoading {
	assets: Vec<HandleUntyped>,
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
) {
	println!("Loading assets");
	
	// load all locale files
	let locale = asset_server.load_folder(from_asset_loc(NAMESPACE, "locale")).expect(format!("locale folder should be present in assets/{}", NAMESPACE).as_str());
	// load all fonts
	let fonts = asset_server.load_folder(from_asset_loc(NAMESPACE, "fonts")).expect(format!("fonts folder should be present in assets/{}", NAMESPACE).as_str());
	// load all textures
	let textures = asset_server.load_folder(from_asset_loc(NAMESPACE, "textures")).expect(format!("textures folder should be present in assets/{}", NAMESPACE).as_str());
	// load all tiles
	let tiles = asset_server.load_folder(from_asset_loc(NAMESPACE, "tiles")).expect(format!("tiles folder should be present in assets/{}", NAMESPACE).as_str());
	
	// add all assets to tracker
	for folder in [locale, fonts, textures, tiles] {
		for handle in folder {
			loading.assets.push(handle.clone());
		}
	}
}

fn check_assets_ready(
	mut next_state: ResMut<NextState<GameState>>,
	asset_server: Res<AssetServer>,
	mut loading: ResMut<AssetsLoading>,
	env: Res<EnvType>,
) {
	for handle in loading.assets.iter() {
		let load_state = asset_server.get_load_state(handle.id());
		match load_state {
			LoadState::Failed => {
				let path = asset_server.get_handle_path(handle.id()).expect(format!("expected asset handle (ID {:?}) to have path", handle.id()).as_str());
				warn!("Failed to load asset at \"{:?}\"", path.path());
			},
			_ => {},
		}
	}
	
	match asset_server.get_group_load_state(loading.assets.iter().map(|handle| handle.id())) {
		LoadState::Loaded => {
			println!("Assets finished loading");
			
			loading.finished = true;
			
			next_state.set(
				if *env == EnvType::Client {
					GameState::BevySplash
				} else {
					GameState::ServerLoading
				}
			)
		},
		_ => {},
	}
}
