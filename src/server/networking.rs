use crate::utils::BevyHashMap;
use std::net::{SocketAddr, UdpSocket};

use bevy::prelude::*;
use bevy_renet::RenetServerPlugin;
use bevy_renet::transport::NetcodeServerPlugin;
use renet::{Bytes, ConnectionConfig, DefaultChannel, RenetServer, ServerEvent};
use renet::transport::{NetcodeServerTransport, ServerAuthentication};
use serde::{Deserialize, Serialize};

use crate::creature::player::{Player, PlayerBundle};
use crate::raw_id::tile::RawTileIds;
use crate::{env, GameState, Username, utils, VERSION_STRING};
use crate::networking::{protocol, time_since_epoch};
use crate::networking::error::{NETWORK_ERROR_MESSAGE, NetworkError};
use crate::networking::protocol::{ChatMessageBundle, ChatMessageContent, ClientId, ClientMessage, ClientMessageBundle, ClientResponse, PlayerData, PROTOCOL_VER, Packet, WorldDenyReason, ServerMessage, ServerResponse};
use crate::networking::stats::PlayerNetStats;
use crate::player::{Source, Target};
use crate::utils::{nonfatal_error_systems, strip_formatting};
use crate::world::{ServerGameWorlds, WorldId};

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugins(RenetServerPlugin)
			.add_plugins(NetcodeServerPlugin)
			.init_resource::<Players>()
			.init_resource::<ServerGameWorlds>()
			.init_resource::<ServerConfig>()
			.add_systems(
				OnEnter(GameState::ServerLoading),
				setup
					.run_if(env::is_server)
			)
			.add_systems(
				Update,
				(
					nonfatal_error_systems!(NETWORK_ERROR_MESSAGE, NetworkError, server, receive_message),
				)
					.run_if(in_state(GameState::ServerLoaded))
					.run_if(env::is_server)
			);
	}
}

macro_rules! send_message {
    ($server:expr, $client_id:expr, $channel_id:expr, $message:expr) => {
		{
			$server.send_message($client_id.into(), $channel_id, TryInto::<renet::Bytes>::try_into($message)?);
		}
	};
}

pub(super) use send_message;

macro_rules! broadcast_message {
    ($server:expr, $channel_id:expr, $message:expr) => {
		{
			$server.broadcast_message($channel_id, TryInto::<Bytes>::try_into($message)?);
		}
	};
}

pub(super) use broadcast_message;

macro_rules! broadcast_world_message {
    ($server:expr, $world:expr, $player_client_id_query:expr, $channel_id:expr, $message:expr) => {
		{
			for player in $world.players.iter() {
				let client_id = $player_client_id_query.get(*player);
				if let Ok(&client_id) = client_id {
					$server.send_message(client_id.into(), $channel_id, TryInto::<Bytes>::try_into($message)?);
				}
			}
		}
	};
}

pub(super) use broadcast_world_message;

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

