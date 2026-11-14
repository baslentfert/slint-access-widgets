//! Accessible widgets for [Slint](https://slint.dev), written in `.slint`,
//! with a few Rust helpers.
//!
//! In `build.rs`, make the widgets importable as `@access-widgets`:
//!
//! ```no_run
//! let config = slint_build::CompilerConfiguration::new()
//!     .with_library_paths(slint_access_widgets::library_paths());
//! slint_build::compile_with_config("ui/app.slint", config).unwrap();
//! ```
//!
//! and in your `.slint` file:
//!
//! ```slint,ignore
//! import { AccessTable } from "@access-widgets";
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

use slint::{Model, StandardListViewItem};

/// The library path for `slint_build`: `@access-widgets` → this crate's `ui/lib.slint`.
pub fn library_paths() -> HashMap<String, PathBuf> {
    HashMap::from([("access-widgets".to_string(), PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/lib.slint")))])
}

/// Type-ahead for `AccessTable`: the first row at or after `from` whose
/// first cell starts with `prefix` (ignoring case), as index + 1; 0 if none.
///
/// ```ignore
/// let rows = model.clone();
/// window.on_find(move |prefix, from| slint_access_widgets::find_prefix(&rows, &prefix, from));
/// ```
pub fn find_prefix<R>(rows: &impl Model<Data = R>, prefix: &str, from: i32) -> i32
where
    R: Model<Data = StandardListViewItem>,
{
    find_prefix_by(rows.row_count(), prefix, from, |i| rows.row_data(i).and_then(|r| r.row_data(0)).map(|c| c.text.to_string()))
}

/// Type-ahead with your own text per row (index + 1; 0 if none).
pub fn find_prefix_by(count: usize, prefix: &str, from: i32, text: impl Fn(usize) -> Option<String>) -> i32 {
    let prefix = prefix.to_lowercase();
    if prefix.is_empty() {
        return 0;
    }
    let from = from.max(0) as usize;
    (from..count)
        .find(|&i| text(i).is_some_and(|t| t.to_lowercase().starts_with(&prefix)))
        .map_or(0, |i| i as i32 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_ahead_finds_from_position() {
        let names = ["apple", "banana", "Blueberry", "cherry"];
        let t = |i: usize| names.get(i).map(|s| s.to_string());
        assert_eq!(find_prefix_by(4, "b", 0, t), 2);
        assert_eq!(find_prefix_by(4, "b", 2, t), 3);
        assert_eq!(find_prefix_by(4, "bl", 0, t), 3);
        assert_eq!(find_prefix_by(4, "x", 0, t), 0);
        assert_eq!(find_prefix_by(4, "", 0, t), 0);
    }
}
