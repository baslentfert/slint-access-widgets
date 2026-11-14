//! The gallery: every widget, with real-looking data. `cargo run`
//! (`cargo run -- 200000` for a long table).

use std::rc::Rc;

use slint::{ModelRc, SharedString, StandardListViewItem, VecModel};

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
    let count: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(2000);
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
    let data = Rc::new(std::cell::RefCell::new(data));
    let model = Rc::new(VecModel::<ModelRc<StandardListViewItem>>::default());
    let fill = {
        let (data, model) = (data.clone(), model.clone());
        move || model.set_vec(data.borrow().iter().map(|(n, s, m)| row(n, *s, m)).collect::<Vec<_>>())
    };
    fill();

    let w = AppWindow::new().unwrap();
    w.set_files(model.clone().into());
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
