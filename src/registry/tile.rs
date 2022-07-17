use crate::i18n::Translatable;
use crate::identifier::Identifier;
use crate::Registry;
use crate::registry::def::Definition;

pub struct TileRegistry<'a>(pub Registry<'a, TileDef<'a>>);

impl<'a> Default for TileRegistry<'a> {
	fn default() -> Self {
		Self(Registry::new())
	}
}

pub struct TileDef<'a> {
	identifier: Identifier<'a>,
	name: Translatable,
}

impl<'a> TileDef<'a> {
	pub fn new(identifier: Identifier<'a>) -> Self {
		Self {
			identifier,
			name: Translatable::new(format!("{}:tile.name.{}", identifier.namespace(), identifier.id()).as_str()),
		}
	}

	pub fn name(&self) -> &Translatable {
		&self.name
	}
}

impl<'a> Definition<'a> for TileDef<'a> {
	fn identifier(&self) -> Identifier<'a> {
		self.identifier
	}
}
