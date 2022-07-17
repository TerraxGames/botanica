pub mod registry;
pub mod identifier;
pub mod tile;
pub mod raw_id;
pub mod asset;
pub mod i18n;
pub mod env;
mod state;

use bevy::prelude::*;
use iyes_loopless::prelude::*;
use crate::asset::from_asset_loc;
use crate::asset::locale::{LocaleAsset, LocaleAssetLoader};
use crate::env::EnvType;
use crate::i18n::Translatable;
use crate::identifier::Identifier;
use crate::registry::Registry;
use crate::registry::tile::TileRegistry;
use state::*;
use crate::loading::LoadingPlugin;
use crate::menu::bevy_splash::BevySplashPlugin;
use crate::menu::title_screen::TitleScreenPlugin;

pub const NAMESPACE: &'static str = "botanica";

pub const DEFAULT_LOCALE: &'static str = "en_us";

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
		.insert_resource(env)
		.init_resource::<loading::AssetsLoading>()
		.init_resource::<TileRegistry>() // todo: tile registry and other registries
		.add_asset::<LocaleAsset>()
		.init_asset_loader::<LocaleAssetLoader>()
		.add_startup_system(
			menu::init_ui
				.run_if(env::is_client)
		)
		.add_plugin(LoadingPlugin)
		.add_plugin(BevySplashPlugin)
		.add_plugin(TitleScreenPlugin)
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

fn id(id: &str) -> Identifier {
	Identifier::new(NAMESPACE, id)
}
