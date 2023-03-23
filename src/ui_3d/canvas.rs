use std::{sync::Arc, sync::Mutex};
use eframe::egui_glow::*;
use glow::*;

use crate::{ui_3d::shaders, simulator::Simulator};

use super::RenderMatrices;

trait Renderable {
    unsafe fn destroy(&self, gl: &Context);
    unsafe fn paint(&self, gl: &Context, render_mats: &RenderMatrices, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>, selected_id: Option<u32>);
    unsafe fn fill_vbo(&self, gl: &Context, render_mats: &RenderMatrices, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>, selected_id: Option<u32>);
    fn is_instanced(&self) -> bool;
    unsafe fn fill_i_vbo(&mut self, _gl: &Context, _model_mats: &Vec<(u32, cgmath::Matrix4<f32>)>, _rot_mats: &Vec<cgmath::Matrix4<f32>>) {
        panic!("This struct is not an instanced RenderObject and thus has no instanced vertex object");
    }
}

struct RenderObject {
    program: Program,
    vao: NativeVertexArray,
    vbo: NativeBuffer,
}

struct InstancedRenderObject {
    program: Program,
    vao: NativeVertexArray,
    vbo: NativeBuffer,
    i_vbo: NativeBuffer,
    render_count: usize,
}

impl Renderable for InstancedRenderObject {
    unsafe fn destroy(&self, gl: &Context) {
        gl.delete_program(self.program);
        gl.delete_vertex_array(self.vao);
        gl.delete_buffer(self.vbo);
        gl.delete_buffer(self.i_vbo);
    }

    unsafe fn paint(&self, gl: &Context, render_mats: &RenderMatrices, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>, selected_id: Option<u32>) {
        self.fill_vbo(gl, render_mats, cam_pos, light_pos, selected_id);

        gl.draw_arrays_instanced(TRIANGLES, 0, 12*3, self.render_count.try_into().unwrap());
        // gl.draw_arrays(TRIANGLES, 0, 12*3);
        gl.bind_vertex_array(None);
    }

    unsafe fn fill_vbo(&self, gl: &Context, render_mats: &RenderMatrices, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>, selected_id: Option<u32>) {
        gl.use_program(Some(self.program));
        gl.bind_vertex_array(Some(self.vao));

        let light_pos_location = gl.get_uniform_location(self.program, "lightPos");
        let light_pos: [f32; 3] = [light_pos.x, light_pos.y, light_pos.z];
        gl.uniform_3_f32_slice(light_pos_location.as_ref(), &light_pos);

        let uniform_location = gl.get_uniform_location(self.program, "view_mat");
        let f32_mat: [[f32; 4]; 4] = render_mats.view.into();
        gl.uniform_matrix_4_f32_slice(uniform_location.as_ref(), false, f32_mat.into_iter().flatten().collect::<Vec<f32>>().as_ref());

        let perspective_location = gl.get_uniform_location(self.program, "perspective_mat");
        let f32_mat: [[f32; 4]; 4] = render_mats.perspective.into();
        gl.uniform_matrix_4_f32_slice(perspective_location.as_ref(), false, f32_mat.into_iter().flatten().collect::<Vec<f32>>().as_ref());

        let cam_pos_location = gl.get_uniform_location(self.program, "camPos");
        let cam_pos: [f32; 3] = [cam_pos.x, cam_pos.y, cam_pos.z];
        gl.uniform_3_f32_slice(cam_pos_location.as_ref(), &cam_pos);

        let id_location = gl.get_uniform_location(self.program, "selected_id");
        let selected_id: f32 = if selected_id.is_some() { selected_id.unwrap() as f32 } else { -1.0 };
        gl.uniform_1_f32(id_location.as_ref(), selected_id);
    }

    unsafe fn fill_i_vbo(&mut self, gl: &Context, model_mats: &Vec<(u32, cgmath::Matrix4<f32>)>, rot_mats: &Vec<cgmath::Matrix4<f32>>) { 
        gl.use_program(Some(self.program));
        gl.bind_vertex_array(Some(self.vao));

        self.render_count = model_mats.len();
        upload_vertex_attrib_model_and_rot(gl, model_mats, rot_mats, 3, &self.i_vbo);
    }

    fn is_instanced(&self) -> bool { true }
}

