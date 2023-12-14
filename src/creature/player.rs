use bevy::prelude::*;

use crate::networking::Username;
use crate::networking::protocol::{ClientId, PlayerData};
use crate::utils::math::Velocity;

use super::CreatureBundle;

#[derive(Component, Debug, Default, Clone)]
pub struct Player; // TODO: make the eyes and arms their own sprites and allow them to rotate and move freely since we're only in 16x16. also, allow changing eye color.

#[derive(Event)]
pub struct SpawnPlayerEvent { // TODO: handle player spawning and collision
	pub transform: Transform,
	pub id: ClientId,
	pub data: PlayerData,
}

/// ### System
/// "Decorates" the player (handles its eyes & arms).
pub fn player_decoration(
	time: Res<Time>,
	player_query: Query<&Velocity, With<Player>>, // parent
	mut eyes_query: Query<(&Parent, &mut Transform), With<PlayerEyes>>, // child
	left_arm_query: Query<(&Parent, &Transform), With<PlayerLeftArm>>, // child
	right_arm_query: Query<(&Parent, &Transform), With<PlayerRightArm>>, // child
) -> anyhow::Result<()> {
	for (parent, mut eyes_transform) in eyes_query.iter_mut() {
		let player_velocity = player_query.get(parent.get())?;
		if eyes_transform.translation.x >= 0.25 || eyes_transform.translation.x <= -0.25 { // continue if maximum eye look reached
			continue
		}
		
		if player_velocity.translation.x > 0.05 {
			eyes_transform.translation.x += time.delta_seconds();
		} else if player_velocity.translation.x < 0.05 {
			eyes_transform.translation.x -= time.delta_seconds();
		}
	}
	
	Ok(())
}

#[derive(Component, Debug, Clone, Default)]
pub struct PlayerEyes;

#[derive(Bundle, Clone, Default)]
pub struct PlayerEyesBundle {
	pub eyes: PlayerEyes,
	pub sprite: SpriteBundle,
}

#[derive(Component, Debug, Clone, Default)]
pub struct PlayerLeftArm;

#[derive(Bundle, Clone, Default)]
pub struct PlayerLeftArmBundle {
	pub left_arm: PlayerLeftArm,
	pub sprite: SpriteBundle,
}

#[derive(Component, Debug, Clone, Default)]
pub struct PlayerRightArm;

#[derive(Bundle, Clone, Default)]
pub struct PlayerRightArmBundle {
	pub left_arm: PlayerRightArm,
	pub sprite: SpriteBundle,
}

/// A bundled player entity. Don't forget to add the eyes and arms as children!
#[derive(Bundle, Clone)]
pub struct PlayerBundle {
	pub creature: CreatureBundle,
	pub player: Player,
	pub id: ClientId,
	pub data: PlayerData,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
			creature: Default::default(),
			player: Default::default(),
			id: ClientId(0),
			data: PlayerData { username: Username("Player".to_string()) },
		}
    }
}
