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
	ids: HashMap<Identifier, RawId>,
}

impl<T> Registry<T> where T : Definition {
	pub fn new() -> Self {
		Self {
			items: HashMap::new(),
			ids: HashMap::new(),
		}
	}
	
	/// Registers a `T` in the `Registry<T>` with its given `Identifier`.
	pub fn register(&mut self, item: T, identifier: Identifier) {
		let raw_id = RawId(self.ids.len() as u32);
		self.items.insert(raw_id, item);
		self.ids.insert(identifier, raw_id);
	}
	
	/// Returns a `&T` of the given `Identifier`.
	pub fn get(&self, id: &Identifier) -> Option<&T> {
		self.items.get(&self.get_raw_id(id)?)
	}
	
	/// Returns the raw ID of the `Identifier`
	pub fn get_raw_id(&self, id: &Identifier) -> Option<RawId> {
		Some(*self.ids.get(id)?)
	}
}
