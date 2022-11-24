use std::fmt;
use std::fmt::Formatter;
use regex::Regex;
use renet::NETCODE_USER_DATA_BYTES;
use serde::{Serialize, Deserialize};
use bevy::prelude::*;

pub mod protocol;
pub mod client;
pub mod server;
pub mod debug;

pub const USERNAME_BYTES: usize = 32;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct Ping(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Component)]
pub struct Username(pub String);

impl fmt::Display for Username {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl Username {
	pub fn to_user_data(&self) -> [u8; NETCODE_USER_DATA_BYTES] {
		let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
		
		if self.0.len() > USERNAME_BYTES {
			panic!("Username too big (maximum is {}, found {})", USERNAME_BYTES, self.0.len());
		}
		
		user_data[0..4].copy_from_slice(&(self.0.len() as u32).to_le_bytes());
		user_data[4..self.0.len() + 4].copy_from_slice(self.0.as_bytes());
		
		user_data
	}
	
	pub fn from_user_data(user_data: &[u8; NETCODE_USER_DATA_BYTES]) -> Self {
		let mut buf = [0u8; 4];
		buf.copy_from_slice(&user_data[0..4]);
		let len = u32::from_le_bytes(buf) as usize;
		
		let mut buf = Vec::new();
		buf.extend_from_slice(&user_data[4..len + 4]);
		
		Self(String::from_utf8(buf).unwrap())
	}
}

pub fn strip_formatting(msg: String) -> String {
	let re = Regex::new("`.").unwrap();
	re.replace_all(&*msg, "").to_string()
}
