use crate::identifier::Identifier;

pub trait Definition {
	fn identifier(&self) -> &Identifier;
}
