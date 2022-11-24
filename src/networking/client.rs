use std::net::UdpSocket;
use std::net::SocketAddr;
use std::str::FromStr;
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use rand::{Rng, RngCore};
use rand::rngs::OsRng;
use renet::{ConnectToken, RenetClient, RenetConnectionConfig};
use crate::{env, GameState, ServerAddressPort, util};
use crate::networking::{Ping, protocol, Username};
use crate::networking::protocol::{PlayerData, ServerMessage, ServerResponse};
use crate::player::Target;

#[derive(Debug)]
pub struct LocalPlayer(PlayerData, u64);

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_enter_system(
				GameState::ClientConnecting,
				setup
					.run_if(env::is_client)
			)
			.add_system(
				client
					.run_in_state(GameState::WorldSelect)
					.run_in_state(GameState::LoadingWorld)
					.run_in_state(GameState::InWorld)
					.run_if(env::is_client)
			)
			.add_system(server_message)
			.add_system(server_response);
	}
}

fn setup(
	server_address: Res<ServerAddressPort>,
	username: Res<Username>,
	mut commands: Commands,
	mut time: ResMut<Time>,
) {
	let connection_config = RenetConnectionConfig::default();
	
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
	
	let client_addr = format!("127.0.0.1:{}", server_address.1);
	let socket = UdpSocket::bind(&client_addr).expect(&format!("Failed to bind to address \"{}\"", client_addr));
	let current_time = time.time_since_startup();
	
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
	
	let client = RenetClient::new(current_time, socket, client_id, connect_token, connection_config).expect("Failed to create client");
	
	commands.insert_resource(client);
	commands.insert_resource(NextState(GameState::WorldSelect));
}

fn client(
	mut client: ResMut<RenetClient>,
	mut time: ResMut<Time>,
	mut commands: Commands,
) {
	time.update();
	
	let current_time = time.time_since_startup();
	
	client.update(current_time).expect("Failed to update client");
	
	if let Some(reason) = client.disconnected() {
		println!("Disconnected.\nReason: {}", reason);
	}
	
	for channel_id in 0..=2 {
		while let Some(buf) = client.receive_message(channel_id) {
			let message = util::deserialize::<protocol::Message>(&buf);
			if let Ok(message) = message {
				match message {
					protocol::Message::ServerMessage(message) => {
						commands.spawn().insert(message);
					}
					protocol::Message::ServerResponse(response) => {
						commands.spawn().insert(response);
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
				commands.spawn_bundle(chat_message.clone());
			}
			ServerMessage::Disconnect(reason) => {
				println!("Disconnected.\nReason: {:?}", reason);
				client.disconnect();
			}
			ServerMessage::PlayerJoin()
			_ => {}
		}
	}
}

fn server_response(
	response_query: Query<&ServerResponse>,
	mut commands: Commands,
	mut world: ResMut<World>,
	mut time: ResMut<Time>,
) {
	time.update();
	
	for response in response_query.iter() {
		match response {
			ServerResponse::PingAck { timestamp } => {
				let ping = (time.time_since_startup().as_millis() - timestamp) as u32;
				world.insert_resource(Ping(ping));
			}
			ServerResponse::EnterWorldAccept => commands.insert_resource(NextState(GameState::LoadingWorld)),
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
