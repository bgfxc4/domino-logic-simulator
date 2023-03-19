use std::{sync::Arc, sync::Mutex};

use crate::{ui_3d::UI3d, simulator::Simulator};

pub struct MainWindow {
    value: f32,
    gt: Option<UI3d>,
    simulator: Arc<Mutex<Simulator>>,
}

impl MainWindow {
    pub fn new(cc: &eframe::CreationContext<'_>, simulator: Arc<Mutex<Simulator>>) -> Self {
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        let slf = Self {
            gt: UI3d::new(cc, simulator.clone()),
            value: 2.4,
            simulator: simulator.clone(),
        };
        slf
    }
}

impl eframe::App for MainWindow {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self { value, gt, simulator} = self;

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::Window::new("Window").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut "asd".to_string().to_owned());
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                ui.label(".");
            });
        });
        match gt {
            Some(g) => (g as &mut dyn eframe::App).update(ctx, frame),
            None => {}
        }
    }
}
