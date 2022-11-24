use bevy::prelude::*;
use crate::creature::Creature;
use crate::networking::protocol::ClientId;
use crate::Position;

#[derive(Debug, Default, Clone, Component)]
pub struct Player;

#[derive(Clone, Bundle)]
pub struct PlayerBundle {
	creature: Creature,
	player: Player,
	id: ClientId,
	pos: Position,
}
