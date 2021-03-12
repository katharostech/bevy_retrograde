use bevy::{
    app::{Events, ManualEventReader},
    prelude::*,
    utils::HashMap,
    window::{WindowCreated, WindowId},
};

#[cfg(not(wasm))]
use glutin::{ContextBuilder, NotCurrent, RawContext};

#[cfg(unix)]
use glutin::platform::unix::{RawContextExt, WindowExtUnix};

#[cfg(windows)]
use glutin::platform::windows::{RawContextExt, WindowExtUnix};

use glow::HasContext;

#[derive(Clone, Debug)]
pub struct RetroRenderOptions {
    pub viewport_scaling_mode: ViewPortScalingMode,
    pub background_color: (f32, f32, f32, f32),
}

#[derive(Clone, Debug)]
pub enum ViewPortScalingMode {
    FixedVertical,
    FixedHorizontal,
}

impl Default for ViewPortScalingMode {
    fn default() -> Self {
        Self::FixedVertical
    }
}

impl Default for RetroRenderOptions {
    fn default() -> Self {
        DEFAULT_RENDER_OPTIONS.clone()
    }
}

const DEFAULT_RENDER_OPTIONS: RetroRenderOptions = RetroRenderOptions {
    background_color: (0.0, 0.0, 0.0, 0.0),
    viewport_scaling_mode: ViewPortScalingMode::FixedVertical,
};

pub(crate) fn get_render_system() -> impl FnMut(&mut World) {
    let mut renderer = RetroRenderer::default();

    move |world| {
        renderer.update(world);
    }
}

pub trait SliceAsBytes<T> {
    fn as_mem_bytes(&self) -> &[u8];
}

impl<T: AsRef<[U]>, U> SliceAsBytes<U> for T {
    fn as_mem_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ref().as_ptr() as *const u8,
                std::mem::size_of::<T>() * self.as_ref().len(),
            )
        }
    }
}

#[derive(Default)]
struct RetroRenderer {
    #[cfg(not(wasm))]
    pub window_contexts: HashMap<bevy::window::WindowId, RawContext<NotCurrent>>,
    #[cfg(not(wasm))]
    pub gl_handles: HashMap<bevy::window::WindowId, glow::Context>,
    pub gl_objects: HashMap<bevy::window::WindowId, GlObjects>,
    pub window_created_event_reader: ManualEventReader<WindowCreated>,
}

struct GlObjects {
    shader_program: <glow::Context as HasContext>::Shader,
    vao: <glow::Context as HasContext>::VertexArray,
}

