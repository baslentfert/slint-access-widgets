//! Messages: alerts in the window and a snackbar (see ui/app.slint).

slint::include_modules!();

fn main() {
    AppWindow::new().unwrap().run().unwrap();
}
