use std::sync::Arc;
use eframe::egui_glow;
use egui::mutex::Mutex;
use egui_glow::glow;
use glow::*;

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

        let right_vec = cgmath::Vector3{
            x: cgmath::Angle::sin(cgmath::Rad(self.cam_angle.x - 90.0)),
            y: 0.0,
            z: cgmath::Angle::cos(cgmath::Rad(self.cam_angle.x - 90.0))
        };

        // let mut up_vec = right_vec.cross(direction);
        let up_vec = cgmath::Vector3{x: 0.0, y: 1.0, z: 0.0};

        if keys_down.contains(&egui::Key::S) {
            self.cam_pos -= move_direction * 0.01f32;
        }
        if keys_down.contains(&egui::Key::W) {
            self.cam_pos += move_direction * 0.01f32;
        }
        if keys_down.contains(&egui::Key::D) {
            self.cam_pos += right_vec * 0.01f32;
        }
        if keys_down.contains(&egui::Key::A) {
            self.cam_pos -= right_vec * 0.01f32;
        }
        if mods.shift {
            self.cam_pos += up_vec * 0.01f32;
        }
        if mods.ctrl {
            self.cam_pos -= up_vec * 0.01f32;
        }

        let proj_mat = cgmath::perspective(cgmath::Deg(60f32), screen_rect.aspect_ratio(), 0.1f32, 100f32);
        let view_mat = cgmath::Matrix4::look_at_rh(self.cam_pos, self.cam_pos + direction, up_vec);
        let mat = proj_mat * view_mat;
        mat
    }
}

struct Canvas {
    program: Program,
    vertex_array: NativeVertexArray,
    vertex_buffer_object: NativeBuffer,
    instance_vertex_buffer_object: NativeBuffer
}

#[allow(unsafe_code)] // we need unsafe code to use glow
impl Canvas {
    fn new(gl: &Context) -> Option<Self> {
        use HasContext as _;

        unsafe {
            let (vertex_shader_source, fragment_shader_source) = (
                r#"#version 330 core
                layout (location = 0) in vec3 pos_model_space;
                layout (location = 1) in vec3 aColor;
                layout (location = 2) in vec2 aOffset;

                uniform mat4 view_mat;

                out vec3 fColor;

                void main()
                {
                    fColor = aColor;
                    gl_Position = view_mat * vec4(pos_model_space.x + aOffset.x, pos_model_space.y, pos_model_space.z + aOffset.y, 1.0);
                }
                "#,
                r#"#version 330 core
                out vec4 FragColor;

                in vec3 fColor;

                void main()
                {
                    FragColor = vec4(fColor, 1.0);
                }
                "#);

            let program = create_program(&gl, vertex_shader_source, fragment_shader_source);
            gl.use_program(Some(program));

            // Create a vertex buffer and vertex array object
            let (vbo, i_vbo, vao) = init_vertex_buffer(&gl);
            
            gl.clear_color(0.1, 0.2, 0.3, 1.0);

            Some(Self {
                program,
                vertex_array: vao,
                vertex_buffer_object: vbo,
                instance_vertex_buffer_object: i_vbo
            })
        }
    }

    fn destroy(&self, gl: &Context) {
        use HasContext as _;
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
            gl.delete_buffer(self.vertex_buffer_object);
        }
    }

    fn paint(&self, gl: &Context, mvp: cgmath::Matrix4<f32>) {
        use HasContext as _;
        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT);

            let count = self.fill_vertex_buffer(gl, mvp);