impl RetroRenderer {
    /// Handle window creation
    fn handle_window_create_events(&mut self, world: &mut World) {
        // Get all the windows in the world
        let windows = world.get_resource::<Windows>().unwrap();
        let window_created_events = world.get_resource::<Events<WindowCreated>>().unwrap();

        let render_options = world
            .get_resource::<RetroRenderOptions>()
            .as_deref()
            .cloned()
            .unwrap_or(DEFAULT_RENDER_OPTIONS);

        // Loop through each window creation event
        for window_created_event in self
            .window_created_event_reader
            .iter(&window_created_events)
        {
            unsafe {
                // Get the window that was created
                let window = windows
                    .get(window_created_event.id)
                    .expect("Received window created event for non-existent window.");

                #[cfg(not(wasm))]
                let (gl, context) = {
                    let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
                    let winit_window = winit_windows.get_window(window.id()).unwrap();

                    // Create the raw context from the winit window
                    let context = ContextBuilder::new()
                        .build_raw_x11_context(
                            winit_window
                                .xlib_xconnection()
                                .expect("TODO: Support non-x11 windows"),
                            winit_window
                                .xlib_window()
                                .expect("TODO: Support non-x11 windows"),
                        )
                        .expect("TODO: handle error");

                    // Make the new context current
                    let context = context.make_current().unwrap();

                    // Get the gl function pointer
                    (
                        glow::Context::from_loader_function(|s| {
                            context.get_proc_address(s) as *const _
                        }),
                        context,
                    )
                };

                #[cfg(wasm)]
                let gl = get_wasm_gl_handle(window.id(), world);

                #[cfg(not(wasm))]
                const SHADER_VERSION: &str = "330 core";
                #[cfg(wasm)]
                const SHADER_VERSION: &str = "300 es";

                const VERTEX_SHADER_SRC: &str = include_str!("shaders/viewport.vert");
                const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/viewport.frag");
                const QUAD_VERT_POS: &[f32] = &[
                    -1.0, 1.0, 0.0,
                    1.0, 1.0, 0.0,
                    -1.0, -1.0, 0.0,
                    1.0, 1.0, 0.0,
                    1.0, -1.0, 0.0,
                    -1.0, -1.0, 0.0
                ];

                //
                // Create vertext array and vertex buffer
                //

                // Create vertex array object that will store our vertex attribute
                // config like a "preset".
                let vao = gl.create_vertex_array().unwrap();

                // Create vertex buffer object that stores the actuall vertex data
                let vbo = gl.create_buffer().unwrap();

                // Bind the VAO so that all vertex attribute operations will be
                // recorded in that VAO
                gl.bind_vertex_array(Some(vao));

                // Bind the vbo as the ARRAY_BUFFER
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

                // Upload vertex data to the VBO
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    QUAD_VERT_POS.as_mem_bytes(),
                    glow::STATIC_DRAW,
                );

                // Describe our vertex attribute data format
                gl.vertex_attrib_pointer_f32(
                    // Corresponds to `location = 0` in the vertex shader
                    0,
                    // The number of components in our attribute ( 3 values in a Vec3 )
                    3,
                    // The data type
                    glow::FLOAT,
                    // makes integer types normalized to 0 and 1 when converting to float
                    false,
                    // The space between each vertex attribute and the next
                    3 * std::mem::size_of::<f32>() as i32,
                    // The offset since the beginning of the buffer to look for the attribute
                    0,
                );

                // Enable the position vertex attribute
                gl.enable_vertex_attrib_array(
                    // also corresponds to `location = 0` in the vertex shader */
                    0,
                );

                //
                // Create shader pipeline
                //

                // Create a shader program to link our shaders to
                let shader_program = gl.create_program().unwrap();

                // Create a vertex shader
                let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
                // Load the shader's GLSL source
                gl.shader_source(
                    vertex_shader,
                    &VERTEX_SHADER_SRC.replace("{{version}}", SHADER_VERSION),
                );
                // Compile the vertex shader
                gl.compile_shader(vertex_shader);
                // Check for shader compile errors
                handle_shader_compile_errors(&gl, vertex_shader);

                // Create a fragment shader
                let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
                // Load the shader's GLSL source
                gl.shader_source(
                    fragment_shader,
                    &FRAGMENT_SHADER_SRC.replace("{{version}}", SHADER_VERSION),
                );
                // Compile the fragment shader
                gl.compile_shader(fragment_shader);
                handle_shader_compile_errors(&gl, fragment_shader);

                // Add both shaders to the program
                gl.attach_shader(shader_program, vertex_shader);
                gl.attach_shader(shader_program, fragment_shader);
                // Link the program
                gl.link_program(shader_program);
                // Handle link errors
                handle_program_link_errors(&gl, shader_program);

                // Delete our shader objects. Now that they are linked we don't need them.
                gl.delete_shader(vertex_shader);
                gl.delete_shader(fragment_shader);

                // Make the program the current program
                gl.use_program(Some(shader_program));

                // Set the clear color
                gl.clear_color(
                    render_options.background_color.0,
                    render_options.background_color.1,
                    render_options.background_color.2,
                    render_options.background_color.3,
                );

                #[cfg(not(wasm))]
                {
                    let context = context.treat_as_not_current();
                    self.window_contexts.insert(window.id(), context);
                    self.gl_objects.insert(
                        window.id(),
                        GlObjects {
                            shader_program,
                            vao,
                        },
                    );
                    self.gl_handles.insert(window.id(), gl);
                }
            }
        }
    }

    fn update(&mut self, world: &mut World) {
        self.handle_window_create_events(world);

        for window in world.get_resource::<Windows>().unwrap().iter() {
            unsafe {
                #[cfg(not(wasm))]
                let (gl, context) = {
                    let context = self.window_contexts.remove(&window.id()).unwrap();
                    let context = context.make_current().unwrap();
                    let gl = self.gl_handles.get(&window.id()).unwrap();

                    (gl, context)
                };

                #[cfg(wasm)]
                let gl = get_wasm_gl_handle(window.id(), world);

                gl.clear(glow::COLOR_BUFFER_BIT);
                gl.draw_arrays(glow::TRIANGLES, 0, 6);

                #[cfg(not(wasm))]
                {
                    context.swap_buffers().unwrap();
                    let context = context.treat_as_not_current();
                    self.window_contexts.insert(window.id(), context);
                }
            }
        }
    }

    // TODO: Drop gl resources when the app exits
}

#[cfg(wasm)]
fn get_wasm_gl_handle(window_id: WindowId, world: &World) -> glow::Context {
    use wasm_bindgen::JsCast;
    use winit::platform::web::WindowExtWebSys;

    let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
    let winit_window = winit_windows.get_window(window_id).unwrap();

    let webgl2_context = winit_window
        .canvas()
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .unwrap();

    glow::Context::from_webgl2_context(webgl2_context)
}

fn handle_shader_compile_errors(gl: &glow::Context, shader: <glow::Context as HasContext>::Shader) {
    unsafe {
        if !gl.get_shader_compile_status(shader) {
            panic!("Shader compile error: {}", gl.get_shader_info_log(shader));
        }
    }
}

fn handle_program_link_errors(gl: &glow::Context, program: <glow::Context as HasContext>::Program) {
    unsafe {
        if !gl.get_program_link_status(program) {
            panic!("Shader link error: {}", gl.get_program_info_log(program));
        }
    }
}
