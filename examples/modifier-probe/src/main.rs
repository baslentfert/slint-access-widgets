//! Reproduces a Windows issue with native context menus and held modifiers
//! (see ui/app.slint).

slint::include_modules!();

fn main() {
    AppWindow::new().unwrap().run().unwrap();
}
