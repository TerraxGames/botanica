use std::fmt;
use std::fmt::Formatter;
use std::net::AddrParseError;
use std::time::{Duration, SystemTime};

use bevy::prelude::*;
use renet::transport::NETCODE_USER_DATA_BYTES;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod protocol;
pub mod debug;
pub mod error;

pub const USERNAME_BYTES: usize = 32;

/// The reason for being kicked to the disconnect screen.
#[derive(Debug, Error, Resource)]
pub enum DisconnectReason {
	#[error("Transport layer disconnection: {0}")]
	Transport(renet::transport::NetcodeDisconnectReason),
	#[error("Client disconnection: {0}")]
	Client(renet::DisconnectReason),
	#[error("Malformed IP address: {0}")]
	AddrParseError(AddrParseError),
	#[error("Disconnected by server: {0}")]
	Disconnected(protocol::DisconnectReason),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component, Resource)]
pub struct Ping(pub u128);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component, Resource)]
pub struct Username(pub String);

impl fmt::Display for Username {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("{}", self.0))
	}
}

impl Username {
	pub fn to_user_data(&self) -> [u8; NETCODE_USER_DATA_BYTES] {
		let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
		
		if self.0.len() > USERNAME_BYTES {
			panic!("Username too big (maximum is length {}, found {})", USERNAME_BYTES, self.0.len()); // fixme: don't panic please?
		}
		
		user_data[0] = self.0.len() as u8;
		user_data[1..self.0.len() + 1].copy_from_slice(self.0.as_bytes());
		
		user_data
	}
	
	pub fn from_user_data(user_data: &[u8; NETCODE_USER_DATA_BYTES]) -> Self {
		let len = user_data[0] as usize;
		
		let mut buf = Vec::new();
		buf.extend_from_slice(&user_data[1..len + 1]);
		
		Self(String::from_utf8(buf).unwrap())
	}
}

/// Get the time since the Unix epoch.
#[inline]
pub fn time_since_epoch() -> Duration {
	SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap()
}

pub(crate) mod stats {
	use std::collections::HashMap;
	
	
	use bevy::ecs::system::Resource;

use crate::networking::protocol::ClientId;
	
	/// A player's network statistics
	#[derive(Debug, Default, Clone)]
	pub struct PlayerNetStat {
		/// The player's latency in milliseconds
		pub ping: u128,
	}
	
	#[derive(Debug, Default, Clone, Resource)]
	pub struct PlayerNetStats(HashMap<ClientId, PlayerNetStat>);
	
	impl PlayerNetStats {
		/// Retrieves an optional network statistics struct that corresponds to the client ID.
		pub fn get(&self, client_id: ClientId) -> Option<&PlayerNetStat> {
			self.0.get(&client_id)
		}
		
		/// Retrieves (or creates if non-existent) a mutable network statistics struct that corresponds to the client ID.
		pub fn get_mut(&mut self, client_id: ClientId) -> &mut PlayerNetStat {
			self.0
				.entry(client_id)
				.or_default()
		}
		
		/// Removes the network statistics struct from the hash map. Returns Some with the removed network statistics struct if successful, None if non-existent.
		pub fn remove(&mut self, client_id: ClientId) -> Option<PlayerNetStat> {
			self.0.remove(&client_id)
		}
	}
}
