use std::net::SocketAddr;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::render::texture::DEFAULT_IMAGE_HANDLE;
use bevy_renet::RenetClientPlugin;
use bevy_renet::transport::NetcodeClientPlugin;
use renet::{ConnectionConfig, DefaultChannel, RenetClient};
use renet::transport::{ClientAuthentication, NetcodeClientTransport};

use crate::asset::tile::TileDef;
use crate::networking;
use crate::raw_id::tile::RawTileIds;
use crate::registry::tile::TileRegistry;
use crate::registry::tile::settings::TileSalience;
use crate::tile::WorldTile;
use crate::utils::asset::load_image;
use crate::world::ClientGameWorld;
use crate::world::SetTileEvent;
use crate::world::TILE_EVENT_ERROR_MESSAGE;
use crate::world::TileEventError;
use crate::{env, GameState, ServerConnectAddress, utils};
use crate::networking::{DisconnectReason, Ping, protocol, time_since_epoch, Username};
use crate::networking::error::{NETWORK_ERROR_MESSAGE, NetworkError};
use crate::networking::protocol::{PlayerData, ServerMessage, ServerResponse};
use crate::player::Target;
use crate::utils::nonfatal_error_systems;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugins(RenetClientPlugin)
			.add_plugins(NetcodeClientPlugin)
			.init_resource::<ServerConnectAddress>()
			.add_systems(
				OnEnter(GameState::ClientConnecting),
				setup
					.run_if(env::is_client)
			)
			.add_systems(
				Update,
				(
						nonfatal_error_systems!(NETWORK_ERROR_MESSAGE, NetworkError, connecting),
				)
					.run_if(in_state(GameState::ClientConnecting))
					.run_if(env::is_client)
			)
			.add_systems(
				Update,
				(
					nonfatal_error_systems!(NETWORK_ERROR_MESSAGE, NetworkError, client, server_message, server_response),
				)
					.run_if(
						in_state(GameState::WorldSelect)
							.or_else(in_state(GameState::LoadingWorld))
							.or_else(in_state(GameState::InWorld))
					)
					.run_if(env::is_client)
			)
			.add_systems(
				Update,
				(
					nonfatal_error_systems!(TILE_EVENT_ERROR_MESSAGE, TileEventError, set_tile_event),
				)
					.run_if(
						in_state(GameState::InWorld)
							.or_else(in_state(GameState::LoadingWorld))
					)
					.run_if(env::is_client)
			);
	}
}

