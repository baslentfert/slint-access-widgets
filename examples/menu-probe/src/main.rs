//! A probe for testing menus with screen readers (see ui/app.slint).

slint::include_modules!();

fn main() {
    AppWindow::new().unwrap().run().unwrap();
}
