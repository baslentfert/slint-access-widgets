//! `cargo run`: a table that a screen reader can follow. Tab into it, use
//! the arrows, press Left/Right to hear one cell.

slint::include_modules!();

fn main() {
    AppWindow::new().unwrap().run().unwrap();
}
