//! Lazy loading: a pretend server with 100,000 files that needs 0.4 s per
//! block of 100 rows. The table only asks for what is on screen.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use slint::ComponentHandle;
use slint_access_widgets::LazyRows;

slint::include_modules!();

const TOTAL: usize = 100_000;
const BLOCK: usize = 100;
const DELAY: Duration = Duration::from_millis(400);

/// The "server": all names, the current search and sort.
struct Server {
    all: Vec<(String, u64, u32)>,
    view: Vec<usize>,
}

impl Server {
    fn new() -> Self {
        let words = ["report", "photo", "invoice", "notes", "backup", "setup", "budget", "letter", "slides", "song"];
        let exts = ["pdf", "jpg", "pdf", "txt", "zip", "exe", "xlsx", "docx", "pptx", "mp3"];
        let all = (0..TOTAL).map(|i| (format!("{}-{:06}.{}", words[i % 10], i, exts[i % 10]), (i as u64 * 7919) % 90_000_000 + 500, (i as u32 * 31) % 3650)).collect();
        Self { all, view: (0..TOTAL).collect() }
    }

    fn search(&mut self, text: &str) {
        let t = text.to_lowercase();
        self.view = (0..self.all.len()).filter(|&i| self.all[i].0.contains(&t)).collect();
    }

    fn sort(&mut self, column: i32, ascending: bool) {
        let all = &self.all;
        match column {
            1 => self.view.sort_by_key(|&i| all[i].1),
            2 => self.view.sort_by_key(|&i| all[i].2),
            _ => self.view.sort_by(|&a, &b| all[a].0.cmp(&all[b].0)),
        }
        if !ascending {
            self.view.reverse();
        }
    }

    fn rows(&self, range: std::ops::Range<usize>) -> Vec<Vec<String>> {
        range
            .filter_map(|r| self.view.get(r))
            .map(|&i| {
                let (name, size, age) = &self.all[i];
                vec![name.clone(), format!("{:.1} MB", *size as f64 / 1_048_576.0), format!("{age} days ago")]
            })
            .collect()
    }
}

fn main() {
    let server = Rc::new(RefCell::new(Server::new()));
    let lazy = Rc::new(LazyRows::new(TOTAL, BLOCK));
    let w = AppWindow::new().unwrap();
    w.set_files(lazy.model());
    w.set_total(TOTAL as i32);
    // A new search or sort: answers still on their way belong to the old
    // one, so they carry a generation number.
    let generation = Rc::new(std::cell::Cell::new(0u32));

    {
        let (lazy, server, generation) = (lazy.clone(), server.clone(), generation.clone());
        w.on_rows_needed(move |first, count| {
            for block in lazy.needed(first, count) {
                let (lazy, server, generation) = (lazy.clone(), server.clone(), generation.clone());
                let asked_in = generation.get();
                slint::Timer::single_shot(DELAY, move || {
                    if generation.get() == asked_in {
                        lazy.fill(block.start, server.borrow().rows(block));
                    }
                });
            }
        });
    }
    {
        let (lazy, server, generation) = (lazy.clone(), server.clone(), generation.clone());
        w.on_search(move |text| {
            server.borrow_mut().search(&text);
            generation.set(generation.get() + 1);
            lazy.reset(server.borrow().view.len());
        });
    }
    w.on_sort(move |column, ascending| {
        server.borrow_mut().sort(column, ascending);
        generation.set(generation.get() + 1);
        lazy.reset(server.borrow().view.len());
    });
    w.run().unwrap();
}
