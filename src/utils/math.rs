use std::ops::Div;

use bevy::{prelude::*, render::texture::DEFAULT_IMAGE_HANDLE};

pub const HANDLE_TO_SCALE_ERROR_MESSAGE: &'static str = "An error occurred while rescaling sprites";

#[derive(Component, Debug, Default, Copy, Clone, PartialEq, Deref, DerefMut)]
pub struct Velocity(Transform);

/// A component that marks an entity for scaling.
#[derive(Component, Debug, Default, Clone)]
pub struct ToScale;

pub fn handle_to_scale(
	mut to_scale_sprite_query: Query<(Entity, &mut Sprite, &Handle<Image>), With<ToScale>>,
	mut commands: Commands,
	image_assets: Res<Assets<Image>>,
) -> anyhow::Result<()> {
	for (entity, mut sprite, handle) in to_scale_sprite_query.iter_mut() {
		commands.entity(entity)
			.remove::<ToScale>();
		let image = image_assets.get(handle).unwrap_or(image_assets.get(&DEFAULT_IMAGE_HANDLE.typed_weak::<Image>()).unwrap());
		sprite.custom_size = Some(image.size().div(16.0))
	}
	
	Ok(())
}
