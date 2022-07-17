#![allow(unused)]

pub mod title_screen;
pub mod button;
pub mod bevy_splash;

use bevy::prelude::*;

const TEXT_MARGIN: Val = Val::Px(5.0);

const NORMAL_BUTTON: Color = Color::rgb(1.0, 1.0, 1.0);
const HOVERED_BUTTON: Color = Color::rgb(0.75, 0.75, 0.75);

const WHITE_BUTTON: Color = NORMAL_BUTTON;
const RED_BUTTON: Color = Color::rgb(0.905882352941, 0.298039215686, 0.235294117647);
const BLUE_BUTTON: Color = Color::rgb(0.203921568627, 0.596078431372, 0.858823529412);
const GREEN_BUTTON: Color = Color::rgb(0.180392156863, 0.8, 0.443137254902);
const CYAN_BUTTON: Color = Color::rgb(0.101960784314, 0.737254901961, 0.611764705882);
const YELLOW_BUTTON: Color = Color::rgb(0.945098039216, 0.768627450981, 0.0588235294118);
const MAGENTA_BUTTON: Color = Color::rgb(0.607843137255, 0.349019607843, 0.713725490196);

const BACKGROUND: Color = Color::rgb(0.125, 0.125, 0.125);

const BUTTON_BOTTOM_PADDING: f32 = 8.0;
const BUTTON_SCALE: f32 = 2.0;
const BUTTON_WIDTH: f32 = 128.0;
const BUTTON_HEIGHT: f32 = 32.0;
const BUTTON_TEXT_SIZE: f32 = 32.0;

/// Initializes the UI.
pub fn init_ui(
	mut commands: Commands,
) {
	commands.spawn_bundle(UiCameraBundle::default());
}
