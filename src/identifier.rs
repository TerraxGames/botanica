use std::fmt::{Debug, Display, Formatter};

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Identifier<'a> {
	namespace: &'a str,
	id: &'a str,
}

impl<'a> Identifier<'a> {
	pub fn new(namespace: &'a str, id: &'a str) -> Self {
		Self {
			namespace,
			id,
		}
	}

	pub fn namespace(&self) -> &str {
		self.namespace
	}

	pub fn id(&self) -> &str {
		self.id
	}
}

impl<'a> Debug for Identifier<'a> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}:{}", self.namespace, self.id))
	}
}

impl<'a> Display for Identifier<'a> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		Debug::fmt(self, f)
	}
}
