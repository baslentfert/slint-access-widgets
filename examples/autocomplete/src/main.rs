//! Typing with suggestions underneath (see ui/app.slint). The suggestions
//! arrive after 300 ms, as if from a server.

use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, SharedString, VecModel};

slint::include_modules!();

const COMPUTERS: &[&str] = &[
    "ada-laptop", "alan-desktop", "build-server", "grace-surface", "home-nas", "kitchen-pi",
    "linus-workstation", "living-room-tv", "margaret-macbook", "office-printer", "studio-imac", "test-vm-01",
    "test-vm-02", "test-vm-03",
];

fn main() {
    let w = AppWindow::new().unwrap();
    let model = Rc::new(VecModel::<SharedString>::default());
    w.set_suggestions(model.clone().into());
    let ww = w.as_weak();
    // Only the last question counts (typing faster than the "server").
    let vraag = Rc::new(std::cell::Cell::new(0u32));
    w.on_edited(move |t| {
        let w = ww.unwrap();
        w.set_loading(true);
        let nr = vraag.get() + 1;
        vraag.set(nr);
        let (ww, model, vraag) = (ww.clone(), model.clone(), vraag.clone());
        let t = t.to_lowercase();
        slint::Timer::single_shot(Duration::from_millis(300), move || {
            if vraag.get() != nr {
                return;
            }
            let found: Vec<SharedString> = COMPUTERS.iter().filter(|c| c.contains(&t)).map(|c| SharedString::from(*c)).collect();
            model.set_vec(found);
            ww.unwrap().set_loading(false);
        });
    });
    w.run().unwrap();
}
