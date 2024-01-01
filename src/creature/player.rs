use bevy::prelude::*;

use crate::networking::Username;
use crate::networking::protocol::{ClientId, PlayerData};
use crate::utils::asset::load_image;
use crate::utils::math::{Velocity, ToScale};
use crate::NAMESPACE;

use super::CreatureBundle;

#[derive(Component, Debug, Default, Clone)]
pub struct Player;

pub const PLAYER_Z: f32 = 2.0;
pub const DEFAULT_EYE_COLOR: Color = Color::rgb(0.0, 0.388235294118, 0.639215686274);
pub const SPAWN_PLAYER_EVENT_ERROR_MESSAGE: &'static str = "An error occurred while spawning a tile";
pub const PLAYER_DECORATION_ERROR_MESSAGE: &'static str = "An error occurred while decorating the player";

#[derive(Event)]
pub struct SpawnPlayerEvent { // TODO: handle player spawning and collision
	pub transform: Transform,
	pub id: ClientId,
	pub data: PlayerData,
}

pub fn spawn_player_event(
	mut spawn_player_event: EventReader<SpawnPlayerEvent>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) -> anyhow::Result<()> {
	for event in spawn_player_event.into_iter() {
		commands.spawn(
			PlayerBundle {
				id: event.id,
				data: event.data.clone(),
				creature: CreatureBundle {
					sprite: SpriteBundle {
						texture: load_image(&asset_server, format!("{NAMESPACE}/textures/creature/player.png")),
						transform: event.transform,
						..default()
					},
					..default()
				},
				..default()
			}
		)
			.with_children(|builder| {
				builder.spawn(
					PlayerEyesBundle {
						sprite: SpriteBundle {
							sprite: Sprite {
								color: DEFAULT_EYE_COLOR,
								..default()
							},
							texture: load_image(&asset_server, format!("{NAMESPACE}/textures/creature/player_eyes.png")),transform: event.transform.with_translation(Vec3::ZERO),
							..default()
						},
						..default()
					}
				);
				let arm_image_handle = load_image(&asset_server, format!("{NAMESPACE}/textures/creature/player_arm.png"));
				builder.spawn(
					PlayerLeftArmBundle {
						sprite: SpriteBundle {
							sprite: Sprite {
								flip_x: true,
								..default()
							},
							texture: arm_image_handle.clone(),
							transform: event.transform.with_translation(Vec3::new(0.0, 0.0, -1.0)),
							..default()
						},
						..default()
					}
				);
				builder.spawn(
					PlayerRightArmBundle {
						sprite: SpriteBundle {
							texture: arm_image_handle,
							transform: event.transform.with_translation(Vec3::new(0.0, 0.0, 1.0)),
							..default()
						},
						..default()
					}
				);
			});
	}
	
	Ok(())
}

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
	pub to_scale: ToScale,
	pub sprite: SpriteBundle,
}

#[derive(Component, Debug, Clone, Default)]
pub struct PlayerLeftArm;

#[derive(Bundle, Clone, Default)]
pub struct PlayerLeftArmBundle {
	pub left_arm: PlayerLeftArm,
	pub to_scale: ToScale,
	pub sprite: SpriteBundle,
}

#[derive(Component, Debug, Clone, Default)]
pub struct PlayerRightArm;

#[derive(Bundle, Clone, Default)]
pub struct PlayerRightArmBundle {
	pub left_arm: PlayerRightArm,
	pub to_scale: ToScale,
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
