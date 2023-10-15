use std::net::SocketAddr;
use std::net::UdpSocket;
use std::str::FromStr;

use bevy::prelude::*;
use rand::{Rng, RngCore};
use rand::rngs::OsRng;
use renet::{ConnectionConfig, RenetClient};
use renet::transport::{ClientAuthentication, ConnectToken, NetcodeClientTransport};

use crate::{env, GameState, ServerAddressPort, util};
use crate::networking::{Ping, protocol, Username};
use crate::networking::protocol::{PlayerData, ServerMessage, ServerResponse};
use crate::player::Target;

#[derive(Debug, Resource)]
pub struct LocalPlayer(PlayerData, u64);

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(
				OnEnter(GameState::ClientConnecting),
				setup
					.run_if(env::is_client)
			)
			.add_systems(
				Update,
				(
					client,
					server_message,
					server_response
				)
					.run_if(in_state(GameState::WorldSelect))
					.run_if(in_state(GameState::LoadingWorld))
					.run_if(in_state(GameState::InWorld))
					.run_if(env::is_client)
			);
	}
}

fn setup(
	server_address: Res<ServerAddressPort>,
	username: Res<Username>,
	mut commands: Commands,
	mut next_state: ResMut<NextState<GameState>>,
	mut time: ResMut<Time>,
) {
	let connection_config = ConnectionConfig::default();
	
	time.update();
	
	let mut rng = OsRng::default();
	let private_key = &mut [0u8; 32];
	rng.fill(private_key);
	
	let client_id = rng.next_u64();
	
	commands.insert_resource(
		LocalPlayer(
			PlayerData {
				username: username.clone(),
			},
			client_id,
		)
	);
	
	let client_addr = "0.0.0.0:44737"; // i think we should keep it on this port, since it doesn't conflict with the default one (in case one is running a server on the same
	let socket = UdpSocket::bind(&client_addr).expect(&format!("Failed to bind to address \"{}\"", client_addr));
	let current_time = time.elapsed();
	
	// todo: authentication
	let connect_token = ConnectToken::generate(
		current_time,
		protocol::PROTOCOL_ID,
		300,
		client_id,
		15,
		vec![SocketAddr::from_str(&server_address.0).unwrap()],
		Some(&username.to_user_data()),
		private_key,
	).unwrap();
	let authentication = ClientAuthentication::Secure { connect_token };
	
	let client = RenetClient::new(connection_config);
	let transport = NetcodeClientTransport::new(current_time, authentication, socket).expect("Failed to create new NetcodeClientTransport");
	
	commands.insert_resource(client);
	commands.insert_resource(transport);
	next_state.set(GameState::WorldSelect);
}

fn client(
	mut client: ResMut<RenetClient>,
	mut transport: ResMut<NetcodeClientTransport>,
	mut time: ResMut<Time>,
	mut commands: Commands,
) {
	time.update();
	
	let current_time = time.elapsed();
	
	client.update(current_time);
	transport.update(current_time, &mut client).expect("Failed to update NetcodeClientTransport");
	
	if let Some(reason) = client.disconnect_reason() {
		println!("Disconnected.\nReason: {}", reason);
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
	mut time: ResMut<Time>,
) {
	time.update();
	
	for response in response_query.iter() {
		match response {
			ServerResponse::PingAck { timestamp } => {
				let ping = (time.elapsed().as_millis() - timestamp) as u32;
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
