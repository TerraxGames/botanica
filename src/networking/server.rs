use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};

use bevy::prelude::*;
use bevy_renet::RenetServerPlugin;
use bevy_renet::transport::NetcodeServerPlugin;
use renet::{ConnectionConfig, RenetServer, ServerEvent};
use renet::transport::{NetcodeServerTransport, ServerAuthentication};
use serde::{Deserialize, Serialize};

use crate::{env, GameState, Username, util};
use crate::networking::{Ping, protocol, time_since_epoch};
use crate::networking::protocol::{ChatMessageBundle, ChatMessageContent, ClientId, ClientMessage, ClientMessageBundle, ClientResponse, ClientResponseBundle, PlayerData};
use crate::player::{Source, Target};
use crate::util::strip_formatting;
use crate::world::{GameWorlds, WorldId};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugins(RenetServerPlugin)
			.add_plugins(NetcodeServerPlugin)
			.init_resource::<Players>()
			.init_resource::<GameWorlds>()
			.init_resource::<ServerConfig>()
			.add_systems(
				OnEnter(GameState::ServerLoading),
				setup
					.run_if(env::is_server)
			)
			.add_systems(
				Update,
				(
					server,
					client_message,
					client_response,
				)
					.run_if(in_state(GameState::ServerLoaded))
					.run_if(env::is_server)
			);
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerAddress(pub String);

