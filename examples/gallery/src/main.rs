//! The gallery: every widget, with real-looking data. `cargo run`
//! (`cargo run -- 200000` for a long table).

use std::rc::Rc;

use slint::{ComponentHandle, Model, ModelRc, SharedString, StandardListViewItem, VecModel};

slint::include_modules!();

fn row(name: &str, size: u64, modified: &str) -> ModelRc<StandardListViewItem> {
    let size = if size >= 1 << 20 { format!("{:.1} MB", size as f64 / (1 << 20) as f64) } else { format!("{} kB", size.div_ceil(1024)) };
    ModelRc::from(Rc::new(VecModel::from(vec![
        StandardListViewItem::from(SharedString::from(name)),
        StandardListViewItem::from(SharedString::from(size)),
        StandardListViewItem::from(SharedString::from(modified)),
    ])))
}

fn main() {
    let count: usize = std::env::args().skip(1).find_map(|a| a.parse().ok()).unwrap_or(2000);
    // One long name in eight, to show the tooltip for cut-off text.
    let kinds = ["report", "photo", "invoice", "notes", "backup", "setup", "budget-overview-for-the-whole-department", "letter"];
    let exts = ["pdf", "jpg", "pdf", "txt", "zip", "exe", "xlsx", "docx"];
    let days = ["today", "yesterday", "Monday", "last week", "in May"];
    // (name, size, modified) so we can sort.
    let data: Vec<(String, u64, String)> = (0..count)
        .map(|i| {
            let k = i % kinds.len();
            (format!("{}-{:05}.{}", kinds[k], i, exts[k]), ((i * 7919) % 50_000_000) as u64 + 300, days[i % days.len()].to_string())
        })
        .collect();
    let total = data.len();
    let data = Rc::new(std::cell::RefCell::new(data));
    let filter = Rc::new(std::cell::RefCell::new(String::new()));
    let model = Rc::new(VecModel::<ModelRc<StandardListViewItem>>::default());
    let selection = Rc::new(slint_access_widgets::Selection::new(0));
    // Rows on screen: the data, filtered. The selection stays on the same
    // files (the name is the id), wherever sorting or filtering puts them.
    let fill = {
        let (data, model, filter, selection) = (data.clone(), model.clone(), filter.clone(), selection.clone());
        move || {
            let f = filter.borrow().to_lowercase();
            let shown: Vec<_> = data.borrow().iter().filter(|(n, ..)| n.to_lowercase().contains(&f)).cloned().collect();
            selection.set_rows_by_id(shown.iter().map(|(n, ..)| n.clone()));
            model.set_vec(shown.iter().map(|(n, s, m)| row(n, *s, m)).collect::<Vec<_>>());
        }
    };
    fill();

    let w = AppWindow::new().unwrap();
    // English unless you ask: `--lang nl` shows the Dutch translation, the
    // widgets' own words included.
    let args: Vec<String> = std::env::args().collect();
    let lang = args.iter().position(|a| a == "--lang").and_then(|i| args.get(i + 1)).map_or("en", |l| l.as_str());
    let _ = slint::select_bundled_translation(lang);
    w.set_files(model.clone().into());
    w.set_total(total as i32);
    w.set_selected(selection.model());
    {
        let s = selection.clone();
        w.on_select(move |r, h| s.apply(r, h));
    }
    {
        // Ctrl+C: the selected rows (or the current one), as tab-separated
        // text: pastes as columns into a spreadsheet.
        let (model, s) = (model.clone(), selection.clone());
        w.on_copy(move |current| slint_access_widgets::copy_rows(&*model, &s.rows(), current));
    }
    {
        let (filter, fill) = (filter.clone(), fill.clone());
        w.on_filter(move |t| {
            *filter.borrow_mut() = t.to_string();
            fill();
        });
    }
    {
        // Pretend to load for a second: the table says "Loading…".
        let ww = w.as_weak();
        w.on_reload(move || {
            if let Some(w) = ww.upgrade() {
                w.set_loading(true);
            }
            let ww = ww.clone();
            slint::Timer::single_shot(std::time::Duration::from_secs(1), move || {
                if let Some(w) = ww.upgrade() {
                    w.set_loading(false);
                }
            });
        });
    }
    {
        let model = model.clone();
        w.on_find(move |p, from| slint_access_widgets::find_prefix(&*model, &p, from));
    }
    let sort = {
        let data = data.clone();
        move |col: i32, asc: bool| {
            let mut d = data.borrow_mut();
            match col {
                1 => d.sort_by_key(|r| r.1),
                2 => d.sort_by(|a, b| a.2.cmp(&b.2)),
                _ => d.sort_by_key(|r| r.0.to_lowercase()),
            }
            if !asc {
                d.reverse();
            }
        }
    };
    // Sort, and keep the current row on the same file (as File Explorer).
    let resort = {
        let (model, selection, ww) = (model.clone(), selection.clone(), w.as_weak());
        move |c: i32, asc: bool| {
            let w = ww.unwrap();
            let current = model.row_data(w.get_current_row().max(0) as usize).and_then(|r| r.row_data(0)).map(|c| c.text.to_string());
            sort(c, asc);
            fill();
            if let Some(i) = current.and_then(|id| selection.index_of(&id)) {
                w.invoke_keep_current(i as i32);
            }
        }
    };
    {
        let resort = resort.clone();
        w.on_sort_ascending(move |c| resort(c, true));
    }
    w.on_sort_descending(move |c| resort(c, false));
    w.run().unwrap();
}
