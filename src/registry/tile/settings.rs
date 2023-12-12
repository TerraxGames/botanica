use serde::{Serialize, Deserialize, de::Visitor};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct TileSettings {
	#[serde(default)]
	hardness: TileHardness,
	#[serde(default)]
	salience: TileSalience,
}

impl TileSettings {
	pub fn hardness(&self) -> TileHardness {
		self.hardness
	}
	
	pub fn salience(&self) -> TileSalience {
		self.salience
	}
}

/// The hardness of a tile.
/// 
/// This is measured in `2x * (1 secs)` where `x` is `TileHardness`.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct TileHardness(pub f32);

impl Default for TileHardness {
	fn default() -> Self {
		Self(0.25) // 1/2 of a second
	}
}

impl Serialize for TileHardness {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer {
		serializer.serialize_f32(self.0)
	}
}

impl<'de> Deserialize<'de> for TileHardness {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de> {
		struct TileHardnessVisitor;
		
		impl<'de> Visitor<'de> for TileHardnessVisitor {
			type Value = TileHardness;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a f32")
			}
			
			fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
				where
					E: serde::de::Error, {
				Ok(TileHardness(v))
			}
		}
		
		deserializer.deserialize_f32(TileHardnessVisitor)
	}
}

/// The salience (foreground-ness/background-ness) of the tile.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TileSalience {
	#[default]
	Invisible = -1000,
	#[serde(alias = "bg")]
	Background = 0,
	#[serde(alias = "fg")]
	Foreground = 1,
}

impl TileSalience {
	/// Translates the salience property into a Z coordinate.
	#[inline]
	pub fn into_z(&self) -> f32 {
		(*self as i32) as f32
	}
}
