use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiContexts};
use renet::{RenetClient, RenetServer, ServerEvent};
use renet_visualizer::{RenetClientVisualizer, RenetServerVisualizer};

use crate::{env, GameState, is_debug};

const VISUALIZER_UPDATE: usize = 200;

pub struct NetworkingDebugPlugin;

impl Plugin for NetworkingDebugPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(
				Update,
				debug_server
					.run_if(in_state(GameState::InWorld))
					.run_if(is_debug)
					.run_if(env::is_server)
			)
			.add_systems(
				Update,
				debug_client
					.run_if(in_state(GameState::LoadingWorld))
					.run_if(in_state(GameState::InWorld))
					.run_if(is_debug)
					.run_if(env::is_client)
			);
	}
}

fn debug_server(
	mut server: ResMut<RenetServer>,
	mut visualizer: ResMut<RenetServerVisualizer<VISUALIZER_UPDATE>>,
	mut contexts: EguiContexts,
) {
	while let Some(event) = server.get_event() {
		match event {
			ServerEvent::ClientConnected { client_id } => {
				visualizer.add_client(client_id);
			}
			ServerEvent::ClientDisconnected { client_id, .. } => {
				visualizer.remove_client(client_id);
			}
		}
	}
	
	visualizer.update(&server);
	
	visualizer.show_window(contexts.ctx());
}

fn debug_client(
	client: Res<RenetClient>,
	mut visualizer: ResMut<RenetClientVisualizer<VISUALIZER_UPDATE>>,
	mut contexts: EguiContexts,
) {
	visualizer.add_network_info(client.network_info());
	
	visualizer.show_window(contexts.ctx());
}
