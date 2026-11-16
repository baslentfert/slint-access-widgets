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

/// Rows as text for the clipboard: cells separated by tabs, rows by line
/// breaks, so pasting into a spreadsheet gives columns (and into an email
/// or editor plain lines). With `titles`, a first line with the column
/// titles. Tabs and line breaks inside a cell become spaces.
pub fn rows_as_text<R>(rows: &impl Model<Data = R>, which: &[usize], titles: Option<&[&str]>) -> String
where
    R: Model<Data = StandardListViewItem>,
{
    let clean = |s: &str| s.replace(['\t', '\n', '\r'], " ");
    let mut out = String::new();
    if let Some(t) = titles {
        out.push_str(&t.iter().map(|s| clean(s)).collect::<Vec<_>>().join("\t"));
        out.push('\n');
    }
    for &i in which {
        let Some(row) = rows.row_data(i) else { continue };
        let cells: Vec<String> = row.iter().map(|c| clean(&c.text)).collect();
        out.push_str(&cells.join("\t"));
        out.push('\n');
    }
    out
}

/// Ctrl+C in an `AccessTable`: copies the selected rows (or the current
/// row) to the clipboard and returns how many, for the announcement; -1
/// when the clipboard refused (announced as "Could not copy").
///
/// ```ignore
/// let (rows, sel) = (model.clone(), selection.clone());
/// window.on_copy(move |current| slint_access_widgets::copy_rows(&*rows, &sel.rows(), current));
/// ```
#[cfg(feature = "clipboard")]
pub fn copy_rows<R>(rows: &impl Model<Data = R>, selected: &[usize], current: i32) -> i32
where
    R: Model<Data = StandardListViewItem>,
{
    let which: Vec<usize> = if selected.is_empty() && current >= 0 { vec![current as usize] } else { selected.to_vec() };
    // Rows still loading (empty) are left out.
    let which: Vec<usize> = which.into_iter().filter(|&i| rows.row_data(i).is_some_and(|r| r.row_count() > 0)).collect();
    if which.is_empty() {
        return 0;
    }
    match copy_text(&rows_as_text(rows, &which, None)) {
        Ok(()) => which.len() as i32,
        Err(_) => -1,
    }
}

