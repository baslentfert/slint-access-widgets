//! The gallery: every widget, with real-looking data. `cargo run`
//! (`cargo run -- 200000` for a long table).

use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, SharedString, StandardListViewItem, VecModel};

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
    let kinds = ["report", "photo", "invoice", "notes", "backup", "setup", "budget", "letter"];
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
    // Rows on screen: the data, filtered. The selection starts over.
    let fill = {
        let (data, model, filter, selection) = (data.clone(), model.clone(), filter.clone(), selection.clone());
        move || {
            let f = filter.borrow().to_lowercase();
            let rows: Vec<_> = data.borrow().iter().filter(|(n, ..)| n.to_lowercase().contains(&f)).map(|(n, s, m)| row(n, *s, m)).collect();
            selection.reset(rows.len());
            model.set_vec(rows);
        }
    };
    fill();

    let w = AppWindow::new().unwrap();
    // `--lang nl`: another language than the system's.
    let args: Vec<String> = std::env::args().collect();
    if let Some(l) = args.iter().position(|a| a == "--lang").and_then(|i| args.get(i + 1)) {
        let _ = slint::select_bundled_translation(l);
    }
    w.set_files(model.clone().into());
    w.set_total(total as i32);
    w.set_selected(selection.model());
    {
        let s = selection.clone();
        w.on_select(move |r, h| s.apply(r, h));
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
                _ => d.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase())),
            }
            if !asc {
                d.reverse();
            }
        }
    };
    {
        let (sort, fill) = (sort.clone(), fill.clone());
        w.on_sort_ascending(move |c| {
            sort(c, true);
            fill();
        });
    }
    w.on_sort_descending(move |c| {
        sort(c, false);
        fill();
    });
    w.run().unwrap();
}
