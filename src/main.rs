use std::net::{AddrParseError, SocketAddr};
use std::str::FromStr;

use asset::image::MissingnoImagePlugin;
use asset::tile::{TileDef, TileDefLoader};
use bevy::asset::{AssetIo, AssetIoError};
use bevy::ecs::archetype::Archetypes;
use bevy::ecs::component::ComponentId;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use raw_id::{RawIds, RawIdsLoader};
use serde::{Deserialize, Serialize};

use state::*;
use world::SetTileEvent;

use crate::asset::from_asset_loc;
use crate::asset::locale::{LocaleAsset, LocaleAssetLoader};
use crate::env::EnvType;
use crate::i18n::Translatable;
use crate::identifier::Identifier;
use crate::networking::Username;
use crate::networking::debug::NetworkingDebugPlugin;
use crate::registry::Registry;
use crate::registry::tile::TileRegistry;
use crate::server::ServerPort;

pub mod registry;
pub mod identifier;
pub mod tile;
pub mod raw_id;
pub mod asset;
pub mod i18n;
pub mod env;
pub mod networking;
pub mod utils;
pub mod player;
pub mod world;
pub mod creature;
pub mod state;
pub mod save;
pub mod client;
pub mod server;
pub mod cursor;
pub mod physics;

pub const NAMESPACE: &'static str = "botanica";

pub const VERSION_STRING: &'static str = "0.1.0-alpha.0";

pub const DEFAULT_LOCALE: &'static str = "en_us";

/// Whether the dedicated server is headless.
#[derive(Debug, Default, Copy, Clone, Resource)]
pub struct Headless(bool);

/// This is what is used in the address text box.
#[derive(Debug, Resource)]
pub struct ServerConnectAddress(pub String);

impl TryInto<SocketAddr> for &ServerConnectAddress {
	type Error = AddrParseError;
	
	fn try_into(self) -> Result<SocketAddr, Self::Error> {
		Ok(SocketAddr::from_str(&*self.0)?)
	}
}

impl Default for ServerConnectAddress {
	fn default() -> Self {
		Self(format!("127.0.0.1:{}", ServerPort::default().0))
	}
}

pub fn is_headless(headless: Headless) -> bool {
	headless.0
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, States, Default)]
pub enum GameState {
	/// The stage at which the server or client loads assets. This is always the default state (i.e., it happens first).
	#[default]
	LoadingAssets,
	
	// Client
	/// The splash screen displaying "made with Bevy".
	BevySplash,
	TitleScreen,
	ServerSelect,
	ClientConnecting, // todo: client connecting screen
	WorldSelect,
	LoadingWorld,
	InWorld,
	
	// Server
	ServerLoading,
	ServerLoaded,
}

fn default_asset_plugin() -> AssetPlugin {
	AssetPlugin {
		asset_folder: "assets".to_string(),
		watch_for_changes: None,
	}
}

#[derive(Component, Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TilePos {
	pub x: i32,
	pub y: i32,
}

#[derive(Component, Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Position {
	pub x: f32,
	pub y: f32,
}

pub fn main() {
	let env = EnvType::try_from(std::env::var("ENVIRONMENT").unwrap_or("client".to_string())).unwrap(); // todo: force EnvType environment variable
	let headless = Headless(std::env::args().find(|s| s.as_str() == "--headless").is_some());
	let username = Username(std::env::args().find(|s| s.as_str().starts_with("--username=")).unwrap_or("Player".to_owned()));
	
	let mut app = App::new();
	
	app
		.add_state::<GameState>()
		.init_resource::<networking::stats::PlayerNetStats>()
		.insert_resource(env)
		.insert_resource(headless)
		.init_resource::<TileRegistry>()
		.init_resource::<loading::AssetsLoading>()
		.add_event::<SetTileEvent>()
		.add_plugins(loading::LoadingPlugin)
		.add_plugins(physics::PhysicsPlugin);
	
	if headless.0 && env == EnvType::Server {
		app
			.add_plugins(MinimalPlugins)
			.add_plugins(default_asset_plugin());
	} else {
		app
			.add_plugins(
				DefaultPlugins
					.set(default_asset_plugin())
					.set(ImagePlugin::default_nearest()) // so our sprites appear crisp and clear
					.add_after::<ImagePlugin, _>(MissingnoImagePlugin)
			)
			.add_plugins((EguiPlugin, NetworkingDebugPlugin))
			.add_plugins(
				(
					menu::bevy_splash::BevySplashPlugin,
					menu::title_screen::TitleScreenPlugin,
					menu::world_select::WorldSelectPlugin,
					menu::server_select::ServerSelectPlugin,
				)
			)
			.add_systems(
				Startup,
				menu::init_ui
			);
	}
	
	app
		.add_asset::<LocaleAsset>()
		.init_asset_loader::<LocaleAssetLoader>()
		.add_asset::<TileDef>()
		.init_asset_loader::<TileDefLoader>()
		.add_asset::<RawIds>()
		.init_asset_loader::<RawIdsLoader>();
	
	if env == EnvType::Client {
		app
			.add_plugins(client::NetworkingPlugin)
			.insert_resource(username);
	} else {
		app
			.add_plugins(server::NetworkingPlugin);
	}
	
	app.run();
}

/// Recursively despawns all entities with the component `T`.
pub fn despawn_with<T: Component>(
	mut commands: Commands,
	all: Query<Entity, With<T>>
) {
	for entity in all.iter() {
		commands.entity(entity).despawn_recursive();
	}
}

pub fn is_debug() -> bool {
	#[cfg(feature = "debug")]
	return true;
	#[cfg(not(feature = "debug"))]
	false
}

pub fn get_components_for_entity<'a>(
	entity: &Entity,
	archetypes: &'a Archetypes,
) -> Option<impl Iterator<Item = ComponentId> + 'a> {
	for archetype in archetypes.iter() {
		if archetype.entities().iter().any(|e| &e.entity() == entity) {
			return Some(archetype.components());
		}
	}
	None
}

/// Loads the bytes of an asset.
pub async fn load_asset_bytes(
	path: String,
	asset_io: &dyn AssetIo,
) -> anyhow::Result<Vec<u8>, AssetIoError> {
	asset_io.load_path(path.as_ref()).await
}

fn id(path: &str) -> Identifier {
	Identifier::from_str(NAMESPACE, path)
}
