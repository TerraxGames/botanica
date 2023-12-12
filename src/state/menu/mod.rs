#![allow(unused)]

use std::convert::Into;

use bevy::asset::{AssetIo, AssetIoError, FileAssetIo};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy_egui::{egui, EguiContext, EguiContexts};
use futures::executor;

use crate::{from_asset_loc, load_asset_bytes, NAMESPACE};

pub mod title_screen;
pub mod button;
pub mod bevy_splash;
pub mod server_select;
pub mod world_select;

const TEXT_MARGIN: Val = Val::Px(5.0);

const NORMAL_BUTTON: BackgroundColor = BackgroundColor(Color::rgb(1.0, 1.0, 1.0));
const HOVERED_BUTTON: BackgroundColor = BackgroundColor(Color::rgb(0.75, 0.75, 0.75));

const WHITE_BUTTON: BackgroundColor = NORMAL_BUTTON;
const RED_BUTTON: BackgroundColor = BackgroundColor(Color::rgb(0.905882352941, 0.298039215686, 0.235294117647));
const BLUE_BUTTON: BackgroundColor = BackgroundColor(Color::rgb(0.203921568627, 0.596078431372, 0.858823529412));
const GREEN_BUTTON: BackgroundColor = BackgroundColor(Color::rgb(0.180392156863, 0.8, 0.443137254902));
const CYAN_BUTTON: BackgroundColor = BackgroundColor(Color::rgb(0.101960784314, 0.737254901961, 0.611764705882));
const YELLOW_BUTTON: BackgroundColor = BackgroundColor(Color::rgb(0.945098039216, 0.768627450981, 0.0588235294118));
const MAGENTA_BUTTON: BackgroundColor = BackgroundColor(Color::rgb(0.607843137255, 0.349019607843, 0.713725490196));

const BACKGROUND: BackgroundColor = BackgroundColor(Color::rgb(0.125, 0.125, 0.125));

const BUTTON_BOTTOM_PADDING: f32 = 8.0;
const BUTTON_SCALE: f32 = 2.0;
const BUTTON_WIDTH: f32 = 128.0;
const BUTTON_HEIGHT: f32 = 32.0;
const BUTTON_TEXT_SIZE: f32 = 32.0;

/// Initializes the UI.
pub fn init_ui(
	mut commands: Commands,
	mut contexts: EguiContexts,
	asset_server: Res<AssetServer>,
) {
	commands.spawn(
		Camera2dBundle {
			projection: OrthographicProjection {
				scale: 1.0 / 8.0,
				scaling_mode: ScalingMode::WindowSize(1.0),
				near: -1000.0,
				far: 1000.0,
				..default()
			},
			..default()
		}
	);
	
	let monogram = executor::block_on(load_asset_bytes(from_asset_loc(NAMESPACE, "fonts/monogram/monogram-extended.ttf"), asset_server.asset_io())).expect("Failed to load monogram font");
	let mut fonts = egui::FontDefinitions::default();
	
	fonts.font_data.insert(
		"monogram".to_owned(),
		egui::FontData::from_owned(monogram),
	);
	
	fonts
		.families
		.entry(egui::FontFamily::Proportional)
		.or_default()
		.insert(0, "monogram".to_owned());
	
	fonts
		.families
		.entry(egui::FontFamily::Monospace)
		.or_default()
		.insert(0, "monogram".to_owned());
	
	contexts.ctx().set_fonts(fonts);
	
	let mut style = egui::Style::default();
	let ref mut visuals = style.visuals;
	
	visuals.override_text_color = Some(egui::Color32::WHITE);
	
	contexts.ctx_mut().set_style(style);
}
