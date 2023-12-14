use bevy::{prelude::*, asset::{AssetPath, LoadState}, render::texture::DEFAULT_IMAGE_HANDLE};

/// Loads the image at the specified asset path or returns missingno if the asset is missing.
pub fn load_image<'a, P>(asset_server: &AssetServer, path: P) -> Handle<Image>
	where
		P: Into<AssetPath<'a>>,
{
	let handle = asset_server.load(path);
	if asset_server.get_load_state(handle.clone()) == LoadState::Loaded {
		handle
	} else {
		DEFAULT_IMAGE_HANDLE.typed()
	}
}
