pub mod main_window;
use main_window::MainWindow;

pub mod ui_3d;

fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(MainWindow::new(cc))),
        )
}
