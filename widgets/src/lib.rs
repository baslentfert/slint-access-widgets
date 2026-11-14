//! Accessible widgets for [Slint](https://slint.dev), written in `.slint`,
//! with a few Rust helpers.
//!
//! In `build.rs`, make the widgets importable as `@access-widgets`:
//!
//! ```ignore
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

use std::cell::Cell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use slint::{Model, ModelRc, StandardListViewItem, VecModel};

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

/// The selection of an `AccessTable` with `multi-select: true`, as in File
/// Explorer: click or arrows select one row, Shift extends from the anchor,
/// Ctrl+click or Space adds or removes a row, Ctrl+A selects all.
///
/// ```ignore
/// let sel = Rc::new(Selection::new(rows.row_count()));
/// window.set_selected(sel.model());
/// let s = sel.clone();
/// window.on_select(move |row, how| s.apply(row, how));
/// ```
///
/// Rows are reused while scrolling, so the selection lives here, not in the
/// widgets. Call [`Selection::reset`] when the rows change (sort, filter).
pub struct Selection {
    model: Rc<VecModel<bool>>,
    anchor: Cell<usize>,
    count: Cell<usize>,
}

impl Selection {
    pub const ONLY: i32 = 0;
    pub const TOGGLE: i32 = 1;
    pub const EXTEND: i32 = 2;
    pub const ALL: i32 = 3;
    pub const CLEAR: i32 = 4;

    pub fn new(rows: usize) -> Self {
        Self { model: Rc::new(VecModel::from(vec![false; rows])), anchor: Cell::new(0), count: Cell::new(0) }
    }

    /// For `selected-rows`.
    pub fn model(&self) -> ModelRc<bool> {
        self.model.clone().into()
    }

    /// New rows: nothing selected.
    pub fn reset(&self, rows: usize) {
        self.model.set_vec(vec![false; rows]);
        self.anchor.set(0);
        self.count.set(0);
    }

    /// Applies a change from the table (`row`, `how` = one of the constants)
    /// and returns how many rows are selected.
    pub fn apply(&self, row: i32, how: i32) -> i32 {
        let n = self.model.row_count();
        let row = (row.max(0) as usize).min(n.saturating_sub(1));
        let set = |i: usize, v: bool| {
            if self.model.row_data(i) != Some(v) {
                self.model.set_row_data(i, v);
                self.count.set(if v { self.count.get() + 1 } else { self.count.get() - 1 });
            }
        };
        match how {
            Self::TOGGLE if n > 0 => {
                set(row, !self.model.row_data(row).unwrap_or(false));
                self.anchor.set(row);
            }
            Self::EXTEND if n > 0 => {
                let (a, b) = (self.anchor.get().min(row), self.anchor.get().max(row));
                for i in 0..n {
                    set(i, (a..=b).contains(&i));
                }
            }
            Self::ALL => (0..n).for_each(|i| set(i, true)),
            Self::CLEAR => (0..n).for_each(|i| set(i, false)),
            _ if n > 0 => {
                for i in 0..n {
                    set(i, i == row);
                }
                self.anchor.set(row);
            }
            _ => {}
        }
        self.count.get() as i32
    }

    /// The selected rows, in order.
    pub fn rows(&self) -> Vec<usize> {
        self.model.iter().enumerate().filter_map(|(i, v)| v.then_some(i)).collect()
    }
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

    #[test]
    fn selection_like_file_explorer() {
        let s = Selection::new(10);
        assert_eq!(s.apply(2, Selection::ONLY), 1);
        assert_eq!(s.apply(5, Selection::EXTEND), 4);
        assert_eq!(s.rows(), [2, 3, 4, 5]);
        // Shift back up from the same anchor.
        assert_eq!(s.apply(0, Selection::EXTEND), 3);
        assert_eq!(s.apply(8, Selection::TOGGLE), 4);
        assert_eq!(s.apply(8, Selection::TOGGLE), 3);
        assert_eq!(s.apply(0, Selection::ALL), 10);
        assert_eq!(s.apply(0, Selection::CLEAR), 0);
        s.reset(3);
        assert_eq!(s.apply(7, Selection::ONLY), 1);
        assert_eq!(s.rows(), [2]);
    }
}
