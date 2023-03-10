use std::sync::Arc;
use cgmath::InnerSpace;
use eframe::egui_glow::{self, *};
use egui::mutex::Mutex;

pub mod shaders;
pub mod canvas;
use canvas::*;

pub struct UI3d {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    canvas: Arc<Mutex<Canvas>>,
    cam_pos: cgmath::Point3<f32>,
    cam_angle: cgmath::Vector2<f32>,
}

impl UI3d {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let gl = cc.gl.as_ref()?;
        Some(Self {
            canvas: Arc::new(Mutex::new(Canvas::new(gl)?)),
            cam_pos: cgmath::Point3{x: 1.0f32, y: 2.0f32, z: 2.0f32},
            cam_angle: cgmath::Vector2{x: 0f32, y: 0f32},
        })
    }
}

impl eframe::App for UI3d {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (keys_down, mods, screen_rect) = ctx.input(|i| (i.keys_down.to_owned(), i.modifiers, i.screen_rect));

        egui::CentralPanel::default().show(ctx, |ui| {
            self.custom_painting(ui, screen_rect, keys_down, mods);
        });
        ctx.request_repaint();
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.canvas.lock().destroy(gl);
        }
    }
}

impl UI3d {
    fn custom_painting(&mut self, ui: &mut egui::Ui, screen_rect: egui::Rect, keys_down: std::collections::HashSet<egui::Key>, mods: egui::Modifiers) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

        let drag = response.drag_delta();
        let mvp = self.calc_mvp(keys_down, mods, drag, screen_rect);

        let canvas = self.canvas.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            canvas.lock().paint(painter.gl(), mvp);
        });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }

    fn calc_mvp(&mut self, keys_down: std::collections::HashSet<egui::Key>, mods: egui::Modifiers, drag: egui::Vec2, screen_rect: egui::Rect) -> cgmath::Matrix4<f32> {
        self.cam_angle.x -= drag.x * 0.003f32;
        self.cam_angle.y -= drag.y * 0.003f32;

        let direction = cgmath::Vector3{
            x: cgmath::Angle::cos(cgmath::Rad(self.cam_angle.y)) * cgmath::Angle::sin(cgmath::Rad(self.cam_angle.x)),
            y: cgmath::Angle::sin(cgmath::Rad(self.cam_angle.y)),
            z: cgmath::Angle::cos(cgmath::Rad(self.cam_angle.y)) * cgmath::Angle::cos(cgmath::Rad(self.cam_angle.x))
        };
        
        let mut move_direction = direction;
        move_direction.y = 0.0;

        let up_vec = cgmath::Vector3{x: 0.0, y: 1.0, z: 0.0};

        let right_vec = direction.cross(up_vec).normalize();

        if keys_down.contains(&egui::Key::S) {
            self.cam_pos -= move_direction * 0.02f32;
        }
        if keys_down.contains(&egui::Key::W) {
            self.cam_pos += move_direction * 0.02f32;
        }
        if keys_down.contains(&egui::Key::D) {
            self.cam_pos += right_vec * 0.02f32;
        }
        if keys_down.contains(&egui::Key::A) {
            self.cam_pos -= right_vec * 0.02f32;
        }
        if mods.shift {
            self.cam_pos += up_vec * 0.02f32;
        }
        if mods.ctrl {
            self.cam_pos -= up_vec * 0.02f32;
        }

        let proj_mat = cgmath::perspective(cgmath::Deg(60f32), screen_rect.aspect_ratio(), 0.1f32, 100f32);
        let view_mat = cgmath::Matrix4::look_at_rh(self.cam_pos, self.cam_pos + direction, up_vec);
        let mat = proj_mat * view_mat;
        mat
    }
}
