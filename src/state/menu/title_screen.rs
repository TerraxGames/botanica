pub mod button;

use std::fmt;
use std::fmt::Formatter;
use bevy::app::AppExit;
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use crate::{asset, DEFAULT_LOCALE, despawn_with, from_asset_loc, GameState, LocaleAsset, menu, NAMESPACE, Translatable};
use crate::menu::{BUTTON_HEIGHT, BUTTON_SCALE, BUTTON_BOTTOM_PADDING, BUTTON_WIDTH, BUTTON_TEXT_SIZE};
use crate::menu::button::{PreviousButtonInteraction, PreviousButtonProperties};
use crate::state::menu::{BACKGROUND, BLUE_BUTTON, NORMAL_BUTTON, RED_BUTTON, TEXT_MARGIN};
use crate::state::menu::button::{ButtonColor, ButtonDownImage, ButtonImageBundle, ButtonUpImage};

pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_enter_system(GameState::TitleScreen, setup)
			.add_exit_system(GameState::TitleScreen, despawn_with::<OnTitleScreen>)
			.add_system(
				menu::button::style
					.run_in_state(GameState::TitleScreen)
			)
			.add_system(
				button_action
					.run_in_state(GameState::TitleScreen)
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
		.spawn_bundle(
			ImageBundle {
				style: Style {
					size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					flex_direction: FlexDirection::ColumnReverse,
					..default()
				},
				// background color
				color: BACKGROUND.into(),
				..default()
			}
		)
		.with_children(|parent| {
			let button_image_bundle = ButtonImageBundle {
				button_up,
				button_down,
				..default()
			};
			let button_size = Size::new(Val::Px(BUTTON_WIDTH * BUTTON_SCALE), Val::Px(BUTTON_HEIGHT * BUTTON_SCALE));
			let button_margin = Rect::all(Val::Auto);
			let (justify_content, align_items) = (JustifyContent::Center, AlignItems::Center);
			let button_padding = Rect {
				bottom: Val::Px(BUTTON_BOTTOM_PADDING * BUTTON_SCALE),
				..default()
			};
			
			// title
			parent
				.spawn_bundle(
					TextBundle {
						style: Style {
							margin: Rect::all(TEXT_MARGIN),
							position_type: PositionType::Absolute,
							position: Rect {
								top: Val::Percent(1.0),
								..default()
							},
							..default()
						},
						text: Text::with_section(
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
							default(),
						),
						..default()
					}
				);
			
			// buttons
			parent
				.spawn_bundle(
					ImageBundle {
						style: Style {
							size: Size::new(Val::Px(512.0), Val::Percent(50.0)),
							justify_content: JustifyContent::Center,
							align_items: AlignItems::Center,
							flex_direction: FlexDirection::ColumnReverse,
							..default()
						},
						color: Color::NONE.into(),
						..default()
					}
				)
				.with_children(|parent| {
					// multiplayer button
					parent
						.spawn_bundle(
							ButtonBundle {
								style: Style {
									size: button_size,
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
						.insert_bundle(button_image_bundle.clone())
						.insert_bundle(PreviousButtonProperties::default())
						.insert(ButtonAction::Multiplayer)
						.with_children(|parent| {
							parent
								.spawn_bundle(
									TextBundle {
										style: Style {
											margin: Rect::all(TEXT_MARGIN),
											..default()
										},
										text: Text::with_section(
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
											},
											default()
										),
										..default()
									}
								);
						});
					
					// quit button
					parent
						.spawn_bundle(
							ButtonBundle {
								style: Style {
									size: button_size,
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
						.insert_bundle(button_image_bundle.clone())
						.insert_bundle(PreviousButtonProperties::default())
						.insert(ButtonAction::Quit)
						.with_children(|parent| {
							parent
								.spawn_bundle(
									TextBundle {
										style: Style {
											margin: Rect::all(TEXT_MARGIN),
											..default()
										},
										text: Text::with_section(
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
											},
											default()
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
	mut commands: Commands,
) {
	for (interaction, previous_interaction, button_action) in interaction_query.iter() {
		// only execute action if still hovering after click ends
		if *interaction == Interaction::Hovered && *previous_interaction == Interaction::Clicked.into() {
			match button_action {
				// ButtonAction::Singleplayer => commands.insert_resource(NextState(GameState::WorldSelect)), // todo: singleplayer
				ButtonAction::Multiplayer => commands.insert_resource(NextState(GameState::ServerSelect)),
				ButtonAction::Quit => app_exit_events.send(AppExit),
				_ => unimplemented!("{}", button_action),
			}
		}
	}
}
