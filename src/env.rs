use std::fmt;
use std::fmt::Formatter;

use bevy::prelude::*;

#[derive(Debug)]
pub struct EnvTypeFromStringError(String);

impl fmt::Display for EnvTypeFromStringError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Resource)]
/// The environment type.
pub enum EnvType {
	#[default]
	/// A client.<br>
	/// This has client and integrated server logic.
	Client,
	/// A dedicated server.<br>
	/// This has dedicated server logic.
	Server,
}

impl TryFrom<String> for EnvType {
	type Error = EnvTypeFromStringError;
	
	fn try_from(value: String) -> Result<Self, Self::Error> {
		match value.as_str() {
			"client" => Ok(EnvType::Client),
			"server" => Ok(EnvType::Server),
			_ => Err(EnvTypeFromStringError(value)),
		}
	}
}

pub fn is_client(env: Res<EnvType>) -> bool {
	*env == EnvType::Client
}

pub fn is_server(env: Res<EnvType>) -> bool {
	*env == EnvType::Server
}
