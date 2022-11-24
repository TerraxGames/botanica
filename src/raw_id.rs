use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Component)]
pub struct RawId(pub u64);
