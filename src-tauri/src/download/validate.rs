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
///
/// Fix #3: the old code used `splitn(2, '/')` which stopped splitting after the
/// first slash, so `owner/repo/extra` produced parts `["owner", "repo/extra"]`.
/// `"repo/extra"` passed all checks because it contains no `..`, `\`, or `\0`.
/// We now explicitly require exactly one `/` and verify neither part contains
/// a nested slash, keeping the expected `owner/repo` shape.
pub fn sanitise_model_id(id: &str) -> Result<String, String> {
    // Require exactly one '/' separator — no more, no less.
    let slash_count = id.chars().filter(|&c| c == '/').count();
    if slash_count != 1 {
        return Err(format!("Invalid model ID (expected 'owner/repo'): '{}'", id));
    }

    // Safe to unwrap: we just confirmed there is exactly one '/'.
    let (owner, repo) = id.split_once('/').unwrap();

    for (label, part) in [("owner", owner), ("repo", repo)] {
        if part.is_empty() {
            return Err(format!("Invalid model ID ({} is empty): '{}'", label, id));
        }
        // After split_once('/'), neither part can contain '/', so the only
        // traversal value possible here is the literal string "..".
        if part == ".." {
            return Err(format!("Invalid model ID ({} traversal): '{}'", label, id));
        }
        if part.contains('\\') || part.contains('\0') {
            return Err(format!("Invalid model ID ({} contains illegal character): '{}'", label, id));
        }
    }

    Ok(id.replace('/', "__"))
}