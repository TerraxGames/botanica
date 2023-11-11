use bincode::{DefaultOptions, Error, Options};
use bincode::config::{AllowTrailing, BigEndian, FixintEncoding, RejectTrailing, WithOtherEndian, WithOtherIntEncoding, WithOtherTrailing};
use once_cell::sync::Lazy;
use regex::Regex;

static OPTIONS: Lazy<WithOtherTrailing<WithOtherEndian<WithOtherIntEncoding<DefaultOptions, FixintEncoding>, BigEndian>, RejectTrailing>> = Lazy::new(|| {
	DefaultOptions::new()
		.with_fixint_encoding()
		.with_big_endian()
		.reject_trailing_bytes()
});

static OPTIONS_TRAILING: Lazy<WithOtherTrailing<WithOtherEndian<WithOtherIntEncoding<DefaultOptions, FixintEncoding>, BigEndian>, AllowTrailing>> = Lazy::new(|| {
	DefaultOptions::new()
		.with_fixint_encoding()
		.with_big_endian()
		.allow_trailing_bytes()
});

pub fn serialize<T>(ser: &T) -> Result<Vec<u8>, Error>
	where
		T: serde::ser::Serialize,
{
	OPTIONS
		.serialize(ser)
}

pub fn serialize_trailing<T>(ser: &T) -> Result<Vec<u8>, Error>
	where
		T: serde::ser::Serialize,
{
	OPTIONS_TRAILING
		.serialize(ser)
}

pub fn deserialize<'a, T>(bytes: &'a [u8]) -> Result<T, Error>
	where
		T: serde::de::Deserialize<'a>,
{
	OPTIONS
		.deserialize(bytes)
}

pub fn deserialize_trailing<'a, T>(bytes: &'a [u8]) -> Result<T, Error>
	where
		T: serde::de::Deserialize<'a>,
{
	OPTIONS_TRAILING
		.deserialize(bytes)
}

pub fn strip_formatting(msg: String) -> String {
	let re = Regex::new("`.").unwrap();
	re.replace_all(&*msg, "").to_string()
}

// cursed
pub trait NewType {
	type Inner;
}

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
				fn enforce<T>(t: &T)
				where T: $ty
				{}
				
				enforce(&$param);
			)+
		}
	};
}

pub(crate) use trait_enforce;
