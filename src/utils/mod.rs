use bincode::{DefaultOptions, Error, Options};
use bincode::config::{AllowTrailing, BigEndian, FixintEncoding, LittleEndian, RejectTrailing, WithOtherEndian, WithOtherIntEncoding, WithOtherTrailing};
use once_cell::sync::Lazy;
use regex::Regex;

pub mod sanitize;
pub mod tilehub;
pub mod math;
pub mod asset;

pub static OPTIONS_BE: Lazy<WithOtherTrailing<WithOtherEndian<WithOtherIntEncoding<DefaultOptions, FixintEncoding>, BigEndian>, RejectTrailing>> = Lazy::new(|| {
	DefaultOptions::new()
		.with_fixint_encoding()
		.with_big_endian()
		.reject_trailing_bytes()
});

pub static OPTIONS_LE: Lazy<WithOtherTrailing<WithOtherEndian<WithOtherIntEncoding<DefaultOptions, FixintEncoding>, LittleEndian>, RejectTrailing>> = Lazy::new(|| {
	DefaultOptions::new()
		.with_fixint_encoding()
		.with_little_endian()
		.reject_trailing_bytes()
});

pub static OPTIONS_TRAILING: Lazy<WithOtherTrailing<WithOtherEndian<WithOtherIntEncoding<DefaultOptions, FixintEncoding>, BigEndian>, AllowTrailing>> = Lazy::new(|| {
	DefaultOptions::new()
		.with_fixint_encoding()
		.with_big_endian()
		.allow_trailing_bytes()
});

pub fn serialize_be<T>(ser: &T) -> Result<Vec<u8>, Error>
	where
		T: serde::ser::Serialize,
{
	OPTIONS_BE
		.serialize(ser)
}

pub fn serialize<T>(ser: &T) -> Result<Vec<u8>, Error>
	where
		T: serde::ser::Serialize,
{
	OPTIONS_LE
		.serialize(ser)
}

pub fn serialize_trailing<T>(ser: &T) -> Result<Vec<u8>, Error>
	where
		T: serde::ser::Serialize,
{
	OPTIONS_TRAILING
		.serialize(ser)
}

pub fn deserialize_be<'a, T>(bytes: &'a [u8]) -> Result<T, Error>
	where
		T: serde::de::Deserialize<'a>,
{
	OPTIONS_BE
		.deserialize(bytes)
}

pub fn deserialize<'a, T>(bytes: &'a [u8]) -> Result<T, Error>
	where
		T: serde::de::Deserialize<'a>,
{
	OPTIONS_LE
		.deserialize(bytes)
}

pub fn deserialize_trailing<'a, T>(bytes: &'a [u8]) -> Result<T, Error>
	where
		T: serde::de::Deserialize<'a>,
{
	OPTIONS_TRAILING
		.deserialize(bytes)
}

pub fn strip_formatting(msg: &str) -> String {
	let re = Regex::new("`[a-zA-Z0-9]").unwrap();
	re.replace_all(msg, "").to_string()
}

// cursed
pub trait NewType {
	type Inner;
}

/// Automagically pipes systems that can return errors.
macro_rules! nonfatal_error_systems {
    ( $error_msg:expr, $error:ty, $( $system_i:ident ),+ ) => {
		{
			use bevy::prelude::In;
			fn __handle_errors__(In(result): In<Result<(), $error>>) {
				if let Err(error) = result {
					eprintln!("{}: {}", $error_msg, error);
				}
			}
			
			($(
				$system_i.pipe(__handle_errors__),
			)+)
		}
	};
}

pub(crate) use nonfatal_error_systems;

/// The standard [`HashMap`](std::collections::HashMap).
pub type StdHashMap<K, V, S = std::collections::hash_map::RandomState> = std::collections::HashMap<K, V, S>;

/// Bevy's [`HashMap`](bevy::utils::hashbrown::HashMap) that implements "aHash, a high speed keyed hashing algorithm intended for use in in-memory hashmaps."
/// 
/// Quoth the docs, "aHash is designed for performance and is NOT cryptographically secure."
pub type BevyHashMap<K, V> = bevy::utils::HashMap<K, V>;

macro_rules! fatal_error_systems {
    ( $error_msg:expr, $error:ty, $( $system_i:ident ),+ ) => {
		{
			use bevy::prelude::In;
			fn __handle_errors__(In(result): In<Result<(), $error>>) {
				if let Err(error) = result {
					panic!("{}: {}", $error_msg, error);
				}
			}
			
			($(
				$system_i.pipe(__handle_errors__),
			)+)
		}
	};
}

pub(crate) use fatal_error_systems;

macro_rules! struct_enforce {
    ($param:expr, $($ty:path),+) => {
		{
			trait Enforce {
				fn enforce(&self);
			}
			
			$(
				impl Enforce for $ty {
					fn enforce(&self) {}
				}
			)+
			
			Enforce::enforce(&$param);
		}
	};
}

pub(crate) use struct_enforce;

macro_rules! trait_enforce {
    ($param:expr, $($ty:path),+) => {
		{
			$(
				fn enforce<T>(_t: &T)
				where T: $ty
				{}
				
				enforce(&$param);
			)+
		}
	};
}

pub(crate) use trait_enforce;
