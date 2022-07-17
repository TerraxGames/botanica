use crate::identifier::Identifier;

pub trait Definition<'a> {
	fn identifier(&self) -> Identifier<'a>;
}
