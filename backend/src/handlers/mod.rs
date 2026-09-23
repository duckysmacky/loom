pub mod auth;
pub mod board;
pub mod checklist;
pub mod dashboard;
pub mod edges;
pub mod error;
pub mod extract;
pub mod nodes;
pub mod pokes;
pub mod topics;

use self::error::ApiError;

/// Validates a node/topic accent color - `#rgb` or `#rrggbb`, matching
/// what a plain HTML `<input type="color">` produces. Storage is a bare
/// `text` column (no CHECK constraint), so this is the only gate before
/// the value reaches the frontend and gets used directly as a CSS value.
pub(crate) fn validate_color(color: &str) -> Result<(), ApiError> {
    let hex = color.strip_prefix('#').ok_or(ApiError::InvalidInput(
        "color must be a hex code like #a3c9ff",
    ))?;
    let valid_len = hex.len() == 3 || hex.len() == 6;
    if valid_len && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ApiError::InvalidInput(
            "color must be a hex code like #a3c9ff",
        ))
    }
}
