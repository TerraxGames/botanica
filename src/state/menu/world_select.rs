use std::fmt;
use std::fmt::Formatter;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use bevy_egui::egui::style::Margin;
use iyes_loopless::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;
use renet::RenetClient;
use crate::{asset, DEFAULT_LOCALE, despawn_with, from_asset_loc, GameState, LocaleAsset, menu, NAMESPACE, ServerAddressPort, Translatable};
use crate::menu::{BACKGROUND, BUTTON_BOTTOM_PADDING, BUTTON_HEIGHT, BUTTON_SCALE, BUTTON_TEXT_SIZE, BUTTON_WIDTH, NORMAL_BUTTON, TEXT_MARGIN};
use crate::menu::button::{ButtonColor, ButtonDownImage, ButtonImageBundle, ButtonUpImage, PreviousButtonInteraction, PreviousButtonProperties};
use crate::server::ServerAddress;

struct WorldSelection(String);

pub struct WorldSelectPlugin;

impl Plugin for WorldSelectPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_enter_system(GameState::WorldSelect, setup)
			.add_exit_system(GameState::WorldSelect, despawn_with::<OnWorldSelect>)
			.add_system(
				menu::button::style
					.run_in_state(GameState::WorldSelect)
			)
			.add_system(
				button_action
					.run_in_state(GameState::WorldSelect)
			)
			.add_system(
				text_box
					.run_in_state(GameState::WorldSelect)
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
) {
	let monogram = asset_server.get_handle(from_asset_loc(NAMESPACE, "fonts/monogram/monogram-extended.ttf"));
	let button_up = ButtonUpImage::from(asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/button/button_up.png")));
	let button_down = ButtonDownImage::from(asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/button/button_down.png")));
	
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
				color: BACKGROUND.into(),
				..default()
			}
		)
		.insert(OnWorldSelect)
		.with_children(|parent| {
			// select a world
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
								asset::namespaced(NAMESPACE, "ui.world_select.text.world_select").as_str(),
								DEFAULT_LOCALE,
								&asset_server,
								&locale_assets,
							),
							TextStyle {
								font: monogram.clone_weak(),
								font_size: 45.0,
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
					// enter button
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
						.insert(ButtonAction::Enter)
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
												asset::namespaced(NAMESPACE, "ui.world_select.button.enter").as_str(),
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
					
					// cancel button
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
						.insert(ButtonAction::Cancel)
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
												asset::namespaced(NAMESPACE, "ui.world_select.button.cancel").as_str(),
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

fn text_box(
	mut gui_ctx: ResMut<EguiContext>,
	asset_server: Res<AssetServer>,
	locale_assets: Res<Assets<LocaleAsset>>,
	mut world_name: ResMut<WorldSelection>,
) {
	let button_up = ButtonUpImage::from(asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/button/button_up.png")));
	let button_down = ButtonDownImage::from(asset_server.get_handle(from_asset_loc(NAMESPACE, "textures/ui/button/button_down.png")));
	
	let button_size = Vec2::new(BUTTON_WIDTH, BUTTON_HEIGHT) * BUTTON_SCALE;
	let button_margin = Margin::default();
	let button_padding = Rect {
		bottom: Val::Px(BUTTON_BOTTOM_PADDING * BUTTON_SCALE),
		..default()
	};
	
	egui::Window::new(Translatable::translate_once(
		asset::namespaced(NAMESPACE, "ui.world_select.window.title.world_name").as_str(),
		DEFAULT_LOCALE,
		&asset_server,
		&locale_assets,
	)).show(gui_ctx.ctx_mut(), |ui| {
		ui.text_edit_singleline(&mut world_name.0);
	});
}

fn button_action(
	interaction_query: Query<
		(&Interaction, &PreviousButtonInteraction, &ButtonAction),
		(Changed<Interaction>, With<Button>),
	>,
	mut client: ResMut<RenetClient>,
	mut commands: Commands,
) {
	for (interaction, previous_interaction, button_action) in interaction_query.iter() {
		if *interaction == Interaction::Hovered && *previous_interaction == Interaction::Clicked.into() {
			match button_action {
				ButtonAction::Enter => commands.insert_resource(NextState(GameState::LoadingWorld)),
				ButtonAction::Cancel => {
					client.disconnect();
					commands.insert_resource(NextState(GameState::TitleScreen));
				},
				_ => unimplemented!("{}", button_action),
			}
		}
	}
}
