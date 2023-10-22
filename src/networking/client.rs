use std::net::SocketAddr;
use std::net::UdpSocket;
use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::Instant;
use bevy_renet::RenetClientPlugin;
use bevy_renet::transport::NetcodeClientPlugin;
use renet::{ConnectionConfig, RenetClient};
use renet::transport::{ClientAuthentication, NetcodeClientTransport};

use crate::{env, GameState, ServerConnectAddress, util};
use crate::networking::{DisconnectReason, Ping, protocol, time_since_epoch, Username};
use crate::networking::protocol::{PlayerData, ServerMessage, ServerResponse};
use crate::player::Target;

#[derive(Debug, Resource)]
pub struct LocalPlayer(pub PlayerData, pub u64);

#[derive(Debug, Resource)]
struct ConnectedTime(Instant);

#[derive(Debug, Resource)]
struct ConnectingTimeout(Duration);

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
				connecting
					.run_if(in_state(GameState::ClientConnecting))
					.run_if(env::is_client)
			)
			.add_systems(
				Update,
				(
					client,
					server_message,
					server_response
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

fn setup(
	server_address: Res<ServerConnectAddress>,
	username: Res<Username>,
	mut commands: Commands,
	mut next_state: ResMut<NextState<GameState>>,
) {
	let connection_config = ConnectionConfig::default();
	
	let client_id = time_since_epoch().as_millis() as u64;
	
	commands.insert_resource(
		LocalPlayer(
			PlayerData {
				username: username.clone(),
			},
			client_id,
		)
	);
	
	let client_addr = "0.0.0.0:0"; // request dynamic port
	let socket = UdpSocket::bind(&client_addr).expect(&format!("Failed to bind to address \"{}\"", client_addr));
	let current_time = time_since_epoch();
	let server_addr: Option<SocketAddr> = { // weird hack
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
	mut client: Option<ResMut<RenetClient>>,
	mut transport: Option<ResMut<NetcodeClientTransport>>,
	mut commands: Commands,
	mut next_state: ResMut<NextState<GameState>>,
	connected_time: Option<Res<ConnectedTime>>,
	connecting_timeout: Option<Res<ConnectingTimeout>>,
	disconnect_reason: Option<Res<DisconnectReason>>,
) {
	if let Some(reason) = disconnect_reason {
		commands.remove_resource::<DisconnectReason>();
		println!("Disconnected.\nReason: {}", reason.into_inner());
		return
	}
	
	if client.is_none() || transport.is_none() || connected_time.is_none() || connecting_timeout.is_none() {
		return
	}
	
	let mut client = client.unwrap();
	let mut transport = transport.unwrap();
	let connected_time = connected_time.unwrap();
	let connecting_timeout = connecting_timeout.unwrap();
	
	if connected_time.0.elapsed() >= connecting_timeout.0 {
		on_disconnect(DisconnectReason::Transport(renet::transport::NetcodeDisconnectReason::ConnectionTimedOut), next_state, transport.into_inner(), client.into_inner());
		return
	}
	
	if let Some(reason) = transport.disconnect_reason() {
		on_disconnect(DisconnectReason::Transport(reason), next_state, transport.into_inner(), client.into_inner());
		return
	}
	
	if let Some(reason) = client.disconnect_reason() {
		on_disconnect(DisconnectReason::Client(reason), next_state, transport.into_inner(), client.into_inner());
		return
	}
	
	if transport.is_connected() {
		println!("Connected to server.");
		next_state.set(GameState::WorldSelect)
	}
}

fn client(
	mut client: ResMut<RenetClient>,
	mut transport: ResMut<NetcodeClientTransport>,
	mut commands: Commands,
	next_state: ResMut<NextState<GameState>>,
) {
	if let Some(reason) = transport.disconnect_reason() {
		on_disconnect(DisconnectReason::Transport(reason), next_state, transport.into_inner(), client.into_inner());
		return
	}
	
	if let Some(reason) = client.disconnect_reason() {
		on_disconnect(DisconnectReason::Client(reason), next_state, transport.into_inner(), client.into_inner());
		return
	}
	
	for channel_id in 0..=2 {
		while let Some(buf) = client.receive_message(channel_id) {
			let message = util::deserialize::<protocol::Message>(&buf);
			if let Ok(message) = message {
				match message {
					protocol::Message::ServerMessage(message) => {
						commands.spawn(message);
					}
					protocol::Message::ServerResponse(response) => {
						commands.spawn(response);
					}
					_ => warn!("Received client message from server: {:?}", message),
				}
			}
		}
	}
}

fn server_message(
	message_query: Query<&ServerMessage>,
	mut client: ResMut<RenetClient>,
	mut commands: Commands,
) {
	for message in message_query.iter() {
		match message {
			ServerMessage::Ping { timestamp } => {
				let message = util::serialize(&protocol::Message::ClientResponse(protocol::ClientResponse::PingAck { timestamp: *timestamp })).unwrap();
				client.send_message(0, message);
			}
			ServerMessage::ChatMessage(chat_message) => {
				commands.spawn(chat_message.clone());
			}
			ServerMessage::Disconnect(reason) => {
				println!("Disconnected.\nReason: {:?}", reason);
				client.disconnect();
			}
			_ => {}
		}
	}
}

fn server_response(
	response_query: Query<&ServerResponse>,
	mut next_state: ResMut<NextState<GameState>>,
	mut commands: Commands,
) {
	for response in response_query.iter() {
		match response {
			ServerResponse::PingAck { timestamp } => {
				let ping = (time_since_epoch().as_millis() - timestamp) as u32;
				commands.insert_resource(Ping(ping));
			}
			ServerResponse::EnterWorldAccept => next_state.set(GameState::LoadingWorld),
			ServerResponse::EnterWorldDeny(reason) => println!("Failed to enter world.\nReason: {:?}", reason),
			_ => {}
		}
	}
}

pub fn send_chat(
	mut client: ResMut<RenetClient>,
	target: Target,
	message: String,
) {
	let message = protocol::Message::ClientMessage(protocol::ClientMessage::ChatMessage(target, message));
	client.send_message(0, util::serialize(&message).unwrap());
}

pub fn disconnect(reason: DisconnectReason, transport: &mut NetcodeClientTransport, client: &mut RenetClient, disconnect_client: bool) {
	if disconnect_client {
		transport.disconnect();
		client.disconnect_due_to_transport();
	}
	println!("Disconnected.\nReason: {}", reason);
}

fn on_disconnect(reason: DisconnectReason, mut next_state: ResMut<NextState<GameState>>, transport: &mut NetcodeClientTransport, client: &mut RenetClient) {
	disconnect(reason, transport, client, false);
	next_state.set(GameState::TitleScreen); // todo: disconnect screen
}
