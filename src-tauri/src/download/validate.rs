use std::path::Path;

/// Reject filenames with path components or control characters.
/// Only .gguf files are accepted.
pub fn sanitise_filename(name: &str) -> Result<(), String> {
    let p = Path::new(name);
    if p.components().count() != 1 {
        return Err(format!("Invalid filename: '{}'", name));
    }
    if name.chars().any(|c| c.is_control()) {
        return Err(format!("Invalid filename: '{}'", name));
    }
    if !name.ends_with(".gguf") {
        return Err("Only .gguf files are supported".to_string());
    }
    Ok(())
}

/// Reject model IDs that could produce a path traversal after '/' → "__" substitution.
pub fn sanitise_model_id(id: &str) -> Result<String, String> {
    let parts: Vec<&str> = id.splitn(2, '/').collect();
    for part in &parts {
        if part.is_empty() || *part == ".." || part.contains('\\') || part.contains('\0') {
            return Err(format!("Invalid model ID: '{}'", id));
        }
    }
    Ok(id.replace('/', "__"))
}