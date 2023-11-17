use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};

use crate::identifier::Identifier;
use crate::raw_id::RawId;
use crate::registry::def::Definition;

pub mod tile;
pub mod def;

#[derive(Default)]
pub struct Registry<T> where T : Definition {
	items: HashMap<RawId, T>,
}

impl<T> Registry<T> where T : Definition {
	pub fn new() -> Self {
		Self {
			items: HashMap::new(),
		}
	}

	/// Registers a `T` in the `Registry<T>` with its given `Identifier`.
	pub fn register(&mut self, item: T) {
		self.items.insert(self.get_raw_id(item.identifier()), item);
	}

	/// Returns a `&T` of the given `Identifier`.
	pub fn get(&self, id: &Identifier) -> Option<&T> {
		self.items.get(&self.get_raw_id(id))
	}

	/// Returns a hash of the `Identifier`.
	pub fn get_id_hash(&self, id: &Identifier) -> u64 {
		let mut hasher = self.items.hasher().build_hasher();
		id.hash(&mut hasher);
		hasher.finish()
	}

	/// Returns the raw ID (in the form of a hash) of the `Identifier`
	pub fn get_raw_id(&self, id: &Identifier) -> RawId {
		RawId(self.get_id_hash(id))
	}
}
