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

        egui::Window::new("Domino Inspector").show(ctx, |ui| {
            let mut s = self.simulator.lock().unwrap();
            let domino_id = match &self.ui_3d {
                Some(u) => u.selected_domino_id,
                None => None
            };
            let domino: Option<&mut Domino> = s.dominos.iter_mut().find(|d| domino_id == Some(d.id));

            if domino_id.is_none() || domino.is_none() {
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

            if ui.button("Delete domino").clicked() {
                let idx = s.dominos.iter().position(|d| Some(d.id) == self.ui_3d.as_ref().unwrap().selected_domino_id);
                if idx.is_some() {
                    s.dominos.remove(idx.unwrap());
                    self.ui_3d.as_mut().unwrap().selected_domino_id = None;
                }
            }
        });

        egui::Window::new("Domino Creator").show(ctx, |ui| {
            if ui.button("Create domino").clicked() {
                let mut s = self.simulator.lock().unwrap();
                let id = match s.dominos.last() {
                    Some(d) => d.id+1,
                    None => 0
                };
                s.dominos.push(Domino {
                    fall_rotation: 0.0,
                    id,
                    rotation_y: 0.0,
                    position: cgmath::point3(0.0, 0.0, 0.0),
                    scale: cgmath::vec3(1.0, 1.0, 1.0),
                });
                self.ui_3d.as_mut().unwrap().selected_domino_id = Some(id);
            }
        });

        match &mut self.ui_3d {
            // Some(g) => (&mut g as &mut dyn eframe::App).update(ctx, frame),
            Some(g) => g.update(ctx, frame),
            None => {}
        }
    }
}
