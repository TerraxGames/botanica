use std::ops::Mul;
use bevy::prelude::*;
use crate::state::menu::{HOVERED_BUTTON, NORMAL_BUTTON};

#[derive(Component)]
pub struct ButtonColor(pub Color);

impl From<Color> for ButtonColor {
	fn from(color: Color) -> Self {
		Self(color)
	}
}

impl Into<Color> for ButtonColor {
	fn into(self) -> Color {
		self.0
	}
}

impl Into<UiColor> for ButtonColor {
	fn into(self) -> UiColor {
		self.0.into()
	}
}

impl Mul for &ButtonColor {
	type Output = ButtonColor;

	fn mul(self, rhs: Self) -> Self::Output {
		ButtonColor(Color::rgba(self.0.r() * rhs.0.r(), self.0.g() * rhs.0.g(), self.0.b() * rhs.0.b(), self.0.a() * rhs.0.a()))
	}
}

impl Mul for ButtonColor {
	type Output = ButtonColor;

	fn mul(self, rhs: Self) -> Self::Output {
		<&ButtonColor>::mul(&self, &rhs)
	}
}

#[derive(Component, Clone, Default)]
pub struct ButtonDownImage(pub Handle<Image>);

impl From<Handle<Image>> for ButtonDownImage {
	fn from(image: Handle<Image>) -> Self {
		Self(image.into())
	}
}

#[derive(Component, Clone, Default)]
pub struct ButtonUpImage(pub Handle<Image>);

impl From<Handle<Image>> for ButtonUpImage {
	fn from(image: Handle<Image>) -> Self {
		Self(image.into())
	}
}

#[derive(Bundle, Clone, Default)]
pub struct ButtonImageBundle {
	pub button_up: ButtonUpImage,
	pub button_down: ButtonDownImage,
	pub previous_button_interaction: PreviousButtonInteraction,
}

#[derive(Component, Copy, Clone, Default, PartialEq, Eq)]
pub struct PreviousButtonInteraction(pub Interaction);

impl From<Interaction> for PreviousButtonInteraction {
	fn from(interaction: Interaction) -> Self {
		Self(interaction)
	}
}

#[derive(Component, Copy, Clone, Default)]
pub struct PreviousButtonBottomPadding(pub Val);

impl From<Val> for PreviousButtonBottomPadding {
	fn from(padding: Val) -> Self {
		Self(padding)
	}
}

impl Into<Val> for PreviousButtonBottomPadding {
	fn into(self) -> Val {
		self.0
	}
}

#[derive(Default)]
pub enum ButtonType {
	#[default]
	Wide,
	Square,
}

#[derive(Bundle, Clone, Default)]
pub struct PreviousButtonProperties {
	pub previous_interaction: PreviousButtonInteraction,
	pub previous_bottom_padding: PreviousButtonBottomPadding,
}

/// Handles button style when interacted
pub fn style(
	mut interaction_query: Query<
		(&Interaction, &mut PreviousButtonInteraction, &mut UiColor, &ButtonColor, &mut UiImage, &ButtonUpImage, &ButtonDownImage, &mut Style, &mut PreviousButtonBottomPadding),
		(Changed<Interaction>, With<Button>),
	>,
) {
	for (interaction, mut previous_interaction, mut color, button_color, mut image, button_up, button_down, mut style, mut previous_bottom_padding) in interaction_query.iter_mut() {
		match *interaction {
			Interaction::Clicked => {
				*color = (button_color * &HOVERED_BUTTON.into()).into();
				*image = button_down.0.clone_weak().into();
				*previous_interaction = Interaction::Clicked.into();
				*previous_bottom_padding = style.padding.bottom.into();
				style.padding.bottom = Val::Px(0.0);
			},
			Interaction::Hovered => {
				*color = (button_color * &HOVERED_BUTTON.into()).into();
				if *previous_interaction == Interaction::Clicked.into() {
					*image = button_up.0.clone_weak().into();
					style.padding.bottom = previous_bottom_padding.clone().into();
				}
			},
			Interaction::None => {
				*color = (button_color * &NORMAL_BUTTON.into()).into();
				*image = button_up.0.clone_weak().into();
				if *previous_interaction == Interaction::Clicked.into() {
					style.padding.bottom = previous_bottom_padding.clone().into();
				}
				*previous_interaction = Interaction::None.into();
			},
		}
	}
}
