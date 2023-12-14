use bevy::prelude::*;

#[derive(Component, Debug, Default, Copy, Clone, PartialEq, Deref, DerefMut)]
pub struct Velocity(Transform);