#[derive(Debug, Deref, Default, Clone, Resource)]
pub struct Players(pub BevyHashMap<ClientId, Entity>);

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
	transport: Res<NetcodeServerTransport>,
	mut commands: Commands,
	mut worlds: ResMut<ServerGameWorlds>,
	mut players: ResMut<Players>,
	player_data_query: Query<&PlayerData, With<Player>>,
	player_world_id_query: Query<&WorldId, With<Player>>,
	mut ev_server: EventReader<ServerEvent>,
	raw_tile_ids: Res<RawTileIds>,
) -> Result<(), NetworkError> {
	for event in ev_server.iter() {
		match event {
			ServerEvent::ClientConnected { client_id: id } => {
				if let Some(user_data) = transport.user_data(*id) {
					let username = Username::from_user_data(&user_data);
					let player_bundle = PlayerBundle {
						id: ClientId(*id),
						data: PlayerData { username: username.clone() },
						..default()
					};
					players.0.insert(ClientId(*id), commands.spawn(player_bundle).id());
					println!("Player {} (ID {:X}) connected", username, id);
					send_message!(server, *id, DefaultChannel::ReliableOrdered, ServerMessage::RawTileIds(raw_tile_ids.clone()));
				} else {
					println!("Player (ID {:X}) attempted to join, but no user data was sent!", id);
					send_message!(server, *id, DefaultChannel::ReliableOrdered, ServerMessage::Disconnect(protocol::DisconnectReason::EmptyUserdata));
					server.disconnect(*id);
				}
			},
			ServerEvent::ClientDisconnected { client_id: id, reason } => {
				let player = players.0.remove(&ClientId(*id));
				if let Some(player_entity) = player {
					let player_data = player_data_query.get(player_entity)?;
					println!("Player {} (ID {:X}) disconnected: {}", player_data.username, id, reason);
					let world = worlds.get_world_mut(player_world_id_query.get(player_entity)?.as_str());
					if let Some(world) = world {
						let index = world.players.iter().position(|x| *x == player_entity);
						if let Some(index) = index {
							world.players.remove(index);
						}
					}
					
					commands.entity(player_entity).despawn_recursive();
				}
			},
		}
	}
	
	for client_id in server.clients_id() {
		for channel_id in 0..=2 {
			while let Some(buf) = server.receive_message(client_id, channel_id) {
				let packet = utils::deserialize_be::<Packet>(&buf);
				
				// allow processing all packets regardless of error
				if let Ok(packet) = packet {
					match packet {
						Packet::ClientMessage(_) => {
							commands.spawn(ClientMessageBundle { id: ClientId(client_id), packet });
						},
						Packet::ClientResponse(_) => {
							commands.spawn(ClientMessageBundle { id: ClientId(client_id), packet });
						},
						_ => warn!("Received incorrect/server message from client ({}): {:?}", client_id, packet),
					}
				} else if let Err(err) = packet {
					eprintln!("{err}");
				}
			}
		}
	}
	
	Ok(())
}

