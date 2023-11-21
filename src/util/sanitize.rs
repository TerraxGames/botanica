use regex::Regex;

/// Sanitizes a string, allowing only alphanumeric characters, dashes, and underscores.
pub fn sanitize_alphanumeric_dash(input: &str) -> String {
	let regex = Regex::new(r"[^a-zA-Z0-9-_]+").unwrap();
	regex.replace_all(input, "").to_string()
}

/// Sanitizes a string, allowing only alphanumeric characters.
pub fn sanitize_alphanumeric(input: &str) -> String {
	let regex = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
	regex.replace_all(input, "").to_string()
}
