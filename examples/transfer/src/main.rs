//! Sending files from a list, simulated: nothing really goes anywhere.
//! One file at a time moves (they share one connection), the others wait.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, Model, VecModel};

slint::include_modules!();

/// Bytes per second in this pretend connection.
const SPEED: f64 = 12.0 * 1024.0 * 1024.0;
const TICK: Duration = Duration::from_millis(100);

fn mb(b: f64) -> String {
    if b >= 1024.0 * 1024.0 * 1024.0 { format!("{:.1} GB", b / 1024f64.powi(3)) } else { format!("{:.0} MB", b / 1024f64.powi(2)) }
}

fn main() {
    let names = ["holiday", "invoice", "report", "backup", "photos", "budget", "notes", "setup", "letter", "slides"];
    let exts = ["mp4", "pdf", "pdf", "zip", "zip", "xlsx", "txt", "exe", "docx", "pptx"];
    let sizes: Vec<f64> = (0..60).map(|i| ((i * 7919) % 400 + 2) as f64 * 1024.0 * 1024.0 * if i % 7 == 0 { 3.0 } else { 0.3 }).collect();
    let items = Rc::new(VecModel::from(
        (0..60)
            .map(|i| Item {
                name: format!("{}-{:02}.{}", names[i % 10], i, exts[i % 10]).into(),
                size: mb(sizes[i]).into(),
                chosen: false,
                state: State::Idle,
                progress: 0.0,
            })
            .collect::<Vec<_>>(),
    ));
    let sizes = Rc::new(sizes);
    let w = AppWindow::new().unwrap();
    w.set_items(items.clone().into());
    let paused = Rc::new(Cell::new(false));
    // The batch being sent: indices, in order.
    let batch = Rc::new(RefCell::new(Vec::<usize>::new()));

    let count_chosen = {
        let items = items.clone();
        move || items.iter().filter(|it| it.chosen && !matches!(it.state, State::Waiting | State::Sending)).count() as i32
    };
    {
        let (items, ww, cc) = (items.clone(), w.as_weak(), count_chosen.clone());
        w.on_toggle(move |i| {
            let mut it = items.row_data(i as usize).unwrap();
            it.chosen = !it.chosen;
            items.set_row_data(i as usize, it);
            let n = cc();
            ww.unwrap().set_chosen_count(n);
            n
        });
    }
    {
        let (items, ww, cc) = (items.clone(), w.as_weak(), count_chosen.clone());
        w.on_toggle_all(move || {
            let all = items.iter().filter(|it| !matches!(it.state, State::Waiting | State::Sending)).all(|it| it.chosen);
            for i in 0..items.row_count() {
                let mut it = items.row_data(i).unwrap();
                if !matches!(it.state, State::Waiting | State::Sending) {
                    it.chosen = !all;
                    items.set_row_data(i, it);
                }
            }
            let n = cc();
            ww.unwrap().set_chosen_count(n);
            n
        });
    }
    {
        let (items, ww, batch) = (items.clone(), w.as_weak(), batch.clone());
        w.on_send(move || {
            let w = ww.unwrap();
            let mut b = batch.borrow_mut();
            let mut n = 0;
            for i in 0..items.row_count() {
                let mut it = items.row_data(i).unwrap();
                if it.chosen && !matches!(it.state, State::Waiting | State::Sending) {
                    it.state = State::Waiting;
                    it.progress = 0.0;
                    it.chosen = false;
                    items.set_row_data(i, it);
                    b.push(i);
                    n += 1;
                }
            }
            w.set_chosen_count(0);
            w.set_busy(true);
            w.invoke_say(format!("{n} files are being sent").into());
        });
    }
    {
        let (ww, paused) = (w.as_weak(), paused.clone());
        w.on_pause(move |p| {
            paused.set(p);
            ww.unwrap().set_paused(p);
        });
    }
    let stop = {
        let (items, batch) = (items.clone(), batch.clone());
        move |i: usize| {
            let mut it = items.row_data(i).unwrap();
            if matches!(it.state, State::Waiting | State::Sending) {
                it.state = State::Cancelled;
                items.set_row_data(i, it);
            }
            batch.borrow_mut().retain(|&j| j != i);
        }
    };
    {
        let stop = stop.clone();
        w.on_cancel(move |i| stop(i as usize));
    }
    {
        let (batch, ww) = (batch.clone(), w.as_weak());
        w.on_cancel_all(move || {
            let all: Vec<usize> = batch.borrow().clone();
            all.into_iter().for_each(&stop);
            ww.unwrap().invoke_say("All transfers cancelled".into());
        });
    }

    // The pretend connection: the first file of the batch moves.
    let timer = slint::Timer::default();
    {
        let (items, ww, batch, sizes, paused) = (items.clone(), w.as_weak(), batch.clone(), sizes.clone(), paused.clone());
        let done_in_batch = Rc::new(Cell::new(0usize));
        let failed_once = RefCell::new(std::collections::HashSet::<usize>::new());
        timer.start(slint::TimerMode::Repeated, TICK, move || {
            let w = ww.unwrap();
            let mut b = batch.borrow_mut();
            if b.is_empty() {
                if w.get_busy() {
                    w.set_busy(false);
                    w.set_paused(false);
                    paused.set(false);
                    w.set_summary(format!("{} files arrived", done_in_batch.get()).into());
                    w.invoke_say(format!("{} files arrived", done_in_batch.get()).into());
                    done_in_batch.set(0);
                }
                return;
            }
            // Totals for the line below the list.
            let total: f64 = b.iter().map(|&i| sizes[i]).sum();
            let left: f64 = b.iter().map(|&i| sizes[i] * (1.0 - items.row_data(i).unwrap().progress as f64)).sum();
            let secs = (left / SPEED).ceil() as u64;
            let when = if paused.get() { "paused".to_string() } else if secs < 60 { format!("about {secs} s left") } else { format!("about {} min left", secs.div_ceil(60)) };
            w.set_summary(format!("{} files left · {} of {} · {when}", b.len(), mb(total - left), mb(total)).into());
            if paused.get() {
                return;
            }
            let i = b[0];
            let mut it = items.row_data(i).unwrap();
            it.state = State::Sending;
            it.progress = (it.progress as f64 + SPEED * TICK.as_secs_f64() / sizes[i]).min(1.0) as f32;
            if it.progress >= 1.0 {
                it.state = State::Done;
                b.remove(0);
                done_in_batch.set(done_in_batch.get() + 1);
            } else if i % 9 == 4 && it.progress > 0.6 && failed_once.borrow_mut().insert(i) {
                // Pretend the connection broke for this one (every 9th
                // file, the first time): red row; tick it to try again.
                it.state = State::Failed;
                b.remove(0);
                w.invoke_say(format!("Failed: {}", it.name).into());
            }
            items.set_row_data(i, it);
        });
    }
    w.run().unwrap();
}
