use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct TileSettings {
    #[serde(default)]
    hardness: TileHardness,
    #[serde(default)]
    salience: TileSalience,
}

/// The hardness of a tile.
/// 
/// This is measured in `2x * (1 secs)` where `x` is `TileHardness`.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TileHardness(pub f32);

impl Default for TileHardness {
    fn default() -> Self {
        Self(0.25) // 1/2 of a second
    }
}

/// The salience ("(fore|back)ground-ness") of the tile.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TileSalience {
    #[serde(alias = "fg")]
    Foreground,
    #[serde(alias = "bg")]
    Background,
    #[default]
    Invisible,
}
