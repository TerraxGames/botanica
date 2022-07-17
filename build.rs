use std::env;

fn main() {
	let profile = env::var("PROFILE").unwrap();
	if profile.as_str() == "debug" {
		println!("cargo:rustc-cfg=feature={:?}", "debug");
		println!("cargo:rustc-cfg=feature={:?}", "fast_bevy_splash");
	}
	println!("cargo:rerun-if-changed=build.rs");
}
