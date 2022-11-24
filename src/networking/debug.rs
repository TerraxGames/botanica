use bevy::prelude::*;
use bevy_egui::EguiContext;
use iyes_loopless::prelude::*;
use renet::{RenetClient, RenetServer, ServerEvent};
use renet_visualizer::{RenetClientVisualizer, RenetServerVisualizer};
use crate::{env, GameState, is_debug};

const VISUALIZER_UPDATE: usize = 200;

pub struct NetworkingDebugPlugin;

impl Plugin for NetworkingDebugPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_system(
				debug_server
					.run_if(is_debug)
					.run_if(env::is_server)
					.run_in_state(GameState::InWorld)
			)
			.add_system(
				debug_client
					.run_if(is_debug)
					.run_if(env::is_client)
					.run_in_state(GameState::LoadingWorld)
					.run_in_state(GameState::InWorld)
			);
	}
}

fn debug_server(
	mut server: ResMut<RenetServer>,
	mut visualizer: ResMut<RenetServerVisualizer<VISUALIZER_UPDATE>>,
	mut gui_ctx: ResMut<EguiContext>,
) {
	while let Some(event) = server.get_event() {
		match event {
			ServerEvent::ClientConnected(client_id, _) => {
				visualizer.add_client(client_id);
			}
			ServerEvent::ClientDisconnected(client_id) => {
				visualizer.remove_client(client_id);
			}
		}
	}
	
	visualizer.update(&server);
	
	visualizer.show_window(gui_ctx.ctx_mut());
}

fn debug_client(
	client: Res<RenetClient>,
	mut visualizer: ResMut<RenetClientVisualizer<VISUALIZER_UPDATE>>,
	mut gui_ctx: ResMut<EguiContext>,
) {
	visualizer.add_network_info(client.network_info());
	
	visualizer.show_window(gui_ctx.ctx_mut());
}