/// Puts text on the system clipboard (the `clipboard` feature, via
/// `arboard`).
#[cfg(feature = "clipboard")]
pub fn copy_text(text: &str) -> Result<(), String> {
    thread_local! {
        // On Linux the text stays on the clipboard only while this lives.
        static CLIPBOARD: std::cell::RefCell<Option<arboard::Clipboard>> = const { std::cell::RefCell::new(None) };
    }
    // Another program can hold the clipboard for a moment: try a few times.
    let mut fout = String::new();
    for poging in 0..5 {
        if poging > 0 {
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        let r = CLIPBOARD.with(|c| {
            let mut c = c.borrow_mut();
            if c.is_none() {
                *c = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
            }
            c.as_mut().unwrap().set_text(text).map_err(|e| e.to_string())
        });
        match r {
            Ok(()) => return Ok(()),
            Err(e) => fout = e,
        }
    }
    Err(fout)
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
/// widgets. When the rows change (sort, filter), call
/// [`Selection::set_rows_by_id`] with an id per row (a file name, a database
/// key): the same items stay selected, wherever they are now. Or
/// [`Selection::reset`] to start over.
pub struct Selection {
    model: Rc<VecModel<bool>>,
    anchor: Cell<usize>,
    count: Cell<usize>,
    /// An id per row, when given with `set_rows_by_id`.
    ids: std::cell::RefCell<Vec<String>>,
}

impl Selection {
    pub const ONLY: i32 = 0;
    pub const TOGGLE: i32 = 1;
    pub const EXTEND: i32 = 2;
    pub const ALL: i32 = 3;
    pub const CLEAR: i32 = 4;

    pub fn new(rows: usize) -> Self {
        Self { model: Rc::new(VecModel::from(vec![false; rows])), anchor: Cell::new(0), count: Cell::new(0), ids: Default::default() }
    }

    /// New rows, with an id each: the items that were selected stay
    /// selected (and the Shift anchor stays on its item). Returns how many
    /// are selected now (items that are gone, e.g. filtered out, drop out).
    pub fn set_rows_by_id<S: Into<String>>(&self, ids: impl IntoIterator<Item = S>) -> i32 {
        let old = self.ids.borrow();
        let chosen: std::collections::HashSet<String> = self.rows().into_iter().filter_map(|i| old.get(i).cloned()).collect();
        let anchor = old.get(self.anchor.get()).cloned();
        drop(old);
        let ids: Vec<String> = ids.into_iter().map(Into::into).collect();
        let marks: Vec<bool> = ids.iter().map(|id| chosen.contains(id)).collect();
        self.count.set(marks.iter().filter(|m| **m).count());
        self.anchor.set(anchor.and_then(|a| ids.iter().position(|id| *id == a)).unwrap_or(0));
        self.model.set_vec(marks);
        *self.ids.borrow_mut() = ids;
        self.count.get() as i32
    }

    /// The ids of the selected rows (after [`Selection::set_rows_by_id`]).
    pub fn selected_ids(&self) -> Vec<String> {
        let ids = self.ids.borrow();
        self.rows().into_iter().filter_map(|i| ids.get(i).cloned()).collect()
    }

    /// The row of an id, e.g. to keep the current row on its item after a
    /// sort (`set-current-row`).
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.ids.borrow().iter().position(|i| i == id)
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
        self.ids.borrow_mut().clear();
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

/// Lazy loading for `AccessTable`: rows come in blocks, from a server or
/// another computer, only when they are (nearly) on screen.
///
/// The model always has all rows (so the table knows the total, End works
/// and a screen reader hears "row 400 of 100,000"); a row not loaded yet is
/// an empty array, which the table shows and reads as "Loading…".
///
/// ```ignore
/// let lazy = Rc::new(LazyRows::new(100_000, 100));
/// window.set_files(lazy.model());
/// let l = lazy.clone();
/// window.on_rows_needed(move |first, count| {
///     for block in l.needed(first, count) {
///         // fetch `block` (a range of row numbers), later: l.fill(block.start, rows)
///     }
/// });
/// ```
pub struct LazyRows {
    model: Rc<VecModel<ModelRc<StandardListViewItem>>>,
    block: usize,
    /// Blocks asked for (or loaded).
    asked: std::cell::RefCell<std::collections::HashSet<usize>>,
}

impl LazyRows {
    /// `total` rows, fetched `block` rows at a time.
    pub fn new(total: usize, block: usize) -> Self {
        let s = Self { model: Rc::new(VecModel::default()), block: block.max(1), asked: Default::default() };
        s.reset(total);
        s
    }

    /// For `rows`.
    pub fn model(&self) -> ModelRc<ModelRc<StandardListViewItem>> {
        self.model.clone().into()
    }

    /// Start over with `total` rows, none loaded (a new search, another sort
    /// order, another folder).
    pub fn reset(&self, total: usize) {
        self.model.set_vec(vec![ModelRc::default(); total]);
        self.asked.borrow_mut().clear();
    }

    /// The blocks to fetch for rows `first .. first + count` (from
    /// `rows-needed`), plus one block before and after so scrolling seldom
    /// shows "Loading…". Each block is handed out once.
    pub fn needed(&self, first: i32, count: i32) -> Vec<std::ops::Range<usize>> {
        let total = self.model.row_count();
        if total == 0 || count <= 0 {
            return Vec::new();
        }
        let first = first.max(0) as usize;
        let last = (first + count as usize).min(total);
        let from = (first / self.block).saturating_sub(1);
        let to = ((last.saturating_sub(1)) / self.block + 1).min(total.div_ceil(self.block) - 1);
        let mut asked = self.asked.borrow_mut();
        (from..=to)
            .filter(|b| asked.insert(*b))
            .map(|b| b * self.block..((b + 1) * self.block).min(total))
            .collect()
    }

    /// Rows arrived, starting at row `start`: one `Vec` of cell texts each.
    pub fn fill<S: Into<slint::SharedString>>(&self, start: usize, rows: impl IntoIterator<Item = Vec<S>>) {
        for (i, cells) in rows.into_iter().enumerate() {
            let row = start + i;
            if row >= self.model.row_count() {
                break;
            }
            let cells: Vec<StandardListViewItem> = cells.into_iter().map(|t| StandardListViewItem::from(t.into())).collect();
            self.model.set_row_data(row, ModelRc::from(Rc::new(VecModel::from(cells))));
        }
    }

    /// Fetching a block failed: ask again next time it is needed.
    pub fn forget(&self, rows: std::ops::Range<usize>) {
        let mut asked = self.asked.borrow_mut();
        for b in rows.start / self.block..=rows.end.saturating_sub(1) / self.block {
            asked.remove(&b);
        }
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
    fn rows_as_text_for_spreadsheets() {
        let row = |cells: &[&str]| ModelRc::from(Rc::new(VecModel::from(cells.iter().map(|c| StandardListViewItem::from(*c)).collect::<Vec<_>>())));
        let rows = VecModel::from(vec![row(&["a.txt", "1 kB"]), row(&["tab\there", "line\nbreak"]), row(&["c", "3"])]);
        assert_eq!(rows_as_text(&rows, &[0, 2], None), "a.txt\t1 kB\nc\t3\n");
        assert_eq!(rows_as_text(&rows, &[1], Some(&["Name", "Size"])), "Name\tSize\ntab here\tline break\n");
    }

    #[test]
    fn lazy_rows_hand_out_each_block_once() {
        let l = LazyRows::new(1000, 100);
        // Rows 250..270 on screen: blocks 1, 2 (around it) and 3.
        assert_eq!(l.needed(250, 20), vec![100..200, 200..300, 300..400]);
        assert!(l.needed(260, 20).is_empty());
        // At the end: no block past the last row.
        assert_eq!(l.needed(990, 10), vec![800..900, 900..1000]);
        l.fill(200, vec![vec!["a", "1"], vec!["b", "2"]]);
        assert_eq!(l.model().row_data(201).unwrap().row_data(0).unwrap().text, "b");
        assert_eq!(l.model().row_data(202).unwrap().row_count(), 0);
        l.forget(300..400);
        // Asked again, with the margin block after it (not asked before).
        assert_eq!(l.needed(300, 10), vec![300..400, 400..500]);
        l.reset(5);
        assert_eq!(l.model().row_count(), 5);
        assert_eq!(l.needed(0, 5), vec![0..5]);
    }

    #[test]
    fn selection_follows_the_items() {
        let s = Selection::new(0);
        s.set_rows_by_id(["a", "b", "c", "d"]);
        s.apply(1, Selection::ONLY);
        assert_eq!(s.apply(2, Selection::EXTEND), 2);
        // Sorted the other way round: b and c are still selected.
        assert_eq!(s.set_rows_by_id(["d", "c", "b", "a"]), 2);
        assert_eq!(s.rows(), [1, 2]);
        assert_eq!(s.selected_ids(), ["c", "b"]);
        // Shift from the same anchor (b, now row 2) to row 0.
        assert_eq!(s.apply(0, Selection::EXTEND), 3);
        assert_eq!(s.selected_ids(), ["d", "c", "b"]);
        // Filtered: d is gone, c and b stay.
        assert_eq!(s.set_rows_by_id(["b", "c", "x"]), 2);
        assert_eq!(s.index_of("x"), Some(2));
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
