use bevy::prelude::*;

use crate::creature::Creature;
use crate::networking::protocol::{ClientId, PlayerData};

#[derive(Debug, Default, Clone, Component)]
pub struct Player; // TODO: make the eyes and arms their own sprites and allow them to rotate and move freely since we're only in 16x16. also, allow changing eye color.

#[derive(Clone, Bundle)]
pub struct PlayerBundle {
	pub creature: Creature,
	pub player: Player,
	pub id: ClientId,
	pub data: PlayerData,
}
