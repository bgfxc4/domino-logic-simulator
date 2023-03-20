use std::{sync::Arc, sync::Mutex};

use crate::{ui_3d::UI3d, simulator::{Simulator, Domino}};

pub struct MainWindow {
    value: f32,
    ui_3d: Option<UI3d>,
    simulator: Arc<Mutex<Simulator>>,
}

impl MainWindow {
    pub fn new(cc: &eframe::CreationContext<'_>, simulator: Arc<Mutex<Simulator>>) -> Self {
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }
        let slf = Self {
            ui_3d: UI3d::new(cc, simulator.clone()),
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
        // let Self { value, ui_3d, simulator: _simulator} = self;

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
            let mut s = self.simulator.lock().unwrap();
            let domino_id = match &self.ui_3d {
                Some(u) => u.selected_domino_id,
                None => None
            };
            let domino: Option<&mut Domino> = s.dominos.iter_mut().find(|d| domino_id == Some(d.id));

            ui.heading("Domino inspector");
            
            if domino_id.is_none() {
                ui.label("No domino is selected.");
                ui.label("Selected a domino by clicking on it to see it in this inspector.");
                return
            } else if domino.is_none() {
                ui.label("No domino is selected.");
                ui.label("Selected a domino by clicking on it to see it in this inspector.");
                return
            }
            let domino = domino.unwrap();
            
            ui.label(domino.id.to_string());

            ui.add(egui::Slider::new(&mut domino.position.x, -10.0..=10.0).text("x-Position"));
            ui.add(egui::Slider::new(&mut domino.position.y, -10.0..=10.0).text("y-Position"));
            ui.add(egui::Slider::new(&mut domino.position.z, -10.0..=10.0).text("z-Position"));

            ui.add(egui::Slider::new(&mut domino.rotation_y, 0.0..=360.0).text("y-Rotation"));
            ui.add(egui::Slider::new(&mut domino.fall_rotation, -90.0..=90.0).text("fall-rotation"));

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

        match &mut self.ui_3d {
            // Some(g) => (&mut g as &mut dyn eframe::App).update(ctx, frame),
            Some(g) => g.update(ctx, frame),
            None => {}
        }
    }
}
