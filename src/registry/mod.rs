use crate::utils::BevyHashMap;

use bevy::{asset::{Handle, Asset}, reflect::{TypeUuid, TypePath}};

use crate::identifier::Identifier;

pub mod tile;
pub mod def;

/// Maps identifiers to handles of a specific type.
/// For example, this is useful when retrieving a tile definition from its identifier.
#[derive(Default)]
pub struct Registry<T>
	where T: TypeUuid + TypePath + Asset {
	handles: BevyHashMap<Identifier, Handle<T>>,
}

impl<T> Registry<T>
	where T: TypeUuid + TypePath + Asset {
	pub fn new() -> Self {
		Self {
			handles: BevyHashMap::new(),
		}
	}
	
	/// Registers a `Handle<T>` in the `Registry<T>` with its given `Identifier`.
	pub fn register(&mut self, item: Handle<T>, identifier: Identifier) {
		self.handles.insert(identifier, item);
	}
	
	/// Returns a *strong* `Handle<T>` of the given `Identifier`.
	pub fn get(&self, id: &Identifier) -> Option<Handle<T>> {
		Some(self.handles.get(id)?.clone())
	}
}
