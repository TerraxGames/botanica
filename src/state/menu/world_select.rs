use std::fmt;
use std::fmt::Formatter;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_egui::egui::style::Margin;
use renet::{DefaultChannel, RenetClient};
use renet::transport::NetcodeClientTransport;

use crate::i18n::{CurrentLocale, TranslationServer};
use crate::world::{ClientGameWorld, WorldId};
use crate::{asset, DEFAULT_LOCALE, despawn_with, from_asset_loc, GameState, LocaleAsset, menu, NAMESPACE, Translatable};
use crate::menu::{BACKGROUND, BUTTON_BOTTOM_PADDING, BUTTON_HEIGHT, BUTTON_SCALE, BUTTON_TEXT_SIZE, BUTTON_WIDTH, NORMAL_BUTTON, TEXT_MARGIN};
use crate::menu::button::{ButtonColor, ButtonDownImage, ButtonImageBundle, ButtonUpImage, PreviousButtonInteraction, PreviousButtonProperties};
use crate::client;
use crate::client::networking::disconnect;
use crate::networking::{DisconnectReason, protocol};
use crate::networking::error::{NetworkError, NETWORK_ERROR_MESSAGE};
use crate::utils::nonfatal_error_systems;

#[derive(Resource, Default)]
struct WorldSelection(String);

pub struct WorldSelectPlugin;

impl Plugin for WorldSelectPlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<WorldSelection>()
			.add_systems(OnEnter(GameState::WorldSelect), setup)
			.add_systems(OnExit(GameState::WorldSelect), despawn_with::<OnWorldSelect>)
			.add_systems(
				Update,
				(
					menu::button::style,
					nonfatal_error_systems!(NETWORK_ERROR_MESSAGE, NetworkError, button_action),
					text_box,
				)
					.run_if(in_state(GameState::WorldSelect))
			);
	}
}

#[derive(Component)]
struct OnWorldSelect;

#[derive(Debug, Component)]
enum ButtonAction {
	Enter,
	Cancel,
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
	current_locale: Res<CurrentLocale>,
	translation_server: Res<TranslationServer>,
) {
	let monogram = asset_server.get_handle(from_asset_loc(NAMESPACE, "fonts/monogram/monogram-extended.ttf"));
	let button_up = ButtonUpImage::from(asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/button/button_up.png")));
	let button_down = ButtonDownImage::from(asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/button/button_down.png")));
	
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
				background_color: BACKGROUND.into(),
				..default()
			}
		)
		.insert(OnWorldSelect)
		.with_children(|parent| {
			// select a world
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
							translation_server.translate(
								NAMESPACE,
								"ui.world_select.text.world_select",
								&current_locale,
							).unwrap(),
							TextStyle {
								font: monogram.clone_weak(),
								font_size: 45.0,
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
					// enter button
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
						.insert(ButtonAction::Enter)
						.with_children(|parent| {
							parent
								.spawn(
									TextBundle {
										style: Style {
											margin: UiRect::all(TEXT_MARGIN),
											..default()
										},
										text: Text::from_section(
											translation_server.translate(
												NAMESPACE,
												"ui.world_select.button.enter",
												&current_locale,
											).unwrap(),
											TextStyle {
												font: monogram.clone(),
												font_size: BUTTON_TEXT_SIZE,
												color: Color::BLACK,
											},
										),
										..default()
									}
								);
						});
					
					// cancel button
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
						.insert(ButtonAction::Cancel)
						.with_children(|parent| {
							parent
								.spawn(
									TextBundle {
										style: Style {
											margin: UiRect::all(TEXT_MARGIN),
											..default()
										},
										text: Text::from_section(
											translation_server.translate(
												NAMESPACE,
												"ui.world_select.button.cancel",
												&current_locale,
											).unwrap(),
											TextStyle {
												font: monogram.clone(),
												font_size: BUTTON_TEXT_SIZE,
												color: Color::BLACK,
											},
										),
										..default()
									}
								);
						});
				});
		});
}

fn text_box(
	mut contexts: EguiContexts,
	asset_server: Res<AssetServer>,
	locale_assets: Res<Assets<LocaleAsset>>,
	mut world_name: ResMut<WorldSelection>,
	current_locale: Res<CurrentLocale>,
	translation_server: Res<TranslationServer>,
) {
	let button_up = ButtonUpImage::from(asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/button/button_up.png")));
	let button_down = ButtonDownImage::from(asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/button/button_down.png")));
	
	let button_size = Vec2::new(BUTTON_WIDTH, BUTTON_HEIGHT) * BUTTON_SCALE;
	let button_margin = Margin::default();
	let button_padding = UiRect {
		bottom: Val::Px(BUTTON_BOTTOM_PADDING * BUTTON_SCALE),
		..default()
	};
	
	egui::Window::new(
		translation_server.translate(
			NAMESPACE,
			"ui.world_select.window.title.world_name",
			&current_locale,
		).unwrap()
	).show(contexts.ctx(), |ui| {
		ui.text_edit_singleline(&mut world_name.0);
	});
}

fn button_action(
	interaction_query: Query<
		(&Interaction, &PreviousButtonInteraction, &ButtonAction),
		(Changed<Interaction>, With<Button>),
	>,
	mut transport: ResMut<NetcodeClientTransport>,
	mut client: ResMut<RenetClient>,
	mut next_state: ResMut<NextState<GameState>>,
	world_name: Res<WorldSelection>,
	mut commands: Commands,
) -> Result<(), NetworkError> {
	for (interaction, previous_interaction, button_action) in interaction_query.iter() {
		if *interaction == Interaction::Hovered && *previous_interaction == Interaction::Pressed.into() {
			match button_action {
				ButtonAction::Enter => {
					commands.insert_resource(
						ClientGameWorld {
							id: WorldId(world_name.0.clone()),
							tiles: default(),
							tile_sprites: default(),
						}
					);
					client::networking::send_message!(client, DefaultChannel::ReliableOrdered, protocol::ClientMessage::EnterWorldRequest(world_name.0.clone()));
				},
				ButtonAction::Cancel => {
					disconnect(DisconnectReason::Client(renet::DisconnectReason::DisconnectedByClient), &mut transport, &mut client, true);
					next_state.set(GameState::TitleScreen);
				},
				_ => unimplemented!("{}", button_action),
			}
		}
	}
	
	Ok(())
}
