//! Choosing columns with a transfer list (see ui/app.slint).

use std::rc::Rc;

use slint_access_widgets::TransferLists;

slint::include_modules!();

fn main() {
    let w = AppWindow::new().unwrap();
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let t = Rc::new(TransferLists::new(
        s(&["Type", "Modified", "Created", "Owner", "Permissions", "Path", "Tags", "Checksum"]),
        s(&["Name", "Size"]),
    ));
    w.set_left_items(t.items(TransferLists::LEFT));
    w.set_right_items(t.items(TransferLists::RIGHT));
    w.set_left_selected(t.selected(TransferLists::LEFT));
    w.set_right_selected(t.selected(TransferLists::RIGHT));
    let c = t.clone();
    w.on_select(move |side, row, how| c.select(side, row, how));
    let c = t.clone();
    w.on_transfer(move |side, current| c.transfer(side, current));
    let c = t.clone();
    w.on_landing(move |side| c.landing(side));
    let c = t.clone();
    w.on_reorder(move |row, delta| c.reorder(row, delta));
    let c = t.clone();
    w.on_find(move |side, p, from| c.find(side, &p, from));
    w.run().unwrap();
}
