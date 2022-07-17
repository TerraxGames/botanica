pub mod registry;
pub mod identifier;
pub mod tile;
pub mod raw_id;
pub mod asset;
pub mod i18n;
pub mod env;
mod state;

use bevy::asset::{AssetIo, AssetIoError};
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

pub const NAMESPACE: &'static str = "botanica";

pub const DEFAULT_LOCALE: &'static str = "en_us";

pub struct ServerAddress(pub String);

impl Default for ServerAddress {
	fn default() -> Self {
		Self("localhost".to_owned())
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
	Loading,
	/// The splash screen displaying "made with Bevy".
	BevySplash,
	TitleScreen,
	WorldSelect,
	ServerSelect,
	LoadingWorld,
	InWorld,
}

pub fn main() {
	let env = EnvType::try_from(std::env::var("ENVIRONMENT").unwrap_or("client".to_string())).unwrap(); // todo: force EnvType environment variable
	
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(EguiPlugin)
		.insert_resource(env)
		.init_resource::<loading::AssetsLoading>()
		.init_resource::<ServerAddress>()
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
		.add_plugin(menu::server_select::ServerSelectPlugin)
		.run();
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
