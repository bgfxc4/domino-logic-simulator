use std::{sync::Mutex as stdMutex, sync::Arc};
use cgmath::InnerSpace;
use eframe::egui_glow::{self, *};
use egui::{mutex::Mutex, Pos2};

pub mod shaders;
pub mod canvas;
use canvas::*;

use crate::simulator::Simulator;

#[derive(Clone)]
pub struct RenderMatrices {
    perspective: cgmath::Matrix4<f32>,
    view: cgmath::Matrix4<f32>,
    rotation: Option<cgmath::Matrix4<f32>>,
    translation: Option<cgmath::Matrix4<f32>>,
    scale: Option<cgmath::Matrix4<f32>>,
}

pub struct UI3d {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    pub canvas: Arc<Mutex<Canvas>>,
    cam_pos: cgmath::Point3<f32>,
    cam_angle: cgmath::Vector2<f32>,
    simulator: Arc<stdMutex<Simulator>>,
    fov: cgmath::Rad<f32>,
    pub selected_domino_id: Option<u32>,
}

impl UI3d {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>, simulator: Arc<stdMutex<Simulator>>) -> Option<Self> {
        let gl = cc.gl.as_ref()?;
        Some(Self {
            canvas: Arc::new(Mutex::new(Canvas::new(gl, simulator.clone())?)),
            cam_pos: cgmath::Point3{x: 1.0f32, y: 2.0f32, z: 2.0f32},
            cam_angle: cgmath::Vector2{x: 0f32, y: 0f32},
            simulator,
            fov: cgmath::Rad(60f32 * ((2.0*std::f32::consts::PI) / 360.0)),
            selected_domino_id: None,
        })
    }
}

impl eframe::App for UI3d {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (keys_down, mods, screen_rect, pointer) = ctx.input(|i| (i.keys_down.to_owned(), i.modifiers, i.screen_rect, i.pointer.to_owned()));

        // if pointer.any_click() {
        //     self.get_clicked_ray_obb_intersection(pointer.interact_pos().unwrap(), screen_rect);
        // }
        let frame = egui::Frame::none().inner_margin(egui::Margin::same(0.0)).outer_margin(egui::Margin::same(0.0));
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            self.custom_painting(ui, screen_rect, keys_down, mods, pointer.interact_pos());
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
    fn custom_painting(&mut self, ui: &mut egui::Ui, screen_rect: egui::Rect, keys_down: std::collections::HashSet<egui::Key>, mods: egui::Modifiers, mouse_pos: Option<Pos2>) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

        if response.clicked() {
            self.get_clicked_ray_obb_intersection(mouse_pos.unwrap_or_default(), screen_rect);
        }

        let drag = response.drag_delta();
        let render_mats = self.calc_mvp(keys_down, mods, drag, screen_rect);
        let cam_pos = self.cam_pos.to_owned();

        let canvas = self.canvas.clone();
        let selected_domino_id = self.selected_domino_id.to_owned();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            canvas.lock().paint(painter.gl(), render_mats.clone(), cam_pos, selected_domino_id);
        });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }

    fn calc_mvp(&mut self, keys_down: std::collections::HashSet<egui::Key>, mods: egui::Modifiers, drag: egui::Vec2, screen_rect: egui::Rect) -> RenderMatrices {
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

        let proj_mat = cgmath::perspective(self.fov, screen_rect.aspect_ratio(), 0.1f32, 100f32);
        let view_mat = cgmath::Matrix4::look_at_rh(self.cam_pos, self.cam_pos + direction, up_vec);
        // let mat = proj_mat * view_mat;
        RenderMatrices { perspective: proj_mat, view: view_mat, rotation: None, translation: None, scale: None }
    }

    fn get_clicked_ray_obb_intersection(&mut self, click_pos: egui::Pos2, screen_size: egui::Rect) {
        let ray_origin = self.cam_pos;

        let relative_click = cgmath::point2((click_pos.x / screen_size.width()) - 0.5, (click_pos.y / screen_size.height()) - 0.5);
        let click_angle = cgmath::point2(self.fov * relative_click.x * screen_size.aspect_ratio(), self.fov * relative_click.y);

        let ray_direction = cgmath::Vector3 {
            x: cgmath::Angle::cos(cgmath::Rad(self.cam_angle.y)-click_angle.y) * cgmath::Angle::sin(cgmath::Rad(self.cam_angle.x)-click_angle.x),
            y: cgmath::Angle::sin(cgmath::Rad(self.cam_angle.y)-click_angle.y),
            z: cgmath::Angle::cos(cgmath::Rad(self.cam_angle.y)-click_angle.y) * cgmath::Angle::cos(cgmath::Rad(self.cam_angle.x)-click_angle.x)
        };

        let aabb_min = cgmath::Point3{x: -0.035f32, y: 0.0, z: -0.01};
        let aabb_max = cgmath::Point3{x: 0.035f32, y: 0.14, z: 0.01};

        let (model_mats, _rot_mats) = self.canvas.lock().get_model_mats_list();

        'mats_loop: for (id, model_mat) in model_mats {
            
            let mut t_min = 0.0f32;
            let mut t_max = f32::MAX;

            let obb_pos_world_space = cgmath::Point3 {x: model_mat.w.x, y: model_mat.w.y, z: model_mat.w.z};
            let delta: cgmath::Vector3<f32> = obb_pos_world_space - ray_origin;

            for i in 0..3 {
                // let xaxis = cgmath::Vector3 {x: model_mat.x.x, y: model_mat.x.y, z: model_mat.x.z};
                let axis = match i {
                    0 => cgmath::Vector3 {x: model_mat.x.x, y: model_mat.x.y, z: model_mat.x.z},
                    1 => cgmath::Vector3 {x: model_mat.y.x, y: model_mat.y.y, z: model_mat.y.z},
                    2 => cgmath::Vector3 {x: model_mat.z.x, y: model_mat.z.y, z: model_mat.z.z},
                    _ => {println!("range is wrong"); cgmath::Vector3 {x: model_mat.x.x, y: model_mat.x.y, z: model_mat.x.z}},
                };
                let e = axis.dot(delta);
                let f = ray_direction.dot(axis);

                let (aabb_min_t, aabb_max_t) = match i {
                    0 => (aabb_min.x, aabb_max.x),
                    1 => (aabb_min.y, aabb_max.y),
                    2 => (aabb_min.z, aabb_max.z),
                    _ => {println!("range is wrong"); (aabb_min.x, aabb_max.x)},
                };
                let mut t1 = (e+aabb_min_t) / f; // Intersection with the "left" plane
                let mut t2 = (e+aabb_max_t) / f; // Intersection with the "right" plane
                
                if t1 > t2 { // if wrong order -> swap -> t1 represents nearest intersection
                    let tmp = t1;
                    t1 = t2;
                    t2 = tmp;
                }

                if t2 < t_max {
                    t_max = t2;
                }

                if t1 > t_min {
                    t_min = t1;
                }

                if t_max < t_min {
                    // println!("no hit");
                    self.selected_domino_id = None;
                    continue 'mats_loop;
                }
            }
            // println!("intersection distance: {}, id: {}", t_min, id);
            self.selected_domino_id = Some(id);
            break;
        }
    }
}