unsafe fn upload_vertex_attrib_model_and_rot(gl: &Context, model_mats: &Vec<(u32, cgmath::Matrix4<f32>)>, rot_mats: &Vec<cgmath::Matrix4<f32>>, location: u32, vbo: &NativeBuffer) {
    // let model_mats: Vec<cgmath::Matrix4<f32>> = model_mats.iter().map(|m| m.1).collect();
    // let mats: Vec<&cgmath::Matrix4<f32>> = model_mats.iter().zip(rot_mats.iter()).map(|t| { vec![t.0, t.1]}).flatten().collect();
    // let values: Vec<f32> = mats.iter().map(|m| { vec![m.x, m.y, m.z, m.w] }).flatten().map(|v| { vec![v.x, v.y, v.z, v.w] }).flatten().collect();

    let mats: Vec<(u32, cgmath::Matrix4<f32>, &cgmath::Matrix4<f32>)> = model_mats.iter().zip(rot_mats.iter()).map(|e| (e.0.0, e.0.1, e.1)).collect();
    let values: Vec<f32> = mats.iter().map(|e| vec![e.1.x.x, e.1.x.y, e.1.x.z, e.1.x.w, e.1.y.x, e.1.y.y, e.1.y.z, e.1.y.w, e.1.z.x, e.1.z.y, e.1.z.z, e.1.z.w, e.1.w.x, e.1.w.y, e.1.w.z, e.1.w.w,
                                            e.2.x.x, e.2.x.y, e.2.x.z, e.2.x.w, e.2.y.x, e.2.y.y, e.2.y.z, e.2.y.w, e.2.z.x, e.2.z.y, e.2.z.z, e.2.z.w, e.2.w.x, e.2.w.y, e.2.w.z, e.2.w.w, e.0 as f32]).flatten().collect();

    let values_u8: &[u8] = core::slice::from_raw_parts(
        values.as_slice().as_ptr() as *const u8,
        (values.len()+3) * core::mem::size_of::<f32>(), // TODO: figure out why "+3" is needed. If
                                                        // left away, last elements of values are
                                                        // not transferred to the shader
    );

    gl.bind_buffer(ARRAY_BUFFER, Some(*vbo));
    gl.buffer_data_u8_slice(ARRAY_BUFFER, values_u8, STATIC_DRAW);
    gl.enable_vertex_attrib_array(location+0); // mat4 model_mat -> 4x vec4 columns
    gl.enable_vertex_attrib_array(location+1);
    gl.enable_vertex_attrib_array(location+2);
    gl.enable_vertex_attrib_array(location+3);

    gl.enable_vertex_attrib_array(location+4+0);
    gl.enable_vertex_attrib_array(location+4+1);
    gl.enable_vertex_attrib_array(location+4+2);
    gl.enable_vertex_attrib_array(location+4+3);

    gl.enable_vertex_attrib_array(location+4+4);

    let stride: i32 = 16 * 4 * 2 + 4; // 16 f32s per matrix, 4 bytes per f32, 2 matrices and one
                                      // u32 (4 bytes) as id

    gl.vertex_attrib_pointer_f32(location+0, 4, FLOAT, false, stride, 0*4*4);
    gl.vertex_attrib_pointer_f32(location+1, 4, FLOAT, false, stride, 1*4*4);
    gl.vertex_attrib_pointer_f32(location+2, 4, FLOAT, false, stride, 2*4*4);
    gl.vertex_attrib_pointer_f32(location+3, 4, FLOAT, false, stride, 3*4*4);

    gl.vertex_attrib_pointer_f32(location+4+0, 4, FLOAT, false, stride, 4*4*4);
    gl.vertex_attrib_pointer_f32(location+4+1, 4, FLOAT, false, stride, 5*4*4);
    gl.vertex_attrib_pointer_f32(location+4+2, 4, FLOAT, false, stride, 6*4*4);
    gl.vertex_attrib_pointer_f32(location+4+3, 4, FLOAT, false, stride, 7*4*4);

    gl.vertex_attrib_pointer_f32(location+4+4, 4, FLOAT, false, stride, 8*4*4);

    gl.bind_buffer(ARRAY_BUFFER, None);
    gl.vertex_attrib_divisor(location+0, 1); // tell OpenGL this is an instanced vertex attribute    
    gl.vertex_attrib_divisor(location+1, 1);
    gl.vertex_attrib_divisor(location+2, 1);
    gl.vertex_attrib_divisor(location+3, 1);

    gl.vertex_attrib_divisor(location+4+0, 1);
    gl.vertex_attrib_divisor(location+4+1, 1);
    gl.vertex_attrib_divisor(location+4+2, 1);
    gl.vertex_attrib_divisor(location+4+3, 1);

    gl.vertex_attrib_divisor(location+4+4, 1);
}

impl Renderable for RenderObject {
    unsafe fn destroy(&self, gl: &Context) { 
        gl.delete_program(self.program);
        gl.delete_vertex_array(self.vao);
        gl.delete_buffer(self.vbo);
    }

    unsafe fn paint(&self, gl: &Context, render_mats: &RenderMatrices, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>, selected_id: Option<u32>) {    
        self.fill_vbo(gl, render_mats, cam_pos, light_pos, selected_id);

        gl.draw_arrays(TRIANGLES, 0, 12*3);
        // gl.draw_arrays(TRIANGLES, 0, 12*3);
        gl.bind_vertex_array(None);
    }

