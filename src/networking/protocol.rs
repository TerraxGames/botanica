use crate::utils::BevyHashMap;
use std::fmt;
use std::fmt::Formatter;

use bevy::prelude::*;
use renet::Bytes;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::raw_id::tile::RawTileIds;
use crate::tile::WorldTile;
use crate::world::{WorldBanUntil, WorldId};
use crate::{TilePos, Username, Position};
use crate::networking::error::NetworkError;
use crate::player::{Source, Target};

pub const PROTOCOL_ID: u64 = 0x460709E200F3661E;
pub const PROTOCOL_VER: ProtocolVersion = ProtocolVersion(0);
pub const CLIENT_TIMEOUT: u64 = 5000;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion(pub u32);

impl fmt::Display for ProtocolVersion {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Component)]
/// A type of message/response.
pub enum Packet {
	ServerMessage(ServerMessage),
	ServerResponse(ServerResponse),
	ClientMessage(ClientMessage),
	ClientResponse(ClientResponse),
}

macro_rules! impl_try_into_bytes {
    ($t:ident) => {
		impl TryInto<Bytes> for $t {
			type Error = NetworkError;
			
			fn try_into(self) -> Result<Bytes, Self::Error> {
				Ok($crate::utils::serialize_be(&Packet::$t(self))?.into())
			}
		}
	};
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// A message that the server sends to a client (or to all/some clients).
pub enum ServerMessage {
	Ping {
		/// The time the ping was sent.
		timestamp: u128,
	},
	/// Signals a disconnection is about to happen and gives the reason.
	Disconnect(DisconnectReason),
	PlayerJoin(ClientId, PlayerData, Position),
	PlayerLeave(ClientId),
	/// A request from a player to change their display name (nickname).
	PlayerNick(ClientId, String),
	ChatMessage(ChatMessageBundle),
	PlayerPosition(ClientId, Position),
	/// Syncs the server's [RawTileIds] with the client.
	RawTileIds(RawTileIds),
	WorldTiles(BevyHashMap<TilePos, WorldTile>),
}

impl_try_into_bytes!(ServerMessage);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// A message that the server sends to a client in response to a message from that client.
pub enum ServerResponse {
	JoinDeny(DisconnectReason),
	JoinAccept,
	// Query { // todo: move to REST API
	// 	/// The protocol version.
	// 	protocol_ver: ProtocolVersion,
	// 	/// The game version string.
	// 	version: String,
	// 	/// The message of the day.
	// 	motd: String,
	// },
	PingAck {
		/// The time the ping was sent.
		timestamp: u128,
	},
	EnterWorldDeny(WorldDenyReason),
	EnterWorldAccept(WorldId),
}

impl_try_into_bytes!(ServerResponse);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// A message that the client sends to the server.
pub enum ClientMessage {
	JoinRequest {
		/// The version of the protocol.
		protocol_ver: ProtocolVersion,
	},
	Ping {
		/// The time the ping was sent.
		timestamp: u128,
	},
	ChatMessage(Target, String),
	EnterWorldRequest(String),
	PlayerPosition(Position),
}

impl_try_into_bytes!(ClientMessage);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// A message that the client sends to the server in response to a message the server sent.
pub enum ClientResponse {
	PingAck {
		/// The time the ping was sent.
		timestamp: u128,
	},
}

impl_try_into_bytes!(ClientResponse);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Bundle)]
pub struct ClientMessageBundle {
	pub id: ClientId,
	pub packet: Packet,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component, Error)]
pub enum DisconnectReason {
	#[error("Mismatched protocol version; required: \"{required_version_string}\" @ {required_protocol_ver}")]
	ProtocolReject {
		/// The protocol version that is required
		required_protocol_ver: ProtocolVersion,
		/// The required version string
		required_version_string: String,
	},
	#[error("The userdata field is empty!")]
	EmptyUserdata,
	#[error("The player's data is non-existent!")]
	PlayerNonexistent,
	#[error("Server is full: {0}")]
	ServerFull(String),
	#[error("Kicked: {0}")]
	Kicked(String),
	#[error("Banned: {0}")]
	Banned(String),
	#[error("The server has shutdown: {0}")]
	Shutdown(String),
	#[error("{}", match .0 {
		Some(string) => string.to_string(),
		None => "Unknown".to_string(),
	})]
	Other(Option<String>),
}

/// The reason entry to a world has been denied.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub enum WorldDenyReason {
	WorldFull(String),
	Banned(String, WorldBanUntil),
	InvalidWorldName,
	Other(Option<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct PlayerData {
	pub username: Username,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct ClientId(pub u64);

impl Into<u64> for ClientId {
	fn into(self) -> u64 {
		self.0
	}
}

impl Into<u64> for &ClientId {
	fn into(self) -> u64 {
		self.0
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct ChatMessageContent(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Bundle)]
pub struct ChatMessageBundle {
	pub content: ChatMessageContent,
	pub source: Source,
	pub target: Target,
}
