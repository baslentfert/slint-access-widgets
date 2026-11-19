//! A breadcrumb trail (see ui/app.slint).

use slint::{Model, VecModel};
use std::rc::Rc;

slint::include_modules!();

fn main() {
    let w = AppWindow::new().unwrap();
    let path: Rc<VecModel<slint::SharedString>> =
        Rc::new(VecModel::from(["Home", "Documents", "Projects", "2026"].map(slint::SharedString::from).to_vec()));
    w.set_path(path.clone().into());
    let p = path.clone();
    // A step up: everything after it goes.
    w.on_go_up(move |i| {
        while p.row_count() > i as usize + 1 {
            p.remove(p.row_count() - 1);
        }
    });
    w.on_go_into(move |f| path.push(f));
    w.run().unwrap();
}
