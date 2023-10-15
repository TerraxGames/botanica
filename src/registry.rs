use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};

use crate::identifier::Identifier;
use crate::registry::def::Definition;

pub mod tile;
pub mod def;

#[derive(Default)]
pub struct Registry<'a, T> where T : Definition<'a> {
	items: HashMap<Identifier<'a>, T>,
}

impl<'a, T> Registry<'a, T> where T : Definition<'a> {
	pub fn new() -> Self {
		Self {
			items: HashMap::new(),
		}
	}

	/// Registers a `T` in the `Registry<T>` with its given `Identifier`.
	pub fn register(&mut self, item: T) {
		self.items.insert(item.identifier(), item);
	}

	/// Returns a `&T` of the given `Identifier`.
	pub fn get(&self, id: Identifier<'a>) -> Option<&T> {
		self.items.get(&id)
	}

	/// Returns a hash of the `Identifier`.
	pub fn get_id_hash(&self, id: Identifier<'a>) -> u64 {
		let mut hasher = self.items.hasher().build_hasher();
		id.hash(&mut hasher);
		hasher.finish()
	}
}
