use bevy::asset::LoadState;
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use crate::{EnvType, from_asset_loc, GameState, NAMESPACE};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_loopless_state(GameState::Loading)
			.add_enter_system(GameState::Loading, load_assets)
			.add_system(
				check_assets_ready
					.run_in_state(GameState::Loading)
			);
	}
}

#[derive(Default)]
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
	println!("Loading assets.");
	
	// load all locale files
	let locale = asset_server.load_folder(from_asset_loc(NAMESPACE, "locale")).expect("Failed to find locale folder.");
	let fonts = asset_server.load_folder(from_asset_loc(NAMESPACE, "fonts")).expect("Failed to find fonts folder.");
	// load all textures
	let textures = asset_server.load_folder(from_asset_loc(NAMESPACE, "textures")).expect("Failed to find textures folder.");
	
	// add all assets to tracker
	for folder in [locale, fonts, textures] {
		for handle in folder {
			loading.assets.push(handle.clone());
		}
	}
}

fn check_assets_ready(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut loading: ResMut<AssetsLoading>,
	env: Res<EnvType>,
) {
	for handle in loading.assets.iter() {
		let load_state = asset_server.get_load_state(handle.id);
		match load_state {
			LoadState::Failed => {
				let path = asset_server.get_handle_path(handle.id).expect("Failed to get path of asset handle");
				warn!("Failed to load asset at \"{:?}\"", path.path());
			},
			_ => {},
		}
	}
	
	match asset_server.get_group_load_state(loading.assets.iter().map(|h| h.id)) {
		LoadState::Loaded => {
			println!("Assets loaded.");
			
			loading.finished = true;
			
			let next_state = if *env == EnvType::Client {
				NextState(GameState::BevySplash)
			} else {
				NextState(GameState::LoadingWorld)
			};
			
			commands.insert_resource(next_state)
		},
		_ => {},
	}
}
