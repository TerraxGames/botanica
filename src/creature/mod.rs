use bevy::prelude::*;

use crate::util::math::Velocity;

pub mod player;

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct Creature;

#[derive(Bundle, Clone, Default)]
pub struct CreatureBundle {
	pub creature: Creature,
	pub sprite: SpriteBundle,
	pub velocity: Velocity,
}
