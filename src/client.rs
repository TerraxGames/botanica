use std::net::SocketAddr;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
use bevy_renet::transport::NetcodeClientPlugin;
use renet::{ConnectionConfig, DefaultChannel, RenetClient};
use renet::transport::{ClientAuthentication, NetcodeClientTransport};

use crate::networking;
use crate::world::ClientGameWorld;
use crate::{env, GameState, ServerConnectAddress, util};
use crate::networking::{DisconnectReason, Ping, protocol, time_since_epoch, Username};
use crate::networking::error::{NETWORK_ERROR_MESSAGE, NetworkError};
use crate::networking::protocol::{PlayerData, ServerMessage, ServerResponse};
use crate::player::Target;
use crate::util::nonfatal_error_systems;

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
			);
	}
}

macro_rules! send_message {
    ($client:expr, $channel_id:expr, $message:expr) => {
		{
			$crate::util::struct_enforce!($client, renet::RenetClient, ResMut<'_, renet::RenetClient>);
			$crate::util::trait_enforce!($channel_id, Into<u8>);
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
				let message = util::deserialize_be::<protocol::Message>(&buf)?;
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
			let message = util::deserialize_be::<protocol::Message>(&buf)?;
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
	mut client_world: ResMut<ClientGameWorld>,
	mut next_state: ResMut<NextState<GameState>>,
	mut transport: ResMut<NetcodeClientTransport>,
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
				client_world.tiles = tiles.clone();
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
	mut client_world: ResMut<ClientGameWorld>,
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
