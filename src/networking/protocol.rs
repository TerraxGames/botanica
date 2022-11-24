use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use crate::{TilePos, Username};
use crate::player::{Source, Target};

pub const PROTOCOL_ID: u64 = 0x460709E200F3661E;
pub const PROTOCOL_VER: u32 = 0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// A type of message.
pub enum Message {
	ServerMessage(ServerMessage),
	ServerResponse(ServerResponse),
	ClientMessage(ClientMessage),
	ClientResponse(ClientResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
/// A message that the server sends to a client (or to all/some clients).
pub enum ServerMessage {
	Ping {
		/// The time the ping was sent.
		timestamp: u128,
	},
	/// Signals a disconnection is about to happen and gives the reason.
	Disconnect(DisconnectReason),
	PlayerJoin(ClientId, PlayerData),
	PlayerLeave(ClientId),
	/// A request from a player to change their display name (nickname).
	PlayerNick(ClientId, String),
	ChatMessage(ChatMessageBundle),
	PlayerPosition(ClientId, TilePos),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
/// A message that the server sends to a client in response to a message from that client.
pub enum ServerResponse {
	Query {
		/// The protocol version.
		protocol_ver: u64,
		/// The game version string.
		version: String,
		motd: String,
	},
	PingAck {
		/// The time the ping was sent.
		timestamp: u128,
	},
	JoinDeny(DisconnectReason),
	JoinAccept,
	EnterWorldDeny(WorldDenyReason),
	EnterWorldAccept,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
/// A message that the client sends to the server.
pub enum ClientMessage {
	Query,
	Ping {
		/// The time the ping was sent.
		timestamp: u128,
	},
	JoinRequest,
	ChatMessage(Target, String),
	EnterWorldRequest(String),
	PlayerPosition(TilePos),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Bundle)]
pub struct ClientMessageBundle {
	pub id: ClientId,
	pub message: ClientMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
/// A message that the client sends to the server in response to a message the server sent.
pub enum ClientResponse {
	QueryAck,
	PingAck {
		/// The time the ping was sent.
		timestamp: u128,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Bundle)]
pub struct ClientResponseBundle {
	pub id: ClientId,
	pub response: ClientResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub enum DisconnectReason {
	ServerFull(String),
	Kicked(String),
	Banned(String),
	Shutdown(String),
	Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub enum WorldDenyReason {
	WorldFull(String),
	Kicked(String),
	Banned(String),
	Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct PlayerData {
	pub username: Username,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct ClientId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct ChatMessageContent(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Bundle)]
pub struct ChatMessageBundle {
	pub content: ChatMessageContent,
	pub source: Source,
	pub target: Target,
}
