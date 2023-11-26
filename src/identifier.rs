use std::fmt::{Debug, Display, Formatter};

use serde::{Deserialize, de::Visitor};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

impl<'de> Deserialize<'de> for Identifier {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de> {
		struct IdentifierVisitor;
		
		impl<'de> Visitor<'de> for IdentifierVisitor {
			type Value = Identifier;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a &str or String, separated by a \":\"")
			}
			
			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
				where
					E: serde::de::Error, {
				let mut split = v.split(":");
				
				let first = split.next();
				if first.is_none() {
					return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self))
				}
				
				let second = split.next();
				if second.is_none() {
					return Ok(
						Identifier {
							namespace: "null".to_string(), // we do magic later to make these the default namespace
							id: first.unwrap().to_string(),
						}
					)
				}
				
				Ok(
					Identifier {
						namespace: first.unwrap().to_string(),
						id: second.unwrap().to_string(),
					}
				)
			}
			
			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
				where
					E: serde::de::Error, {
				self.visit_str(&v)
			}
		}
		
		deserializer.deserialize_str(IdentifierVisitor)
	}
}

impl Display for Identifier {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}:{}", self.namespace, self.id))
	}
}