    unsafe fn fill_vbo(&self, gl: &Context, render_mats: &RenderMatrices, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>, selected_id: Option<u32>) {
        gl.use_program(Some(self.program));
        gl.bind_vertex_array(Some(self.vao));

        let light_pos_location = gl.get_uniform_location(self.program, "lightPos");
        let light_pos: [f32; 3] = [light_pos.x, light_pos.y, light_pos.z];
        gl.uniform_3_f32_slice(light_pos_location.as_ref(), &light_pos);

        let uniform_location = gl.get_uniform_location(self.program, "view_mat");
        let f32_mat: [[f32; 4]; 4] = render_mats.view.into();
        gl.uniform_matrix_4_f32_slice(uniform_location.as_ref(), false, f32_mat.into_iter().flatten().collect::<Vec<f32>>().as_ref());

        let perspective_location = gl.get_uniform_location(self.program, "perspective_mat");
        let f32_mat: [[f32; 4]; 4] = render_mats.perspective.into();
        gl.uniform_matrix_4_f32_slice(perspective_location.as_ref(), false, f32_mat.into_iter().flatten().collect::<Vec<f32>>().as_ref());

        let cam_pos_location = gl.get_uniform_location(self.program, "camPos");
        let cam_pos: [f32; 3] = [cam_pos.x, cam_pos.y, cam_pos.z];
        gl.uniform_3_f32_slice(cam_pos_location.as_ref(), &cam_pos);

        let id_location = gl.get_uniform_location(self.program, "selected_id");
        let selected_id: f32 = if selected_id.is_some() { selected_id.unwrap() as f32 } else { -1.0 };
        gl.uniform_1_f32(id_location.as_ref(), selected_id);
    }

    fn is_instanced(&self) -> bool { false }
}

pub struct Canvas {
    domino_obj: InstancedRenderObject,
    light_obj: RenderObject,
    ground_obj: RenderObject,
    simulator: Arc<Mutex<Simulator>>,
}

#[allow(unsafe_code)] // we need unsafe code to use glow
impl Canvas {
    pub fn new(gl: &Context, simulator: Arc<Mutex<Simulator>>) -> Option<Self> {
        unsafe {
            // Create a vertex buffer and vertex array object
            let (domino_obj, light_obj, ground_obj) = init_vertex_buffer(&gl);

            Some(Self {
                domino_obj,
                light_obj,
                ground_obj,
                simulator,
            })
        }
    }

    pub fn destroy(&self, gl: &Context) {
        unsafe {
            self.domino_obj.destroy(gl);
            self.light_obj.destroy(gl);
            self.ground_obj.destroy(gl);
        }
    }

    pub fn paint(&mut self, gl: &Context, render_mats: RenderMatrices, cam_pos: cgmath::Point3<f32>, selected_id: Option<u32>) {
        let light_pos = cgmath::point3(0.0f32, 2.0f32, 0.0f32);

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            // gl.enable(glow::CULL_FACE); TODO: might improve performance
            gl.depth_func(glow::LESS);
            gl.clear_color(0.1, 0.2, 0.3, 1.0);

            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT); 

            let (model_mats, rot_mats) = self.get_model_mats_list();

            self.domino_obj.fill_i_vbo(gl, &model_mats, &rot_mats);
            self.domino_obj.paint(gl, &render_mats, cam_pos, light_pos, selected_id);
            self.light_obj.paint(gl, &render_mats, cam_pos, light_pos, selected_id);
            self.ground_obj.paint(gl, &render_mats, cam_pos, light_pos, selected_id);
        }
    }

    pub fn get_model_mats_list(&self) -> (Vec<(u32, cgmath::Matrix4<f32>)>, Vec<cgmath::Matrix4<f32>>) {
        let mut model_mats: Vec<(u32, cgmath::Matrix4<f32>)> = vec![];
        let mut rot_mats: Vec<cgmath::Matrix4<f32>> = vec![];
        for d in self.simulator.lock().unwrap().dominos.iter() {
            let translation = cgmath::Matrix4::from_translation(cgmath::Vector3{ x: d.position.x, y: d.position.y, z: d.position.z });
            let scale = cgmath::Matrix4::from_nonuniform_scale(d.scale.x, d.scale.y, d.scale.z);
            let rotation_y: cgmath::Matrix4<f32> = cgmath::Matrix4::from_angle_y(cgmath::Deg(d.rotation_y));
            let fall_rotation_axis = rotation_y * cgmath::Vector4::unit_x();
            let fall_rotation = cgmath::Matrix4::from_axis_angle(cgmath::Vector3 {x: fall_rotation_axis.x, y: fall_rotation_axis.y, z: fall_rotation_axis.z}, cgmath::Deg(d.fall_rotation));
            let rotation = fall_rotation * rotation_y;

            model_mats.push((d.id, translation * rotation * scale));
            rot_mats.push(rotation);
        }
        (model_mats, rot_mats)
    }
}

