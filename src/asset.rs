pub mod locale;

/// Gets the proper location of the asset at the given location with format `<namespace>:<asset>`
pub fn from_asset_loc_raw(location: String) -> String {
	location.replacen(":", "/", 1)
}

/// Gets the proper location of the asset at the given location in the given namespace.
pub fn from_asset_loc(namespace: &str, location: &str) -> String {
	format!("{}/{}", namespace, location)
}

/// Gets the namespace and item from a given namespaced item with format `<namespace>:<item>`.
pub fn parse_namespaced(item: &str) -> (&str, &str) {
	item.split_once(':').unwrap()
}

/// Translates the item into a namespaced item with format `<namespace>:<item>`
pub fn namespaced(namespace: &str, item: &str) -> String {
	format!("{}:{}", namespace, item)
}
