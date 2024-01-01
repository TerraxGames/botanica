use bevy::render::render_resource::{TextureFormat, TextureDescriptor, Extent3d, TextureDimension, TextureUsages};
use bevy::render::texture::ImageSampler;
use bevy::prelude::*;

/// Returns a 2x2 missingno texture.
pub fn missingno(width: u32, height: u32) -> Image {
	let format = TextureFormat::Rgba8UnormSrgb;
	let data = vec![
	 // magenta					 black
		0xFF, 0x00, 0xFF, 0xFF,  0x00, 0x00, 0x00, 0xFF,
	 // black					 magenta
		0x00, 0x00, 0x00, 0xFF,  0xFF, 0x00, 0xFF, 0xFF,
	];
	Image {
		data,
		texture_descriptor: TextureDescriptor {
			size: Extent3d {
				width,
				height,
				depth_or_array_layers: 1,
			},
			format,
			dimension: TextureDimension::D2,
			label: None,
			mip_level_count: 1,
			sample_count: 1,
			usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
			view_formats: &[],
		},
		sampler: ImageSampler::nearest(),
		texture_view_descriptor: None,
	}
}

pub struct MissingnoImagePlugin;

impl Plugin for MissingnoImagePlugin {
    fn build(&self, app: &mut App) {
        app.world
			.resource_mut::<Assets<Image>>()
			.insert(Handle::default(), missingno(2, 2));
    }
}
