use bevy::prelude::*;

use crate::{registry::tile::TileRegistry, asset::tile::TileDef};

/// Funky name for a quirky struct.
/// 
/// *Slaps struct* this little fella can hold a Res\<TileRegistry>, a Res<Assets\<TileDef>>, *and* a Res\<AssetServer>.
pub struct TileHub<'a>(Res<'a, TileRegistry>, Res<'a, Assets<TileDef>>, Res<'a, AssetServer>);