            gl.draw_arrays_instanced(TRIANGLES, 0, 12*3, count.try_into().unwrap());
            gl.bind_vertex_array(None);
        }
    }

    unsafe fn fill_vertex_buffer(&self, gl: &Context, mvp: cgmath::Matrix4<f32>) -> usize {
        gl.use_program(Some(self.program));
        gl.bind_vertex_array(Some(self.vertex_array));

        let mut translations: [f32; 200] = [0.0; 200];
        let offset = 2f32;
        let mut index = 0;
        for y in -5..5 {
            for x in -5..5 {
                translations[index] = x as f32 * 4.0 + offset;
                index += 1;
                translations[index] = y as f32 * 4.0 + offset;
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
        -1.0f32,-1.0f32,-1.0f32,   1.0f32, 0.0f32, 0.0f32,
        -1.0f32,-1.0f32, 1.0f32,   0.0f32, 1.0f32, 0.0f32,
        -1.0f32, 1.0f32, 1.0f32,   0.0f32, 0.0f32, 1.0f32,
        1.0f32, 1.0f32,-1.0f32,    1.0f32, 0.0f32, 0.0f32,
        -1.0f32,-1.0f32,-1.0f32,   0.0f32, 1.0f32, 0.0f32,
        -1.0f32, 1.0f32,-1.0f32,   0.0f32, 0.0f32, 1.0f32,
        1.0f32,-1.0f32, 1.0f32,    1.0f32, 0.0f32, 0.0f32,
        -1.0f32,-1.0f32,-1.0f32,   0.0f32, 1.0f32, 0.0f32,
        1.0f32,-1.0f32,-1.0f32,    0.0f32, 0.0f32, 1.0f32,
        1.0f32, 1.0f32,-1.0f32,    1.0f32, 0.0f32, 0.0f32,
        1.0f32,-1.0f32,-1.0f32,    0.0f32, 1.0f32, 0.0f32,
        -1.0f32,-1.0f32,-1.0f32,   0.0f32, 0.0f32, 1.0f32,
        -1.0f32,-1.0f32,-1.0f32,   1.0f32, 0.0f32, 0.0f32,
        -1.0f32, 1.0f32, 1.0f32,   0.0f32, 1.0f32, 0.0f32,
        -1.0f32, 1.0f32,-1.0f32,   0.0f32, 0.0f32, 1.0f32,
        1.0f32,-1.0f32, 1.0f32,    1.0f32, 0.0f32, 0.0f32,
        -1.0f32,-1.0f32, 1.0f32,   0.0f32, 1.0f32, 0.0f32,
        -1.0f32,-1.0f32,-1.0f32,   0.0f32, 0.0f32, 1.0f32,
        -1.0f32, 1.0f32, 1.0f32,   1.0f32, 0.0f32, 0.0f32,
        -1.0f32,-1.0f32, 1.0f32,   0.0f32, 1.0f32, 0.0f32,
        1.0f32,-1.0f32, 1.0f32,    0.0f32, 0.0f32, 1.0f32,
        1.0f32, 1.0f32, 1.0f32,    1.0f32, 0.0f32, 0.0f32,
        1.0f32,-1.0f32,-1.0f32,    0.0f32, 1.0f32, 0.0f32,
        1.0f32, 1.0f32,-1.0f32,    0.0f32, 0.0f32, 1.0f32,
        1.0f32,-1.0f32,-1.0f32,    1.0f32, 0.0f32, 0.0f32,
        1.0f32, 1.0f32, 1.0f32,    0.0f32, 1.0f32, 0.0f32,
        1.0f32,-1.0f32, 1.0f32,    0.0f32, 0.0f32, 1.0f32,
        1.0f32, 1.0f32, 1.0f32,    1.0f32, 0.0f32, 0.0f32,
        1.0f32, 1.0f32,-1.0f32,    0.0f32, 1.0f32, 0.0f32,
        -1.0f32, 1.0f32,-1.0f32,   0.0f32, 0.0f32, 1.0f32,
        1.0f32, 1.0f32, 1.0f32,    1.0f32, 0.0f32, 0.0f32,
        -1.0f32, 1.0f32,-1.0f32,   0.0f32, 1.0f32, 0.0f32,
        -1.0f32, 1.0f32, 1.0f32,   0.0f32, 0.0f32, 1.0f32,
        1.0f32, 1.0f32, 1.0f32,    1.0f32, 0.0f32, 0.0f32,
        -1.0f32, 1.0f32, 1.0f32,   0.0f32, 1.0f32, 0.0f32,
        1.0f32,-1.0f32, 1.0f32,    0.0f32, 0.0f32, 1.0f32,
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
    gl.enable_vertex_attrib_array(1); //vec3 stone vertices color
    gl.vertex_attrib_pointer_f32(1, 3, FLOAT, false, 6*4, 3*4);

    let instance_vbo = gl.create_buffer().unwrap();

    gl.bind_vertex_array(None);
    (quad_vbo, instance_vbo, vao)
}
