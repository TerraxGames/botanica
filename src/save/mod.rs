use std::fs::File;
use std::io::Read;
use crate::save::format::WorldSave;
use crate::util::sanitize::sanitize_alphanumeric_dash;

pub mod format;

pub fn open_world(name: &str) -> std::io::Result<WorldSave> {
	let path = std::env::current_dir().unwrap().with_file_name(format!("saves/worlds/{}.dat", /* just in case ;) */ sanitize_alphanumeric_dash(name)));
	if path.exists() {
		let mut file = File::open(path)?;
		if file.metadata().unwrap().is_file() {
			let mut vec = vec![];
			file.read_to_end(&mut vec)?;
			Ok(WorldSave::deserialize(vec))
		} else {
			Ok(WorldSave::default())
		}
	} else {
		Ok(WorldSave::default())
	}
}