macro_rules! send_message {
    ($client:expr, $channel_id:expr, $message:expr) => {
		{
			$crate::utils::struct_enforce!($client, renet::RenetClient, ResMut<'_, renet::RenetClient>);
			$crate::utils::trait_enforce!($channel_id, Into<u8>);
			$client.send_message($channel_id, TryInto::<renet::Bytes>::try_into($message)?);
		}
	};
}

pub(crate) use send_message;

use networking::protocol::ClientId;

#[derive(Debug, Default, Component)]
pub struct LocalPlayer;

#[derive(Debug, Bundle)]
struct LocalPlayerBundle {
	data: PlayerData,
	client_id: ClientId,
	local_player: LocalPlayer,
}

#[derive(Debug, Resource)]
struct ConnectedTime(Instant);

#[derive(Debug, Resource)]
struct ConnectingTimeout(Duration);

/// Indicates that the connection has been established and that we have already sent the initial packets.
#[derive(Debug, Default, Resource)]
struct ConnectionEstablished;

fn setup(
	server_address: Res<ServerConnectAddress>,
	username: Res<Username>,
	mut commands: Commands,
	mut next_state: ResMut<NextState<GameState>>,
) {
	let connection_config = ConnectionConfig::default();
	
	let client_id = time_since_epoch().as_millis() as u64;
	
	// spawn local player
	commands.spawn(
		LocalPlayerBundle {
			data: PlayerData {
				username: username.clone(),
			},
			client_id: ClientId(client_id),
			local_player: default(),
		}
	);
	
	let client_addr = "0.0.0.0:0"; // request dynamic port
	let socket = UdpSocket::bind(&client_addr).expect(&format!("Failed to bind to address \"{}\"", client_addr)); // fixme: kick to disconnect screen
	let current_time = time_since_epoch();
	let server_addr: Option<SocketAddr> = { // weird hack
		// basically what this does is it goes to the disconnect screen if there's an error parsing the address
		let server_address = server_address.into_inner();
		let res = server_address.try_into();
		match res {
			Ok(addr) => Some(addr),
			Err(error) => {
				commands.insert_resource(DisconnectReason::AddrParseError(error));
				next_state.set(GameState::TitleScreen); // todo: disconnect screen
				return;
			},
		}
	};
	let server_addr: SocketAddr = server_addr.unwrap(); // should've returned if none, so we can unwrap
	
	// todo: authentication
	let authentication = ClientAuthentication::Unsecure {
		protocol_id: protocol::PROTOCOL_ID,
		client_id,
		server_addr,
		user_data: Some(username.to_user_data()),
	};
	
	let client = RenetClient::new(connection_config);
	let transport = NetcodeClientTransport::new(current_time, authentication, socket).expect("Failed to initialize NetcodeClientTransport");
	
	commands.insert_resource(client);
	commands.insert_resource(transport);
	commands.insert_resource(ConnectedTime(Instant::now()));
	commands.insert_resource(ConnectingTimeout(Duration::from_millis(protocol::CLIENT_TIMEOUT)));
}

fn connecting(
	client: Option<ResMut<RenetClient>>,
	transport: Option<ResMut<NetcodeClientTransport>>,
	mut commands: Commands,
	mut next_state: ResMut<NextState<GameState>>,
	connected_time: Option<Res<ConnectedTime>>,
	connecting_timeout: Option<Res<ConnectingTimeout>>,
	disconnect_reason: Option<Res<DisconnectReason>>,
	connection_established: Option<Res<ConnectionEstablished>>,
) -> Result<(), NetworkError> {
	if let Some(reason) = disconnect_reason {
		commands.remove_resource::<DisconnectReason>();
		println!("Disconnected.\nReason: {}", reason.into_inner());
		return Ok(())
	}
	
	if client.is_none() || transport.is_none() || connected_time.is_none() || connecting_timeout.is_none() {
		return Ok(())
	}
	
	let mut client = client.unwrap();
	let transport = transport.unwrap();
	let connected_time = connected_time.unwrap();
	let connecting_timeout = connecting_timeout.unwrap();
	
	if connected_time.0.elapsed() >= connecting_timeout.0 {
		on_disconnect(DisconnectReason::Transport(renet::transport::NetcodeDisconnectReason::ConnectionTimedOut), next_state.into_inner(), transport.into_inner(), client.into_inner());
		return Ok(())
	}
	
	if let Some(reason) = transport.disconnect_reason() {
		on_disconnect(DisconnectReason::Transport(reason), next_state.into_inner(), transport.into_inner(), client.into_inner());
		return Ok(())
	}
	
	if let Some(reason) = client.disconnect_reason() {
		on_disconnect(DisconnectReason::Client(reason), next_state.into_inner(), transport.into_inner(), client.into_inner());
		return Ok(())
	}
	
	if transport.is_connected() {
		if connection_established.is_none() {
			commands.init_resource::<ConnectionEstablished>();
			println!("Connection established");
			send_message!(client, DefaultChannel::ReliableOrdered, protocol::ClientMessage::JoinRequest { protocol_ver: protocol::PROTOCOL_VER });
		} else {
			if let Some(buf) = client.receive_message(DefaultChannel::ReliableOrdered) {
				let message = utils::deserialize_be::<protocol::Message>(&buf)?;
				match message {
					protocol::Message::ServerMessage(message) => {
						match message {
							ServerMessage::Disconnect(reason) => on_disconnect(DisconnectReason::Disconnected(reason), next_state.into_inner(), transport.into_inner(), client.into_inner()),
							ServerMessage::RawTileIds(raw_tile_ids) => commands.insert_resource(raw_tile_ids),
							_ => warn!("Unexpected message from server: {:?}", message),
						}
					},
					protocol::Message::ServerResponse(response) => {
						match response {
							ServerResponse::JoinAccept => next_state.set(GameState::WorldSelect),
							ServerResponse::JoinDeny(reason) => on_disconnect(DisconnectReason::Disconnected(reason), next_state.into_inner(), transport.into_inner(), client.into_inner()),
							_ => warn!("Unexpected message from server: {:?}", response),
						}
					},
					_ => warn!("Received incorrect/client message from server: {:?}", message),
				}
			}
		}
	}
	
	Ok(())
}

fn client(
	mut client: ResMut<RenetClient>,
	transport: ResMut<NetcodeClientTransport>,
	mut commands: Commands,
	next_state: ResMut<NextState<GameState>>,
) -> Result<(), NetworkError> {
	if let Some(reason) = transport.disconnect_reason() {
		on_disconnect(DisconnectReason::Transport(reason), next_state.into_inner(), transport.into_inner(), client.into_inner());
		return Ok(())
	}
	
	if let Some(reason) = client.disconnect_reason() {
		on_disconnect(DisconnectReason::Client(reason), next_state.into_inner(), transport.into_inner(), client.into_inner());
		return Ok(())
	}
	
	for channel_id in 0..=2 {
		while let Some(buf) = client.receive_message(channel_id) {
			let message = utils::deserialize_be::<protocol::Message>(&buf)?;
			match message {
				protocol::Message::ServerMessage(message) => {
					commands.spawn(message);
				},
				protocol::Message::ServerResponse(response) => {
					commands.spawn(response);
				},
				_ => warn!("Received incorrect/client message from server: {:?}", message),
			}
		}
	}
	
	Ok(())
}

fn server_message(
	message_query: Query<(Entity, &ServerMessage)>,
	mut client: ResMut<RenetClient>,
	mut commands: Commands,
	mut next_state: ResMut<NextState<GameState>>,
	mut transport: ResMut<NetcodeClientTransport>,
	raw_tile_ids: Res<RawTileIds>,
	mut ev_set_tile: EventWriter<SetTileEvent>,
) -> Result<(), NetworkError> {
	for (entity, message) in message_query.iter() {
		commands.entity(entity).despawn();
		println!("{:?}", message);
		match message {
			ServerMessage::Ping { timestamp } => {
				send_message!(client, DefaultChannel::ReliableOrdered, protocol::ClientResponse::PingAck { timestamp: *timestamp });
			},
			ServerMessage::ChatMessage(chat_message) => {
				commands.spawn(chat_message.clone());
			},
			ServerMessage::Disconnect(reason) => {
				on_disconnect(networking::DisconnectReason::Disconnected(reason.clone()), &mut next_state, &mut transport, &mut client);
			},
			ServerMessage::WorldTiles(tiles) => {
				for (pos, tile) in tiles.clone() {
					let id = raw_tile_ids.get_id(tile.0);
					if id.is_none() {
						return Err(NetworkError::TileEventError(TileEventError::InvalidRawId(tile.0, pos)))
					}
					
					ev_set_tile.send(
						SetTileEvent {
							pos,
							id: id.unwrap().clone(),
							data: tile.1,
						}
					);
				}
			},
			_ => {},
		}
	}
	
	Ok(())
}

fn server_response(
	response_query: Query<(Entity, &ServerResponse)>,
	mut next_state: ResMut<NextState<GameState>>,
	mut commands: Commands,
	mut client_world: Option<ResMut<ClientGameWorld>>,
) -> Result<(), NetworkError> {
	for (entity, response) in response_query.iter() {
		commands.entity(entity).despawn();
		println!("{:?}", response);
		match response {
			ServerResponse::PingAck { timestamp } => {
				let ping = time_since_epoch().as_millis() - timestamp;
				commands.insert_resource(Ping(ping));
			},
			ServerResponse::EnterWorldAccept(world_id) => {
				let client_world = client_world.as_mut().unwrap();
				client_world.id = *world_id;
				
				next_state.set(GameState::LoadingWorld);
			},
			ServerResponse::EnterWorldDeny(reason) => {
				commands.remove_resource::<ClientGameWorld>();
				println!("Failed to enter world. Reason: {reason:?}");
			},
			_ => {},
		}
	}
	
	Ok(())
}

fn set_tile_event(
	mut client_world: ResMut<ClientGameWorld>,
	mut ev_set_tile: EventReader<SetTileEvent>,
	mut commands: Commands,
	tile_registry: Res<TileRegistry>,
	raw_tile_ids: Res<RawTileIds>,
	tile_def_assets: Res<Assets<TileDef>>,
	asset_server: Res<AssetServer>,
) -> Result<(), TileEventError> {
	for event in ev_set_tile.iter() {
		// clear current tile
		client_world.tiles.remove(&event.pos);
		let tile_sprite = client_world.tile_sprites.remove(&event.pos);
		if let Some(tile_sprite) = tile_sprite {
			let sprite_commands = commands.get_entity(tile_sprite);
			if let Some(sprite_commands) = sprite_commands {
				sprite_commands.despawn_recursive();
			}
		}
		
		let def_handle = tile_registry.get(&event.id);
		if def_handle.is_none() {
			return Err(TileEventError::TileDefNotFound(event.id.clone(), event.pos))
		}
		
		let def = tile_def_assets.get(&def_handle.unwrap());
		if def.is_none() {
			return Err(TileEventError::TileDefNotFound(event.id.clone(), event.pos))
		}
		let def = def.unwrap();
		
		if !def.is_air() { // spawn new tile if this isn't air
			println!("{:?}", event.pos);
			let raw_id = raw_tile_ids.get_raw_id(&event.id);
			if raw_id.is_none() {
				return Err(TileEventError::InvalidId(event.id.clone(), event.pos))
			}
			
			client_world.tiles.insert(event.pos.clone(), WorldTile(raw_id.unwrap(), event.data.clone()));
			
			// don't render the tile if it's invisible
			if def.settings().salience() == TileSalience::Invisible {
				return Ok(())
			}
			
			let mut tile_image_handle: Handle<Image> = DEFAULT_IMAGE_HANDLE.typed();
			if !def.is_missingno() { // if it isn't missingno, we can safely load the image
				tile_image_handle = load_image(&asset_server, format!("{}/textures/tile/{}.png", event.id.namespace(), event.id.path()));
			}
			
			commands.spawn(
				SpriteBundle {
					texture: tile_image_handle,
					transform: Transform::from_xyz(event.pos.x as f32, event.pos.y as f32, def.settings().salience().into_z()),
					sprite: Sprite {
						custom_size: Some(Vec2::new(1.0, 1.0)),
						..default()
					},
					..default()
				}
			);
		}
	}
	
	Ok(())
}

pub fn send_chat(
	mut client: ResMut<RenetClient>,
	target: Target,
	message: String,
) -> Result<(), NetworkError> {
	send_message!(client, 0, protocol::ClientMessage::ChatMessage(target, message));
	Ok(())
}

pub fn disconnect(reason: DisconnectReason, transport: &mut NetcodeClientTransport, client: &mut RenetClient, disconnect_client: bool) {
	if disconnect_client {
		transport.disconnect();
		client.disconnect_due_to_transport();
	}
	println!("Disconnected.\nReason: {}", reason);
}

fn on_disconnect(reason: DisconnectReason, next_state: &mut NextState<GameState>, transport: &mut NetcodeClientTransport, client: &mut RenetClient) {
	disconnect(reason, transport, client, false);
	next_state.set(GameState::TitleScreen); // todo: disconnect screen
}
