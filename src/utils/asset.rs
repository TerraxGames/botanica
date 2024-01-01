use bevy::{prelude::*, asset::{AssetPath, LoadState}};

/// Loads the image at the specified asset path or returns missingno if the asset is missing.
pub fn load_image<'a, P>(asset_server: &AssetServer, path: P) -> Handle<Image>
	where
		P: Into<AssetPath<'a>>,
{
	let handle = asset_server.load(path);
	let load_state = asset_server.get_load_state(handle.clone());
	if let Some(load_state) = load_state {
		if load_state == LoadState::Loaded {
			handle
		}
	} else {
		Handle::default()
	}
}
