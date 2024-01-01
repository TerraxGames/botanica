//! TODO: change name to source.rs

use std::fmt;
use std::fmt::Formatter;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::networking::protocol::ClientId;
use crate::world;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
/// The executor of an action.
pub enum Source {
	Player(ClientId, Option<world::WorldId>),
	/// An action done by the system (the game/server).
	System,
	/// An unknown/anonymous executor.<br>
	/// This should be used sparingly.
	Unknown,
}

// mainly for chat message sources
impl fmt::Display for Source {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Player(player_id, world_id) => {
				f.write_fmt(format_args!("%p{{{}}}", player_id.0))?;
				if let Some(world_id) = world_id {
					f.write_fmt(format_args!(" [in %w{{{}}}]", world_id.0))
				} else {
					f.write_fmt(format_args!(""))
				}
			},
			Self::Unknown => f.write_fmt(format_args!("(anonymous)")),
			_ => f.write_str(""),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub enum Target {
	Player(ClientId),
	Players(Vec<ClientId>),
	/// The entire world.
	World,
	/// The entire server.
	All,
}
