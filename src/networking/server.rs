use std::collections::HashMap;
use std::net::UdpSocket;
use bevy::ecs::component::ComponentId;
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use serde::{Deserialize, Serialize};
use rand::Rng;
use rand::rngs::OsRng;
use renet::{RenetConnectionConfig, RenetServer, ServerEvent};
use crate::{env, GameState, mut_component_for_entity, Username, util};
use crate::networking::{Ping, protocol, strip_formatting};
use crate::networking::protocol::{ChatMessageBundle, ChatMessageContent, ClientId, ClientMessage, ClientMessageBundle, ClientResponse, ClientResponseBundle, PlayerData};
use crate::player::{Source, Target};
use crate::world::{WorldId, Worlds};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_enter_system(
				GameState::ServerLoading,
				setup
					.run_if(env::is_server)
			)
			.add_system(
				server
					.run_in_state(GameState::ServerLoaded)
					.run_if(env::is_server)
			)
			.add_system(client_message)
			.add_system(client_response);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone)]
pub struct Players(pub HashMap<u64, (PlayerData, Option<WorldId>, Option<&'static Entity>)>);

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct ServerPing(pub Ping);

fn setup(
	server_config: Res<ServerConfig>,
	mut commands: Commands,
	mut time: ResMut<Time>,
) {
	let connection_config = RenetConnectionConfig::default();
	
	time.update();
	
	let address = format!("{}:{}", server_config.address.0, server_config.port.0);
	
	let mut rng = OsRng::default();
	let private_key = &mut [0u8; 32];
	rng.fill(private_key);
	
	let server_config = renet::ServerConfig {
		max_clients: server_config.max_clients,
		protocol_id: protocol::PROTOCOL_ID,
		public_addr: address.parse().expect(&format!("Failed to parse address \"{}\"", address)),
		private_key: *private_key,
	};
	
	let socket = UdpSocket::bind(&address).expect(&format!("Failed to bind to address \"{}\"", address));
	let current_time = time.time_since_startup();
	
	let server = RenetServer::new(current_time, server_config, connection_config, socket).expect("Failed to create server");
	
	commands.insert_resource(server);
	commands.init_resource::<Players>();
	commands.insert_resource(NextState(GameState::ServerLoaded));
}

fn server(
	mut server: ResMut<RenetServer>,
	mut time: ResMut<Time>,
	mut commands: Commands,
	mut players: ResMut<Players>,
) {
	time.update();
	
	let current_time = time.time_since_startup();
	
	server.update(current_time).expect("Failed to update server");
	
	while let Some(event) = server.get_event() {
		match event {
			ServerEvent::ClientConnected(id, user_data) => {
				let username = Username::from_user_data(&*user_data);
				players.0.insert(id, (PlayerData { username: username.clone() }, None, None));
				println!("Player {} (ID {:x}) joined", username, id);
			}
			ServerEvent::ClientDisconnected(id) => {
				let (player, _, _) = players.0.remove(&id).unwrap();
				println!("Player {} (ID {:x}) disconnected", player.username, id);
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
							commands.spawn_bundle(ClientMessageBundle { id: ClientId(client_id), message });
						}
						protocol::Message::ClientResponse(response) => {
							commands.spawn_bundle(ClientResponseBundle { id: ClientId(client_id), response });
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
	mut worlds: ResMut<Worlds>,
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
			ClientMessage::EnterWorldRequest(world_name) => {
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
	mut world: ResMut<World>,
	mut time: ResMut<Time>,
	mut worlds: ResMut<Worlds>,
	mut players: ResMut<Players>,
) {
	time.update();
	
	for (client_id, response) in message_query.iter() {
		match response {
			ClientResponse::PingAck { timestamp } => {
				let timestamp = time.time_since_startup().as_millis();
				let (_, _, entity) = players.0.get(&client_id.0).unwrap();
				let entity = entity.unwrap();
				// SAFE: only mutable reference
				let ping = unsafe { mut_component_for_entity::<ServerPing>(entity, &world) }.unwrap();
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
			server.send_message(id, 0, util::serialize(&message).unwrap());
		}
		Target::Players(players) => {
			for id in players {
				server.send_message(id, 0, util::serialize(&message).unwrap());
			}
		}
		Target::World => {
		}
		Target::All => {
			server.broadcast_message(0, util::serialize(&message).unwrap());
		}
	}
}
