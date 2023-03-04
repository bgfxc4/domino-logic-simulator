use std::sync::Arc;

use eframe::egui_glow;
use egui::mutex::Mutex;
use egui_glow::glow;
use glow::*;

pub struct UI3d {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    rotating_triangle: Arc<Mutex<RotatingTriangle>>,
    angle: f32,
}

impl UI3d {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
        let gl = cc.gl.as_ref()?;
        Some(Self {
            rotating_triangle: Arc::new(Mutex::new(RotatingTriangle::new(gl)?)),
            angle: 0.0,
        })
    }
}

impl eframe::App for UI3d {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.custom_painting(ui);
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.rotating_triangle.lock().destroy(gl);
        }
    }
}

impl UI3d {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());

        self.angle = match response.hover_pos() {
            Some(pos) => pos.x,
            None => 0.0
        };

        // Clone locals so we can move them into the paint callback:
        let angle = self.angle;
        let rotating_triangle = self.rotating_triangle.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            rotating_triangle.lock().paint(painter.gl(), angle);
        });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }
}

struct RotatingTriangle {
    program: Program,
    vertex_array: VertexArray,
    vertex_buffer_object: NativeBuffer
}

#[allow(unsafe_code)] // we need unsafe code to use glow
impl RotatingTriangle {
    fn new(gl: &Context) -> Option<Self> {
        use HasContext as _;

        let shader_version = egui_glow::ShaderVersion::get(gl);
        println!("{}", shader_version.version_declaration());
        unsafe {
            let (vertex_shader_source, fragment_shader_source) = (
                format!("{}\n{}", shader_version.version_declaration(), r#"
                    in vec3 in_position;
                    in vec2 in_x_y_rot;

                    out vec2 position;

                    void main() {
                        position = vec2(in_position.x, in_position.y);
                        gl_Position = vec4(in_position.x - 0.5, in_position.y - 0.5, 0.0, 1.0);
                    }
                "#),
                format!("{}\n{}", shader_version.version_declaration(), r#"
                    precision mediump float;
                    in vec2 position;
                    out vec4 color;
                    uniform float blue;
                    void main() {
                        color = vec4(position, blue, 1.0);
                    }
                "#),
            );

            let program = create_program(&gl, vertex_shader_source.as_str(), fragment_shader_source.as_str());
            gl.use_program(Some(program));

            // Create a vertex buffer and vertex array object
            let (vbo, vao) = init_vertex_buffer(&gl);
            
            gl.clear_color(0.1, 0.2, 0.3, 1.0);

            // Upload some uniforms
            set_uniform(&gl, program, "blue", 0.8);

            Some(Self {
                program,
                vertex_array: vao,
                vertex_buffer_object: vbo,
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

    fn paint(&self, gl: &Context, angle: f32) {
        use HasContext as _;
        unsafe {
            // gl.clear(COLOR_BUFFER_BIT);

            gl.use_program(Some(self.program));

            gl.clear(glow::COLOR_BUFFER_BIT);

            self.fill_vertex_buffer(gl, angle);

            gl.draw_arrays(LINE_LOOP, 0, 4);
        }
    }

    unsafe fn fill_vertex_buffer(&self, gl: &Context, angle: f32) {
        // let triangle_vertices = [0.5f32, 1.0f32, 0.0f32, 0.0f32, 1.0f32, (angle/100.0) % 3.0];
        // let triangle_vertices_u8: &[u8] = core::slice::from_raw_parts(
        //     triangle_vertices.as_ptr() as *const u8,
        //     triangle_vertices.len() * core::mem::size_of::<f32>(),
        // );
        let stone_vertices = [
            0.0f32, 0.0f32, 5.0f32,  5.0f32, 5.0f32,
            1.0f32, 0.0f32, 5.0f32,  5.0f32, 5.0f32,
            1.0f32, 0.5f32, 0.0f32,  0.0f32, 0.0f32,
            0.5f32, 1.0f32, 0.0f32,  0.0f32, 0.0f32,
        ];
        let stone_vertices_u8: &[u8] = core::slice::from_raw_parts(
            stone_vertices.as_ptr() as *const u8,
            stone_vertices.len() * core::mem::size_of::<f32>(),
        );

        // upload the data
        gl.bind_buffer(ARRAY_BUFFER, Some(self.vertex_buffer_object));
        gl.buffer_data_u8_slice(ARRAY_BUFFER, stone_vertices_u8, STATIC_DRAW);

        // We now construct a vertex array to describe the format of the input buffer
        gl.bind_vertex_array(Some(self.vertex_array));
        
        gl.enable_vertex_attrib_array(0); //vec3 stone origin pos
        gl.vertex_attrib_pointer_f32(0, 3, FLOAT, false, 20, 0);

        gl.enable_vertex_attrib_array(1); //vec2 stone x and y rot
        gl.vertex_attrib_pointer_f32(1, 2, FLOAT, false, 20, 12);
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

unsafe fn init_vertex_buffer(gl: &Context) -> (NativeBuffer, NativeVertexArray) {
    // We now construct a vertex array to describe the format of the input buffer
    let vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(vao));

    // We construct a buffer
    let vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(ARRAY_BUFFER, Some(vbo));

    (vbo, vao)
}

unsafe fn set_uniform(gl: &Context, program: NativeProgram, name: &str, value: f32) {
    let uniform_location = gl.get_uniform_location(program, name);
    // See also `uniform_n_i32`, `uniform_n_u32`, `uniform_matrix_4_f32_slice` etc.
    gl.uniform_1_f32(uniform_location.as_ref(), value)
}
