use eframe::egui_glow::*;
use glow::*;

use crate::ui_3d::shaders;

pub struct Canvas {
    program: Program,
    vertex_array: NativeVertexArray,
    vertex_buffer_object: NativeBuffer,
    instance_vertex_buffer_object: NativeBuffer
}

#[allow(unsafe_code)] // we need unsafe code to use glow
impl Canvas {
    pub fn new(gl: &Context) -> Option<Self> {
        use HasContext as _;

        unsafe {

            let program = create_program(&gl, shaders::dominos::VERTEX_SHADER, shaders::dominos::FRAGMENT_SHADER);
            gl.use_program(Some(program));

            // Create a vertex buffer and vertex array object
            let (vbo, i_vbo, vao) = init_vertex_buffer(&gl);

            Some(Self {
                program,
                vertex_array: vao,
                vertex_buffer_object: vbo,
                instance_vertex_buffer_object: i_vbo
            })
        }
    }

    pub fn destroy(&self, gl: &Context) {
        use HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
            gl.delete_buffer(self.vertex_buffer_object);
        }
    }

    pub fn paint(&self, gl: &Context, mvp: cgmath::Matrix4<f32>) {
        use HasContext as _;
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            // gl.enable(glow::CULL_FACE); TODO: might improve performance
            gl.depth_func(glow::LESS);
            gl.clear_color(0.1, 0.2, 0.3, 1.0);

            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            let count = self.fill_vertex_buffer(gl, mvp);

            gl.draw_arrays_instanced(TRIANGLES, 0, 12*3, count.try_into().unwrap());
            // gl.draw_arrays(TRIANGLES, 0, 12*3);
            gl.bind_vertex_array(None);
        }
    }

    unsafe fn fill_vertex_buffer(&self, gl: &Context, mvp: cgmath::Matrix4<f32>) -> usize {
        gl.use_program(Some(self.program));
        gl.bind_vertex_array(Some(self.vertex_array));

        let mut translations: [f32; 200] = [0.0; 200];
        let mut index = 0;
        for y in -5..5 {
            for x in -5..5 {
                translations[index] = x as f32 * 3.50;
                index += 1;
                translations[index] = y as f32 * 3.50;
                index += 1;
            }
        }
        let translations_u8: &[u8] = core::slice::from_raw_parts(
            translations.as_ptr() as *const u8,
            translations.len() * core::mem::size_of::<f32>(),
        );

        gl.bind_buffer(ARRAY_BUFFER, Some(self.instance_vertex_buffer_object));
        gl.buffer_data_u8_slice(ARRAY_BUFFER, translations_u8, STATIC_DRAW);
        gl.enable_vertex_attrib_array(2); //vec2 stone vertices positions
        gl.vertex_attrib_pointer_f32(2, 2, FLOAT, false, 2*4, 0);

        gl.bind_buffer(ARRAY_BUFFER, None);
        gl.vertex_attrib_divisor(2, 1); // tell OpenGL this is an instanced vertex attribute


        let light_pos_location = gl.get_uniform_location(self.program, "lightPos");
        let light_pos: [f32; 3] = [0.0, 10.0, 0.0];
        gl.uniform_3_f32_slice(light_pos_location.as_ref(), &light_pos);


        let uniform_location = gl.get_uniform_location(self.program, "view_mat");
        let f32_mat: [[f32; 4]; 4] = mvp.into();
        gl.uniform_matrix_4_f32_slice(uniform_location.as_ref(), false, f32_mat.into_iter().flatten().collect::<Vec<f32>>().as_ref());

        100
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

unsafe fn init_vertex_buffer(gl: &Context) -> (NativeBuffer, NativeBuffer, NativeVertexArray) {
    // We now construct a vertex array to describe the format of the input buffer
    
    let vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(vao));

    let quad_vertices = [
    -0.5f32, -0.5f32, -0.5f32,  0.0f32,  0.0f32, -1.0f32,
     0.5f32, -0.5f32, -0.5f32,  0.0f32,  0.0f32, -1.0f32, 
     0.5f32,  0.5f32, -0.5f32,  0.0f32,  0.0f32, -1.0f32, 
     0.5f32,  0.5f32, -0.5f32,  0.0f32,  0.0f32, -1.0f32, 
    -0.5f32,  0.5f32, -0.5f32,  0.0f32,  0.0f32, -1.0f32, 
    -0.5f32, -0.5f32, -0.5f32,  0.0f32,  0.0f32, -1.0f32, 

    -0.5f32, -0.5f32,  0.5f32,  0.0f32,  0.0f32, 1.0f32,
     0.5f32, -0.5f32,  0.5f32,  0.0f32,  0.0f32, 1.0f32,
     0.5f32,  0.5f32,  0.5f32,  0.0f32,  0.0f32, 1.0f32,
     0.5f32,  0.5f32,  0.5f32,  0.0f32,  0.0f32, 1.0f32,
    -0.5f32,  0.5f32,  0.5f32,  0.0f32,  0.0f32, 1.0f32,
    -0.5f32, -0.5f32,  0.5f32,  0.0f32,  0.0f32, 1.0f32,

    -0.5f32,  0.5f32,  0.5f32, -1.0f32,  0.0f32,  0.0f32,
    -0.5f32,  0.5f32, -0.5f32, -1.0f32,  0.0f32,  0.0f32,
    -0.5f32, -0.5f32, -0.5f32, -1.0f32,  0.0f32,  0.0f32,
    -0.5f32, -0.5f32, -0.5f32, -1.0f32,  0.0f32,  0.0f32,
    -0.5f32, -0.5f32,  0.5f32, -1.0f32,  0.0f32,  0.0f32,
    -0.5f32,  0.5f32,  0.5f32, -1.0f32,  0.0f32,  0.0f32,

     0.5f32,  0.5f32,  0.5f32,  1.0f32,  0.0f32,  0.0f32,
     0.5f32,  0.5f32, -0.5f32,  1.0f32,  0.0f32,  0.0f32,
     0.5f32, -0.5f32, -0.5f32,  1.0f32,  0.0f32,  0.0f32,
     0.5f32, -0.5f32, -0.5f32,  1.0f32,  0.0f32,  0.0f32,
     0.5f32, -0.5f32,  0.5f32,  1.0f32,  0.0f32,  0.0f32,
     0.5f32,  0.5f32,  0.5f32,  1.0f32,  0.0f32,  0.0f32,

    -0.5f32, -0.5f32, -0.5f32,  0.0f32, -1.0f32,  0.0f32,
     0.5f32, -0.5f32, -0.5f32,  0.0f32, -1.0f32,  0.0f32,
     0.5f32, -0.5f32,  0.5f32,  0.0f32, -1.0f32,  0.0f32,
     0.5f32, -0.5f32,  0.5f32,  0.0f32, -1.0f32,  0.0f32,
    -0.5f32, -0.5f32,  0.5f32,  0.0f32, -1.0f32,  0.0f32,
    -0.5f32, -0.5f32, -0.5f32,  0.0f32, -1.0f32,  0.0f32,

    -0.5f32,  0.5f32, -0.5f32,  0.0f32,  1.0f32,  0.0f32,
     0.5f32,  0.5f32, -0.5f32,  0.0f32,  1.0f32,  0.0f32,
     0.5f32,  0.5f32,  0.5f32,  0.0f32,  1.0f32,  0.0f32,
     0.5f32,  0.5f32,  0.5f32,  0.0f32,  1.0f32,  0.0f32,
    -0.5f32,  0.5f32,  0.5f32,  0.0f32,  1.0f32,  0.0f32,
    -0.5f32,  0.5f32, -0.5f32,  0.0f32,  1.0f32,  0.0f32
    ];
    let quad_vertices_u8: &[u8] = core::slice::from_raw_parts(
        quad_vertices.as_ptr() as *const u8,
        quad_vertices.len() * core::mem::size_of::<f32>(),
    );

    // We construct a buffer
    let quad_vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(ARRAY_BUFFER, Some(quad_vbo));
    gl.buffer_data_u8_slice(ARRAY_BUFFER, quad_vertices_u8, STATIC_DRAW);
    gl.enable_vertex_attrib_array(0); //vec2 stone vertices positions
    gl.vertex_attrib_pointer_f32(0, 3, FLOAT, false, 6*4, 0);
    gl.enable_vertex_attrib_array(1); //vec3 stone vertices normal
    gl.vertex_attrib_pointer_f32(1, 3, FLOAT, false, 6*4, 3*4);

    let instance_vbo = gl.create_buffer().unwrap();

    gl.bind_vertex_array(None);
    (quad_vbo, instance_vbo, vao)
}
