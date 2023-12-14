use bevy::prelude::*;
use serde::{Serialize, Deserialize};

use crate::utils::{math::Velocity, nonfatal_error_systems};

pub const PHYSICS_ERROR: &'static str = "An error occurred polling physics";
pub const G_FORCE: f32 = 1.0;

#[derive(Component, Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct HasGravity;

#[derive(Component, Copy, Clone, Debug, Deref, DerefMut, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Mass(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Self(1.0)
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
			.add_systems(
				Update,
				nonfatal_error_systems!(PHYSICS_ERROR, anyhow::Error,
					gravity,
					movement
				),
			);
    }
}

pub fn movement(
	time: Res<Time>,
	mut query: Query<(&mut Transform, &Velocity)>,
) -> anyhow::Result<()> {
	for (mut transform, velocity) in query.iter_mut() {
		if velocity.translation.length_squared() > 0.0 {
			transform.translation += velocity.translation * time.delta_seconds();
		}
		
		if velocity.rotation.length_squared() > 0.0 {
			transform.rotate(velocity.rotation * time.delta_seconds());
		}
	}
	
	Ok(())
}

pub fn gravity(
	time: Res<Time>,
	mut query: Query<(&mut Velocity, &Mass), With<HasGravity>>,
) -> anyhow::Result<()> {
	for (mut velocity, &mass) in query.iter_mut() {
		if velocity.translation.y >= 55.0 {
			continue
		}
		
		velocity.translation.y += (G_FORCE / *mass) * time.delta_seconds();
	}
	
	Ok(())
}
