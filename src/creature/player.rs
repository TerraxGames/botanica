use bevy::prelude::*;

use crate::creature::Creature;
use crate::networking::protocol::{ClientId, PlayerData};

#[derive(Debug, Default, Clone, Component)]
pub struct Player;

#[derive(Clone, Bundle)]
pub struct PlayerBundle {
	pub creature: Creature,
	pub player: Player,
	pub id: ClientId,
	pub data: PlayerData,
}
