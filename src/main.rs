pub mod registry;
pub mod identifier;
pub mod tile;
pub mod raw_id;
pub mod asset;
pub mod i18n;
pub mod env;
pub mod networking;
pub mod util;
pub mod player;
pub mod world;
pub mod creature;
mod state;

use std::any::TypeId;
use bevy::asset::{AssetIo, AssetIoError};
use bevy::ecs::archetype::Archetypes;
use bevy::ecs::component::ComponentId;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use iyes_loopless::prelude::*;
use crate::asset::from_asset_loc;
use crate::asset::locale::{LocaleAsset, LocaleAssetLoader};
use crate::env::EnvType;
use crate::i18n::Translatable;
use crate::identifier::Identifier;
use crate::registry::Registry;
use crate::registry::tile::TileRegistry;
use state::*;
use crate::networking::debug::NetworkingDebugPlugin;
use crate::networking::{client, server, Username};
use crate::server::{ServerAddress, ServerConfig, ServerPort};
use serde::{Serialize, Deserialize};

pub const NAMESPACE: &'static str = "botanica";

pub const DEFAULT_LOCALE: &'static str = "en_us";

/// Whether the dedicated server is headless.
#[derive(Debug, Default, Copy, Clone)]
pub struct Headless(bool);

/// fixme: [`ServerAddressPort::default`]
pub struct ServerAddressPort(pub String, pub u16);

impl Default for ServerAddressPort {
	fn default() -> Self {
		let address = ServerAddress::default().0;
		let port = ServerPort::default().0;
		Self(format!("{}:{}", address, port), port) // what the fuck
	}
}

pub fn is_headless(headless: Res<Headless>) -> bool {
	headless.0
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
	Loading,
	/// The splash screen displaying "made with Bevy".
	BevySplash,
	TitleScreen,
	ServerSelect,
	ClientConnecting, // todo: client connecting screen
	WorldSelect,
	LoadingWorld,
	InWorld,
	ServerLoading,
	ServerLoaded,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct TilePos(pub u64, pub u64);

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Component)]
pub struct Position(pub f64, pub f64);

pub fn main() {
	let env = EnvType::try_from(std::env::var("ENVIRONMENT").unwrap_or("client".to_string())).unwrap(); // todo: force EnvType environment variable
	let headless = Headless(std::env::args().find(|s| s.as_str() == "--headless").is_some());
	let username = Username(std::env::args().find(|s| s.as_str().starts_with("--username=")).unwrap_or("Player".to_owned()));
	
	let mut app = App::new();
	
	app
		.add_plugins(DefaultPlugins)
		.add_plugin(EguiPlugin)
		.add_plugin(NetworkingDebugPlugin)
		.insert_resource(env)
		.insert_resource(headless)
		.init_resource::<loading::AssetsLoading>()
		.init_resource::<TileRegistry>() // todo: tile registry and other registries
		.add_asset::<LocaleAsset>()
		.init_asset_loader::<LocaleAssetLoader>()
		.add_startup_system(
			menu::init_ui
				.run_if(env::is_client)
		)
		.add_plugin(loading::LoadingPlugin)
		.add_plugin(menu::bevy_splash::BevySplashPlugin)
		.add_plugin(menu::title_screen::TitleScreenPlugin)
		.add_plugin(menu::world_select::WorldSelectPlugin)
		.add_plugin(menu::server_select::ServerSelectPlugin);
	
	if env == EnvType::Client {
		app
			.add_plugin(client::NetworkingPlugin)
			.insert_resource(username)
			.init_resource::<ServerAddressPort>();
	} else {
		app
			.add_plugin(server::NetworkingPlugin)
			.init_resource::<ServerConfig>();
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
		if archetype.entities().contains(entity) {
			return Some(archetype.components());
		}
	}
	None
}

/// # Safety
/// When calling this function, you must guarantee that the following criteria are met:
/// 
/// * This is a unique mutable pointer to the component.
pub unsafe fn mut_component_for_entity<'a, C>(
	entity: &Entity,
	world: &World,
) -> Option<&'a mut C> where C: Component {
	let components = get_components_for_entity(entity, world.archetypes());
	let mut component: Option<&'a mut C> = None;
	
	for component_id in components.expect("No components found for entity") {
		let info = world.components().get_info(component_id).unwrap();
		if info.type_id().unwrap() == TypeId::of::<C>() {
			for table in world.storages().tables.iter() {
				if let Some(column) = table.get_column(component_id) {
					// SAFE: the caller must guarantee that rust mutability rules aren't violated
					let mut ptr = unsafe { column.get_data_ptr() }.cast::<C>();
					let val = unsafe { ptr.as_mut() };
					component = Some(val);
				}
			}
		}
	}
	
	component
}

pub fn component_for_entity<'a, C>(
	entity: &Entity,
	world: &World,
) -> Option<&'a C> where C: Component {
	// SAFE: this is a shared reference, so we can ignore rust mutability rules
	if let Some(component) = unsafe { mut_component_for_entity(entity, world) } {
		return Some(component);
	}
	None
}

/// Loads the bytes of an asset
pub async fn load_asset_bytes(
	path: String,
	asset_io: &dyn AssetIo,
) -> anyhow::Result<Vec<u8>, AssetIoError> {
	asset_io.load_path(path.as_ref()).await
}

fn id(id: &str) -> Identifier {
	Identifier::new(NAMESPACE, id)
}
