use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Component)]
pub struct RawId(pub u64);
