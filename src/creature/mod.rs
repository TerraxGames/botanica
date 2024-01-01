use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::{utils::math::{Velocity, ToScale}, physics::HasGravity, networking::NetworkId};

pub mod player;

#[derive(Component, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Creature;

#[derive(Bundle, Clone, Default)]
pub struct CreatureBundle {
	pub creature: Creature,
	pub to_scale: ToScale,
	pub gravity: HasGravity,
	pub network_id: NetworkId,
	pub sprite: SpriteBundle,
	pub velocity: Velocity,
}