unsafe fn create_program(
    gl: &Context,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> NativeProgram {
    let program = gl.create_program().expect("Cannot create program");
    
    let shader_sources = [
        (VERTEX_SHADER, vertex_shader_source),
        (FRAGMENT_SHADER, fragment_shader_source),
    ];

    let mut shaders = Vec::with_capacity(shader_sources.len());

    for (shader_type, shader_source) in shader_sources.iter() {
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");
        gl.shader_source(shader, shader_source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
        gl.attach_shader(program, shader);
        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    program
}

unsafe fn init_vertex_buffer(gl: &Context) -> (InstancedRenderObject, RenderObject, RenderObject) {
    // We now construct a vertex array to describe the format of the input buffer 
    let domino_program = create_program(&gl, shaders::dominos::VERTEX_SHADER, shaders::dominos::FRAGMENT_SHADER);
    let domino_vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(domino_vao));

    let domino_vertices = shaders::dominos::get_vertices(cgmath::Vector3{x: 0.07, y: 0.14, z: 0.02});
    let domino_vertices_u8: &[u8] = core::slice::from_raw_parts(
        domino_vertices.as_ptr() as *const u8,
        domino_vertices.len() * core::mem::size_of::<f32>(),
    );

    // We construct a buffer
    let domino_vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(ARRAY_BUFFER, Some(domino_vbo));
    gl.buffer_data_u8_slice(ARRAY_BUFFER, domino_vertices_u8, STATIC_DRAW);
    gl.enable_vertex_attrib_array(0); //vec2 stone vertices positions
    gl.vertex_attrib_pointer_f32(0, 3, FLOAT, false, 6*4, 0);
    gl.enable_vertex_attrib_array(1); //vec3 stone vertices normal
    gl.vertex_attrib_pointer_f32(1, 3, FLOAT, false, 6*4, 3*4);

    let domino_i_vbo = gl.create_buffer().unwrap();

    gl.bind_vertex_array(None);
    let dominos = InstancedRenderObject{vbo: domino_vbo, vao: domino_vao, i_vbo: domino_i_vbo, program: domino_program, render_count: 0};


    let light_program = create_program(&gl, shaders::light_source::VERTEX_SHADER, shaders::light_source::FRAGMENT_SHADER);
    let light_vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(light_vao));

    let light_vertices = shaders::light_source::VERTICES;
    let light_vertices_u8: &[u8] = core::slice::from_raw_parts(
        light_vertices.as_ptr() as *const u8,
        light_vertices.len() * core::mem::size_of::<f32>(),
    );

    // We construct a buffer
    let light_vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(ARRAY_BUFFER, Some(light_vbo));
    gl.buffer_data_u8_slice(ARRAY_BUFFER, light_vertices_u8, STATIC_DRAW);
    gl.enable_vertex_attrib_array(0); //vec2 stone vertices positions
    gl.vertex_attrib_pointer_f32(0, 3, FLOAT, false, 6*4, 0);
    gl.enable_vertex_attrib_array(1); //vec3 stone vertices normal
    gl.vertex_attrib_pointer_f32(1, 3, FLOAT, false, 6*4, 3*4);

    gl.bind_vertex_array(None);
    let light = RenderObject{vbo: light_vbo, vao: light_vao, program: light_program};


    let ground_program = create_program(&gl, shaders::ground_plane::VERTEX_SHADER, shaders::ground_plane::FRAGMENT_SHADER);
    let ground_vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(ground_vao));

    let ground_vertices = shaders::ground_plane::VERTICES;
    let ground_vertices_u8: &[u8] = core::slice::from_raw_parts(
        ground_vertices.as_ptr() as *const u8,
        ground_vertices.len() * core::mem::size_of::<f32>(),
    );

    // We construct a buffer
    let ground_vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(ARRAY_BUFFER, Some(ground_vbo));
    gl.buffer_data_u8_slice(ARRAY_BUFFER, ground_vertices_u8, STATIC_DRAW);
    gl.enable_vertex_attrib_array(0); //vec2 stone vertices positions
    gl.vertex_attrib_pointer_f32(0, 3, FLOAT, false, 6*4, 0);
    gl.enable_vertex_attrib_array(1); //vec3 stone vertices normal
    gl.vertex_attrib_pointer_f32(1, 3, FLOAT, false, 6*4, 3*4);

    gl.bind_vertex_array(None);
    let ground = RenderObject{vbo: ground_vbo, vao: ground_vao, program: ground_program};

    (dominos, light, ground)
}
