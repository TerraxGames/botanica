/// Sanitizes a string, allowing only alphanumeric characters, dashes, and underscores.
pub fn sanitize_alphanumeric_dash(input: &str) -> String {
	input.replace(r"[^a-zA-Z0-9-_]+", "")
}

/// Sanitizes a string, allowing only alphanumeric characters.
pub fn sanitize_alphanumeric(input: &str) -> String {
	input.replace(r"[^a-zA-Z0-9]+", "")
}
