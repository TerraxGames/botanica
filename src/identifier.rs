use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Identifier {
	namespace: String,
	id: String,
}

impl Identifier {
	pub fn new(namespace: String, id: String) -> Self {
		Self {
			namespace,
			id,
		}
	}

	pub fn from_str(namespace: &str, id: &str) -> Self {
		Self {
			namespace: namespace.to_string(),
			id: id.to_string(),
		}
	}

	pub fn namespace(&self) -> &str {
		self.namespace.as_str()
	}

	pub fn id(&self) -> &str {
		self.id.as_str()
	}
}

impl Display for Identifier {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}:{}", self.namespace, self.id))
	}
}