fn receive_message(
	message_query: Query<(Entity, &ClientId, &Packet)>,
	mut server: ResMut<RenetServer>,
	mut worlds: ResMut<ServerGameWorlds>,
	players: Res<Players>,
	player_query: Query<(&PlayerData, Option<&WorldId>), With<Player>>,
	player_world_id_query: Query<&WorldId, With<Player>>,
	player_client_id_query: Query<&ClientId, With<Player>>,
	raw_tile_ids: Res<RawTileIds>,
	mut player_stats: ResMut<PlayerNetStats>,
	mut commands: Commands,
) -> Result<(), NetworkError> {
	for (entity, client_id, packet) in message_query.iter() {
		commands.entity(entity).despawn();
		if let Some(player_entity) = players.0.get(client_id) {
			let player = player_query.get(*player_entity)?;
			let player_data = player.0;
			println!("({}:{:X}): {:?}", player_data.username, client_id.0, packet);
		} else {
			continue
		}
		if let Packet::ClientMessage(message) = packet {
			match message {
				ClientMessage::Ping { timestamp } => {
					send_message!(server, client_id, DefaultChannel::ReliableUnordered, ServerResponse::PingAck { timestamp: *timestamp });
					
					let now = time_since_epoch().as_millis();
					let stats = player_stats.get_mut(*client_id);
					stats.ping = now - timestamp;
				},
				ClientMessage::JoinRequest { protocol_ver } => {
					if *protocol_ver != PROTOCOL_VER {
						send_message!(server, client_id, DefaultChannel::ReliableOrdered, ServerResponse::JoinDeny(protocol::DisconnectReason::ProtocolReject { required_protocol_ver: PROTOCOL_VER, required_version_string: VERSION_STRING.to_string() }));
						server.disconnect(client_id.0);
					} else {
						send_message!(server, client_id, DefaultChannel::ReliableOrdered, ServerResponse::JoinAccept);
					}
				},
				ClientMessage::PlayerPosition(position) => {
					let player_entity = players.get(client_id);
					if player_entity.is_none() {
						continue
					}
					
					let world = worlds.get_world(player_world_id_query.get(player_entity.unwrap().clone())?.as_str());
					if world.is_none() {
						continue
					}
					
					broadcast_world_message!(server, world.unwrap(), player_client_id_query, DefaultChannel::Unreliable, ServerMessage::PlayerPosition(*client_id, *position));
				},
				ClientMessage::EnterWorldRequest(world_name) => {
					let world_name = utils::sanitize::sanitize_alphanumeric_dash(world_name);
					if world_name.len() == 0 {
						send_message!(server, client_id, DefaultChannel::ReliableOrdered, ServerResponse::EnterWorldDeny(WorldDenyReason::InvalidWorldName));
						continue
					}
					
					let world = worlds.get_or_gen_world_mut(world_name.as_str(), &*raw_tile_ids)?;
					
					let player_entity = players.get(client_id);
					if player_entity.is_none() {
						send_message!(server, client_id, DefaultChannel::ReliableOrdered, ServerMessage::Disconnect(protocol::DisconnectReason::PlayerNonexistent));
						server.disconnect(client_id.0);
						continue
					}
					let player = player_query.get(*player_entity.unwrap())?;
					
					// check if this player is banned & kick 'em if they are
					if world.bans.contains_key(&player.0.username) {
						let ban = world.bans.get(&player.0.username).unwrap();
						send_message!(server, client_id, DefaultChannel::ReliableOrdered, ServerResponse::EnterWorldDeny(WorldDenyReason::Banned(ban.reason().to_string(), ban.until())));
						// todo: unload world if no other players are present
						continue
					}
					
					// add player to world
					if !world.players.contains(player_entity.unwrap()) {
						world.players.push(player_entity.unwrap().clone());
					}
					
					commands.entity(player_entity.unwrap().clone()).insert(world.id.clone());
					
					send_message!(server, client_id, DefaultChannel::ReliableOrdered, ServerResponse::EnterWorldAccept(world.id.clone()));
					send_message!(server, client_id, DefaultChannel::ReliableOrdered, ServerMessage::WorldTiles(world.tiles.clone()));
					broadcast_world_message!(server, world, player_client_id_query, DefaultChannel::ReliableOrdered, ServerMessage::PlayerJoin(*client_id, player.0.clone(), world.spawnpoint));
				},
				ClientMessage::ChatMessage(target, content) => {
					let player = player_query.get(*players.0.get(client_id).unwrap())?;
					let chat_message = ChatMessageBundle {
						content: ChatMessageContent(content.into()),
						source: Source::Player(*client_id, player.1.cloned()),
						target: target.clone(),
					};
					send_message!(server, client_id, DefaultChannel::ReliableOrdered, ServerMessage::ChatMessage(chat_message)); // todo: broadcast chat message to players in target range
				},
			}
		} else if let Packet::ClientResponse(response) = packet {
			match response {
				ClientResponse::PingAck { timestamp } => {
					let now = time_since_epoch().as_millis();
					player_stats.get_mut(*client_id).ping = now - timestamp;
				},
			}
		}
	}
	
	Ok(())
}

pub fn send_chat(
	mut server: ResMut<RenetServer>,
	source: Source,
	target: Target,
	message: String,
) -> Result<(), NetworkError> { // todo: mute
	if target == Target::All { // todo: private message spying
		println!("{}", strip_formatting(&format!("{} {}", source, message)));
	}
	let message = ServerMessage::ChatMessage(ChatMessageBundle {
		content: ChatMessageContent(message),
		source,
		target: target.clone(),
	});
	match target {
		Target::Player(id) => {
			send_message!(server, id.0, DefaultChannel::ReliableOrdered, message);
		}
		Target::Players(players) => {
			for id in players {
				send_message!(server, id.0, DefaultChannel::ReliableOrdered, message.clone());
			}
		}
		Target::World => { // todo: send chat message to world
		}
		Target::All => {
			broadcast_message!(server, DefaultChannel::ReliableOrdered, message);
		}
	}
	
	Ok(())
}
