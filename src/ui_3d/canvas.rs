use eframe::egui_glow::*;
use glow::*;

use crate::ui_3d::shaders;

trait Renderable {
    unsafe fn destroy(&self, gl: &Context);
    unsafe fn paint(&self, gl: &Context, mvp: cgmath::Matrix4<f32>, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>);
    unsafe fn fill_vbo(&self, gl: &Context, mvp: cgmath::Matrix4<f32>, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>) -> usize;
    fn is_instanced(&self) -> bool;
    unsafe fn fill_i_vbo(&self, _gl: &Context, _values: &Vec<f32>) {
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
    i_vbo: NativeBuffer
}

impl Renderable for InstancedRenderObject {
    unsafe fn destroy(&self, gl: &Context) {
        gl.delete_program(self.program);
        gl.delete_vertex_array(self.vao);
        gl.delete_buffer(self.vbo);
        gl.delete_buffer(self.i_vbo);
    }

    unsafe fn paint(&self, gl: &Context, mvp: cgmath::Matrix4<f32>, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>) {
        let count = self.fill_vbo(gl, mvp, cam_pos, light_pos);

        gl.draw_arrays_instanced(TRIANGLES, 0, 12*3, count.try_into().unwrap());
        // gl.draw_arrays(TRIANGLES, 0, 12*3);
        gl.bind_vertex_array(None);
    }

    unsafe fn fill_vbo(&self, gl: &Context, mvp: cgmath::Matrix4<f32>, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>) -> usize {
        gl.use_program(Some(self.program));
        gl.bind_vertex_array(Some(self.vao));

        let light_pos_location = gl.get_uniform_location(self.program, "lightPos");
        let light_pos: [f32; 3] = [light_pos.x, light_pos.y, light_pos.z];
        gl.uniform_3_f32_slice(light_pos_location.as_ref(), &light_pos);

        let uniform_location = gl.get_uniform_location(self.program, "view_mat");
        let f32_mat: [[f32; 4]; 4] = mvp.into();
        gl.uniform_matrix_4_f32_slice(uniform_location.as_ref(), false, f32_mat.into_iter().flatten().collect::<Vec<f32>>().as_ref());

        let cam_pos_location = gl.get_uniform_location(self.program, "camPos");
        let cam_pos: [f32; 3] = [cam_pos.x, cam_pos.y, cam_pos.z];
        gl.uniform_3_f32_slice(cam_pos_location.as_ref(), &cam_pos);

        100
    }

    unsafe fn fill_i_vbo(&self, gl: &Context, values: &Vec<f32>) { 
        gl.use_program(Some(self.program));
        gl.bind_vertex_array(Some(self.vao));

        let values_u8: &[u8] = core::slice::from_raw_parts(
            values.as_slice().as_ptr() as *const u8,
            values.len() * core::mem::size_of::<f32>(),
        );

        gl.bind_buffer(ARRAY_BUFFER, Some(self.i_vbo));
        gl.buffer_data_u8_slice(ARRAY_BUFFER, values_u8, STATIC_DRAW);
        gl.enable_vertex_attrib_array(2); //vec2 stone vertices positions
        gl.vertex_attrib_pointer_f32(2, 2, FLOAT, false, 2*4, 0);

        gl.bind_buffer(ARRAY_BUFFER, None);
        gl.vertex_attrib_divisor(2, 1); // tell OpenGL this is an instanced vertex attribute

    }

    fn is_instanced(&self) -> bool { true }
}

impl Renderable for RenderObject {
    unsafe fn destroy(&self, gl: &Context) { 
        gl.delete_program(self.program);
        gl.delete_vertex_array(self.vao);
        gl.delete_buffer(self.vbo);
    }

    unsafe fn paint(&self, gl: &Context, mvp: cgmath::Matrix4<f32>, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>) {    
        self.fill_vbo(gl, mvp, cam_pos, light_pos);

        gl.draw_arrays(TRIANGLES, 0, 12*3);
        // gl.draw_arrays(TRIANGLES, 0, 12*3);
        gl.bind_vertex_array(None);
    }

    unsafe fn fill_vbo(&self, gl: &Context, mvp: cgmath::Matrix4<f32>, cam_pos: cgmath::Point3<f32>, light_pos: cgmath::Point3<f32>) -> usize {
        gl.use_program(Some(self.program));
        gl.bind_vertex_array(Some(self.vao));

        let light_pos_location = gl.get_uniform_location(self.program, "light_pos");
        let light_pos: [f32; 3] = [light_pos.x, light_pos.y, light_pos.z];
        gl.uniform_3_f32_slice(light_pos_location.as_ref(), &light_pos);

        let uniform_location = gl.get_uniform_location(self.program, "view_mat");
        let f32_mat: [[f32; 4]; 4] = mvp.into();
        gl.uniform_matrix_4_f32_slice(uniform_location.as_ref(), false, f32_mat.into_iter().flatten().collect::<Vec<f32>>().as_ref());

        let cam_pos_location = gl.get_uniform_location(self.program, "camPos");
        let cam_pos: [f32; 3] = [cam_pos.x, cam_pos.y, cam_pos.z];
        gl.uniform_3_f32_slice(cam_pos_location.as_ref(), &cam_pos);

        1
    }

    fn is_instanced(&self) -> bool { false }
}

pub struct Canvas {
    domino_obj: InstancedRenderObject,
    light_obj: RenderObject,
    ground_obj: RenderObject,
}

#[allow(unsafe_code)] // we need unsafe code to use glow
impl Canvas {
    pub fn new(gl: &Context) -> Option<Self> {
        unsafe {
            // Create a vertex buffer and vertex array object
            let (domino_obj, light_obj, ground_obj) = init_vertex_buffer(&gl);

            Some(Self {
                domino_obj,
                light_obj,
                ground_obj
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

    pub fn paint(&self, gl: &Context, mvp: cgmath::Matrix4<f32>, cam_pos: cgmath::Point3<f32>) {
        let light_pos = cgmath::point3(0.0f32, 2.0f32, 1.0f32);

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            // gl.enable(glow::CULL_FACE); TODO: might improve performance
            gl.depth_func(glow::LESS);
            gl.clear_color(0.1, 0.2, 0.3, 1.0);

            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        
            

            let mut translations: Vec<f32> = vec![];
            for y in -5..5 {
                for x in -5..5 {
                    translations.push(x as f32 * 3.50);
                    translations.push(y as f32 * 3.50);
                }
            }
            self.domino_obj.fill_i_vbo(gl, &translations);
            self.domino_obj.paint(gl, mvp, cam_pos, light_pos);
            self.light_obj.paint(gl, mvp, cam_pos, light_pos);
            self.ground_obj.paint(gl, mvp, cam_pos, light_pos);
        }
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

    let domino_vertices = shaders::dominos::VERTICES;
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
    let dominos = InstancedRenderObject{vbo: domino_vbo, vao: domino_vao, i_vbo: domino_i_vbo, program: domino_program};


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