impl Default for ServerAddress {
	fn default() -> Self {
		Self("127.0.0.1".to_owned())
	}
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ServerPort(pub u16);

impl Default for ServerPort {
	fn default() -> Self {
		Self(44738)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct ServerConfig {
	pub address: ServerAddress,
	pub port: ServerPort,
	pub max_clients: usize,
}

impl Default for ServerConfig {
	fn default() -> Self {
		Self {
			address: Default::default(),
			port: Default::default(),
			max_clients: 16,
		}
	}
}

#[derive(Debug, Default, Clone, Resource)]
pub struct Players(pub HashMap<u64, (PlayerData, Option<WorldId>, Option<&'static Entity>)>);

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct ServerPing(pub Ping);

fn setup(
	server_config: Res<ServerConfig>,
	mut commands: Commands,
	mut next_state: ResMut<NextState<GameState>>,
) {
	let connection_config = ConnectionConfig::default();
	
	let address = format!("{}:{}", server_config.address.0, server_config.port.0);
	
	let authentication = ServerAuthentication::Unsecure;
	
	let server_config = renet::transport::ServerConfig {
		max_clients: server_config.max_clients,
		protocol_id: protocol::PROTOCOL_ID,
		public_addr: address.parse::<SocketAddr>().expect(&format!("Failed to parse address \"{}\"", address)),
		authentication,
	};
	
	let socket = UdpSocket::bind(&address).expect(&format!("Failed to bind to address \"{}\"", address));
	let current_time = time_since_epoch();
	
	let server = RenetServer::new(connection_config);
	let transport = NetcodeServerTransport::new(current_time, server_config, socket).expect("Failed to create transport");
	
	commands.insert_resource(server);
	commands.insert_resource(transport);
	println!("Loaded server! Listening @ {}", address);
	next_state.set(GameState::ServerLoaded);
}

fn server(
	mut server: ResMut<RenetServer>,
	mut transport: ResMut<NetcodeServerTransport>,
	mut commands: Commands,
	mut players: ResMut<Players>,
) {
	while let Some(event) = server.get_event() {
		match event {
			ServerEvent::ClientConnected { client_id: id } => {
				if let Some(user_data) = transport.user_data(id) {
					let username = Username::from_user_data(&user_data);
					players.0.insert(id, (PlayerData { username: username.clone() }, None, None));
					println!("Player {} (ID {:x}) joined", username, id);
				} else {
					println!("Player (ID {:x}) attempted to join, but no user data was sent!", id);
					let message = util::serialize(&protocol::Message::ServerResponse(protocol::ServerResponse::JoinDeny(protocol::DisconnectReason::EmptyUserdata))).expect("Failed to serialize");
					server.send_message(id, 0, message);
					server.disconnect(id);
				}
			}
			ServerEvent::ClientDisconnected { client_id: id, reason } => {
				let (player, _, _) = players.0.remove(&id).unwrap();
				println!("Player {} (ID {:x}) disconnected: {}", player.username, id, reason);
			}
		}
	}
	
	for client_id in server.clients_id() {
		for channel_id in 0..=2 {
			while let Some(buf) = server.receive_message(client_id, channel_id) {
				let message = util::deserialize::<protocol::Message>(&buf);
				if let Ok(message) = message {
					match message {
						protocol::Message::ClientMessage(message) => {
							commands.spawn(ClientMessageBundle { id: ClientId(client_id), message });
						}
						protocol::Message::ClientResponse(response) => {
							commands.spawn(ClientResponseBundle { id: ClientId(client_id), response });
						}
						_ => warn!("Received server message from client ({}): {:?}", client_id, message),
					}
				}
			}
		}
	}
}

fn client_message(
	message_query: Query<(&ClientId, &ClientMessage)>,
	mut server: ResMut<RenetServer>,
	mut worlds: ResMut<GameWorlds>,
	mut players: ResMut<Players>,
) {
	for (client_id, message) in message_query.iter() {
		match message {
			ClientMessage::Ping { timestamp } => {
				let message = util::serialize(&protocol::Message::ServerResponse(protocol::ServerResponse::PingAck { timestamp: *timestamp })).unwrap();
				server.send_message(client_id.0, 0, message);
			}
			ClientMessage::PlayerPosition(position) => {
				let message = util::serialize(&protocol::Message::ServerMessage(protocol::ServerMessage::PlayerPosition(*client_id, *position))).unwrap();
				server.send_message(client_id.0, 1, message);
			}
			ClientMessage::EnterWorldRequest(_world_name) => {
				let response = protocol::ServerResponse::EnterWorldAccept; // todo: deny joining worlds?
				let message = util::serialize(&protocol::Message::ServerResponse(response)).unwrap();
				server.send_message(client_id.0, 0, message);
			}
			ClientMessage::ChatMessage(target, content) => {
				let chat_message = ChatMessageBundle {
					content: ChatMessageContent(content.into()),
					source: Source::Player(*client_id, players.0.get(&client_id.0).unwrap().1),
					target: target.clone(),
				};
				let message = util::serialize(&protocol::Message::ServerMessage(protocol::ServerMessage::ChatMessage(chat_message))).unwrap();
				server.send_message(client_id.0, 0, message);
			}
			_ => {}
		}
	}
}

fn client_response(
	message_query: Query<(&ClientId, &ClientResponse)>,
	mut server: ResMut<RenetServer>,
	mut commands: Commands,
	mut worlds: ResMut<GameWorlds>,
	mut players: ResMut<Players>,
) {
	for (client_id, response) in message_query.iter() {
		match response {
			ClientResponse::PingAck { timestamp } => {
				let timestamp_now = time_since_epoch().as_millis();
				let (_, _, entity) = players.0.get(&client_id.0).unwrap();
				let entity = entity.unwrap();
				commands.entity(*entity).log_components();
				// todo: finish client response
			}
			_ => {}
		}
	}
}

pub fn send_chat(
	mut server: ResMut<RenetServer>,
	source: Source,
	target: Target,
	message: String,
) { // todo: mute
	if target == Target::All { // todo: private message spying
		println!("{}", strip_formatting(format!("{} {}", source, message)));
	}
	let message = protocol::Message::ServerMessage(protocol::ServerMessage::ChatMessage(ChatMessageBundle {
		content: ChatMessageContent(message),
		source,
		target: target.clone(),
	}));
	match target {
		Target::Player(id) => {
			server.send_message(id.0, 0, util::serialize(&message).unwrap());
		}
		Target::Players(players) => {
			for id in players {
				server.send_message(id.0, 0, util::serialize(&message).unwrap());
			}
		}
		Target::World => { // todo: send chat message to world
		}
		Target::All => {
			server.broadcast_message(0, util::serialize(&message).unwrap());
		}
	}
}
