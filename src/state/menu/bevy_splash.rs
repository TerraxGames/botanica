use bevy::prelude::*;

use crate::{asset, DEFAULT_LOCALE, despawn_with, from_asset_loc, GameState, LocaleAsset, NAMESPACE, Translatable};
use crate::state::menu::{BACKGROUND, TEXT_MARGIN};

/// The amount of time (in seconds) to show the "made with Bevy" splash.
const BEVY_SPLASH_TIME: f32 = {
	#[cfg(feature = "fast_bevy_splash")]
	{ 1.0 }
	#[cfg(not(feature = "fast_bevy_splash"))]
	{ 5.0 } // don't be sad, i just really love bevy!
};

pub struct BevySplashPlugin;

impl Plugin for BevySplashPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(OnEnter(GameState::BevySplash), setup)
			.add_systems(OnExit(GameState::BevySplash), despawn_with::<OnBevySplash>)
			.add_systems(
				Update,
				check_timer_up
					.run_if(in_state(GameState::BevySplash))
			);
	}
}

#[derive(Component)]
struct OnBevySplash;

#[derive(Component)]
struct BevySplashTimer(pub Timer);

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	locale_assets: Res<Assets<LocaleAsset>>
) {
	let monogram = asset_server.get_handle(from_asset_loc(NAMESPACE, "fonts/monogram/monogram-extended.ttf"));
	let bevy_logo = asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/branding/bevy_logo.png"));
	
	// root
	commands
		.spawn(
			ImageBundle {
				style: Style {
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					flex_direction: FlexDirection::ColumnReverse,
					..default()
				},
				// background color
				background_color: BACKGROUND.into(),
				..default()
			}
		)
		.insert(OnBevySplash)
		.insert(BevySplashTimer(Timer::from_seconds(BEVY_SPLASH_TIME, TimerMode::Once)))
		.with_children(|parent| {
			// logo
			parent
				.spawn(
					ImageBundle {
						style: Style {
							width: Val::Px(256.0),
							height: Val::Px(256.0),
							..default()
						},
						image: bevy_logo.into(),
						..default()
					}
				);
			// "made with" text
			parent
				.spawn(
					TextBundle {
						style: Style {
							margin: UiRect::all(TEXT_MARGIN),
							..default()
						},
						text: Text::from_section(
							Translatable::translate_once(
								asset::namespaced(NAMESPACE, "ui.bevy_splash.text.made_with").as_str(),
								DEFAULT_LOCALE,
								&asset_server,
								&locale_assets,
							),
							TextStyle {
								font: monogram,
								font_size: 45.0,
								color: Color::WHITE,
							},
						),
						..default()
					}
				);
		});
}

fn check_timer_up(
	mut commands: Commands,
	mut timer_query: Query<(&mut BevySplashTimer)>,
	mut next_state: ResMut<NextState<GameState>>,
	time: Res<Time>,
) {
	for mut timer in timer_query.iter_mut() {
		timer.0.tick(time.delta());
		if timer.0.finished() {
			next_state.set(GameState::TitleScreen)
		}
	}
}
