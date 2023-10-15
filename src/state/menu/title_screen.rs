use std::fmt;
use std::fmt::Formatter;

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::{asset, DEFAULT_LOCALE, despawn_with, from_asset_loc, GameState, LocaleAsset, menu, NAMESPACE, Translatable};
use crate::menu::{BUTTON_BOTTOM_PADDING, BUTTON_HEIGHT, BUTTON_SCALE, BUTTON_TEXT_SIZE, BUTTON_WIDTH};
use crate::menu::button::{PreviousButtonInteraction, PreviousButtonProperties};
use crate::state::menu::{BACKGROUND, NORMAL_BUTTON, TEXT_MARGIN};
use crate::state::menu::button::{ButtonColor, ButtonDownImage, ButtonImageBundle, ButtonUpImage};

pub mod button;

pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(OnEnter(GameState::TitleScreen), setup)
			.add_systems(OnExit(GameState::TitleScreen), despawn_with::<OnTitleScreen>)
			.add_systems(
				Update,
				(menu::button::style, button_action)
					.run_if(in_state(GameState::TitleScreen))
			);
	}
}

#[derive(Component)]
struct OnTitleScreen;

#[derive(Debug, Component)]
enum ButtonAction {
	Singleplayer,
	Multiplayer,
	Quit,
}

impl fmt::Display for ButtonAction {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str("ButtonAction::")?;
		fmt::Debug::fmt(self, f)
	}
}

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	locale_assets: Res<Assets<LocaleAsset>>,
) {
	let monogram = asset_server.get_handle(from_asset_loc(NAMESPACE, "fonts/monogram/monogram-extended.ttf"));
	let button_up = ButtonUpImage::from(asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/button/button_up.png")));
	let button_down = ButtonDownImage::from(asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/button/button_down.png")));

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
		.insert(OnTitleScreen)
		.with_children(|parent| {
			let button_image_bundle = ButtonImageBundle {
				button_up,
				button_down,
				..default()
			};
			let button_size = (Val::Px(BUTTON_WIDTH * BUTTON_SCALE), Val::Px(BUTTON_HEIGHT * BUTTON_SCALE));
			let button_margin = UiRect::all(Val::Auto);
			let (justify_content, align_items) = (JustifyContent::Center, AlignItems::Center);
			let button_padding = UiRect {
				bottom: Val::Px(BUTTON_BOTTOM_PADDING * BUTTON_SCALE),
				..default()
			};
			
			// title
			parent
				.spawn(
					TextBundle {
						style: Style {
							margin: UiRect::all(TEXT_MARGIN),
							position_type: PositionType::Absolute,
							top: Val::Percent(1.0),
							..default()
						},
						text: Text::from_section(
							Translatable::translate_once(
								asset::namespaced(NAMESPACE, "ui.title_screen.text.title").as_str(),
								DEFAULT_LOCALE,
								&asset_server,
								&locale_assets,
							),
							TextStyle {
								font: monogram.clone(),
								font_size: 90.0,
								color: Color::WHITE,
							},
						),
						..default()
					}
				);
			
			// buttons
			parent
				.spawn(
					ImageBundle {
						style: Style {
							width: Val::Px(512.0),
							height: Val::Percent(50.0),
							justify_content: JustifyContent::Center,
							align_items: AlignItems::Center,
							flex_direction: FlexDirection::ColumnReverse,
							..default()
						},
						background_color: Color::NONE.into(),
						..default()
					}
				)
				.with_children(|parent| {
					// multiplayer button
					parent
						.spawn(
							ButtonBundle {
								style: Style {
									width: button_size.0,
									height: button_size.1,
									margin: button_margin,
									justify_content,
									align_items,
									padding: button_padding,
									..default()
								},
								..default()
							}
						)
						.insert(ButtonColor(NORMAL_BUTTON))
						.insert(button_image_bundle.clone())
						.insert(PreviousButtonProperties::default())
						.insert(ButtonAction::Multiplayer)
						.with_children(|parent| {
							parent
								.spawn(
									TextBundle {
										style: Style {
											margin: UiRect::all(TEXT_MARGIN),
											..default()
										},
										text: Text::from_section(
											Translatable::translate_once(
												asset::namespaced(NAMESPACE, "ui.title_screen.button.multiplayer").as_str(),
												DEFAULT_LOCALE,
												&asset_server,
												&locale_assets,
											),
											TextStyle {
												font: monogram.clone(),
												font_size: BUTTON_TEXT_SIZE,
												color: Color::BLACK,
											}
										),
										..default()
									}
								);
						});
					
					// quit button
					parent
						.spawn(
							ButtonBundle {
								style: Style {
									width: button_size.0,
									height: button_size.1,
									margin: button_margin,
									justify_content,
									align_items,
									padding: button_padding,
									..default()
								},
								..default()
							}
						)
						.insert(ButtonColor(NORMAL_BUTTON))
						.insert(button_image_bundle.clone())
						.insert(PreviousButtonProperties::default())
						.insert(ButtonAction::Quit)
						.with_children(|parent| {
							parent
								.spawn(
									TextBundle {
										style: Style {
											margin: UiRect::all(TEXT_MARGIN),
											..default()
										},
										text: Text::from_section(
											Translatable::translate_once(
												asset::namespaced(NAMESPACE, "ui.title_screen.button.quit").as_str(),
												DEFAULT_LOCALE,
												&asset_server,
												&locale_assets,
											),
											TextStyle {
												font: monogram.clone(),
												font_size: BUTTON_TEXT_SIZE,
												color: Color::BLACK,
											}
										),
										..default()
									}
								);
						});
				});
		});
}

fn button_action(
	interaction_query: Query<
		(&Interaction, &PreviousButtonInteraction, &ButtonAction),
		(Changed<Interaction>, With<Button>),
	>,
	mut app_exit_events: EventWriter<AppExit>,
	mut next_state: ResMut<NextState<GameState>>,
) {
	for (interaction, previous_interaction, button_action) in interaction_query.iter() {
		// only execute action if still hovering after click ends
		if *interaction == Interaction::Hovered && *previous_interaction == Interaction::Pressed.into() {
			match button_action {
				// todo: singleplayer
				ButtonAction::Multiplayer => next_state.set(GameState::ServerSelect),
				ButtonAction::Quit => app_exit_events.send(AppExit),
				_ => unimplemented!("{}", button_action),
			}
		}
	}
}
