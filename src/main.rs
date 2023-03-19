pub mod main_window;

use std::{sync::Arc, sync::Mutex};

use main_window::MainWindow;

pub mod ui_3d;

pub mod simulator;
use simulator::Simulator;

fn main() -> eframe::Result<()> {
    let simulator = Simulator::new();
    
    // Log to stdout (if you run with `RUST_LOG=debug`).
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "domino simulator",
        native_options,
        Box::new(|cc| Box::new(MainWindow::new(cc, Arc::new(Mutex::new(simulator))))),
        )
}
