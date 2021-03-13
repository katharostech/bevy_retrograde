use bevy::{
    app::{Events, ManualEventReader},
    log::*,
    prelude::*,
    utils::HashMap,
    window::{WindowCreated, WindowResized},
};

#[cfg(not(wasm))]
use glutin::{ContextBuilder, NotCurrent, RawContext};

#[cfg(unix)]
use glutin::platform::unix::{RawContextExt, WindowExtUnix};

#[cfg(windows)]
use glutin::platform::windows::{RawContextExt, WindowExtUnix};

use glow::HasContext;
use winit::dpi::LogicalSize;

use crate::SpriteImage;

#[derive(Clone, Debug)]
pub struct RetroRenderOptions {
    pub viewport_scaling_mode: ViewPortScalingMode,
    pub background_color: (f32, f32, f32, f32),
}

impl Default for RetroRenderOptions {
    fn default() -> Self {
        Self {
            viewport_scaling_mode: Default::default(),
            background_color: (0.0, 0.0, 0.0, 1.0),
        }
    }
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

#[derive(Default, Debug, Clone)]
pub(crate) struct RetroRenderImage(image::RgbaImage);

pub(crate) fn pre_render_system(
    sprite_image_handles: Query<&Handle<SpriteImage>>,
    sprite_image_assets: Res<Assets<SpriteImage>>,
    mut render_image: ResMut<RetroRenderImage>,
) {
    if let Some(handle) = sprite_image_handles.iter().next() {
        if let Some(sprite) = sprite_image_assets.get(handle) {
            *render_image = RetroRenderImage(sprite.image.clone());
        } else {
            *render_image = RetroRenderImage(Default::default());
        }
    }
}

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
    pub gl_handles: HashMap<bevy::window::WindowId, glow::Context>,
    pub gl_objects: HashMap<bevy::window::WindowId, GlObjects>,
    pub window_created_event_reader: ManualEventReader<WindowCreated>,
    pub window_resized_event_reader: ManualEventReader<WindowResized>,
}

/// # Safety
/// Safe because WASM doesn't have threads anyway
#[cfg(wasm)]
unsafe impl Send for RetroRenderer {}
/// # Safety
/// Safe because WASM doesn't have threads anyway
#[cfg(wasm)]
unsafe impl Sync for RetroRenderer {}

struct GlObjects {
    shader_program: <glow::Context as HasContext>::Program,
    vao: <glow::Context as HasContext>::VertexArray,
}

impl RetroRenderer {
    /// Handle window resize
    fn handle_window_resize_events(&mut self, world: &mut World) {
        // Get all the windows in the world
        let windows = world.get_resource::<Windows>().unwrap();
        let window_resized_events = world.get_resource::<Events<WindowResized>>().unwrap();

        // for every window resize event
        for window_resized_event in self
            .window_resized_event_reader
            .iter(&window_resized_events)
        {
            debug!("{:#?}", window_resized_event);

            unsafe {
                let window_id = window_resized_event.id;
                let window = windows.get(window_id).unwrap();

                // Resize the GL context ( must be manually done for MacOS and Wayland )
                #[cfg(not(wasm))]
                let context = {
                    let context = self.window_contexts.remove(&window_id).unwrap();

                    let context = context.make_current().unwrap();

                    context.resize(glutin::dpi::PhysicalSize {
                        width: window.physical_width(),
                        height: window.physical_height(),
                    });

                    context
                };

                // Resize the GL viewport
                let gl = self.gl_handles.get(&window_id).unwrap();
                gl.viewport(0, 0, window.width() as i32, window.height() as i32);

                #[cfg(not(wasm))]
                {
                    self.window_contexts
                        .insert(window_id, context.treat_as_not_current());
                }
            }
        }
    }

    /// Handle window creation
    fn handle_window_create_events(&mut self, world: &mut World) {
        let render_options = world.get_resource::<RetroRenderOptions>().unwrap();

        // Get all the windows in the world
        let windows = world.get_resource::<Windows>().unwrap();
        let window_created_events = world.get_resource::<Events<WindowCreated>>().unwrap();

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
                let gl = {
                    use wasm_bindgen::JsCast;
                    use winit::platform::web::WindowExtWebSys;

                    let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
                    let winit_window = winit_windows.get_window(window.id()).unwrap();

                    // Get the browser window size
                    let browser_window = web_sys::window().unwrap();
                    let window_width = browser_window.inner_width().unwrap().as_f64().unwrap();
                    let window_height = browser_window.inner_height().unwrap().as_f64().unwrap();

                    // Set the canvas to the browser size
                    winit_window.set_inner_size(winit::dpi::Size::Logical(LogicalSize {
                        width: window_width,
                        height: window_height,
                    }));

                    // Get the webgl context
                    let webgl2_context = winit_window
                        .canvas()
                        .get_context("webgl2")
                        .unwrap()
                        .unwrap()
                        .dyn_into::<web_sys::WebGl2RenderingContext>()
                        .unwrap();

                    glow::Context::from_webgl2_context(webgl2_context)
                };

                #[cfg(not(wasm))]
                const SHADER_VERSION: &str = "330 core";
                #[cfg(wasm)]
                const SHADER_VERSION: &str = "300 es";

                const VERTEX_SHADER_SRC: &str = include_str!("shaders/viewport.vert");
                const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/viewport.frag");

                //
                // Create shader program
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

                // // Get the index for the time uniform from our shader program
                // let time_uniform = gl.get_uniform_location(shader_program, "time").unwrap();
                // Use the shader program
                gl.use_program(Some(shader_program));

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

                // Make a square
                #[rustfmt::skip]
                const QUAD_VERTICES: &[f32] = &[
                    // Positions (3)    // UV (2)
                    -0.8, -1.0,  0.0,   0., 0.,          // bottom left
                     0.8, -1.0,  0.0,   0., 1.,          // bottom right
                     0.8,  1.0,  0.0,   1., 1.,          // top right
                    -0.8,  1.0,  0.0,   1., 0.,          // top left
                ];
                const QUAD_INDICES: &[u32] = &[
                    0, 1, 2, // First triangle
                    0, 2, 3, // Second triangle
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
                    QUAD_VERTICES.as_mem_bytes(),
                    glow::STATIC_DRAW,
                );

                // Create the element buffer object ( EBO ) for indexing into the vertices in the VBO
                let ebo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
                gl.buffer_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    QUAD_INDICES.as_mem_bytes(),
                    glow::STATIC_DRAW,
                );

                // Describe our vertex position attribute data format
                gl.vertex_attrib_pointer_f32(
                    // The index of the `in` var in the shader
                    0,
                    // The number of components in our attribute ( 3 values in a Vec3 )
                    3,
                    // The data type
                    glow::FLOAT,
                    // makes integer types normalized to 0 and 1 when converting to float
                    false,
                    // The space between each vertex attribute ( which is the size of the
                    // vertex position, 3 floats, plus the size of the uvs, 2 floats ).
                    5 * std::mem::size_of::<f32>() as i32,
                    // The offset since the beginning of the buffer to look for the attribute
                    0,
                );

                // Describe our vertex UV attribute data fomrat
                gl.vertex_attrib_pointer_f32(
                    1,
                    2,
                    glow::FLOAT,
                    false,
                    5 * std::mem::size_of::<f32>() as i32,
                    // Offset the length of the 3 floats in the position
                    3 * std::mem::size_of::<f32>() as i32,
                );

                // Enable the position vertex attribute
                gl.enable_vertex_attrib_array(0);

                // Enable the uv vertex attribute
                gl.enable_vertex_attrib_array(1);

                self.gl_objects.insert(
                    window.id(),
                    GlObjects {
                        shader_program,
                        vao,
                    },
                );
                self.gl_handles.insert(window.id(), gl);

                #[cfg(not(wasm))]
                {
                    let context = context.treat_as_not_current();
                    self.window_contexts.insert(window.id(), context);
                }
            }
        }
    }

    fn update(&mut self, world: &mut World) {
        self.handle_window_create_events(world);
        self.handle_window_resize_events(world);

        let render_image = world.get_resource::<RetroRenderImage>().unwrap();

        for window in world.get_resource::<Windows>().unwrap().iter() {
            // Set the WASM canvas to the size of the window
            #[cfg(wasm)]
            {
                let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
                let winit_window = winit_windows.get_window(window.id()).unwrap();

                // Get the browser window size
                let browser_window = web_sys::window().unwrap();
                let window_width = browser_window.inner_width().unwrap().as_f64().unwrap();
                let window_height = browser_window.inner_height().unwrap().as_f64().unwrap();

                // Set the canvas to the browser size
                winit_window.set_inner_size(winit::dpi::Size::Logical(LogicalSize {
                    width: window_width,
                    height: window_height,
                }));
            }

            unsafe {
                let gl_objects = self.gl_objects.get(&window.id()).unwrap();
                let gl = self.gl_handles.get(&window.id()).unwrap();

                #[cfg(not(wasm))]
                let context = {
                    let context = self.window_contexts.remove(&window.id()).unwrap();
                    let context = context.make_current().unwrap();

                    context
                };

                // Set the clear color
                gl.clear(glow::COLOR_BUFFER_BIT);

                // Load the texture image
                let render_texture = load_and_bind_texture(&gl, glow::TEXTURE0, render_image);

                // Bind the texture
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(glow::TEXTURE_2D, Some(render_texture));
                // Set the texture uniform
                gl.uniform_1_i32(
                    gl.get_uniform_location(gl_objects.shader_program, "renderTexture")
                        .as_ref(),
                    0,
                );

                // Draw the quad
                gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);

                // Print any debug messages
                #[cfg(not(wasm))]
                for log in gl.get_debug_message_log(10) {
                    dbg!(log);
                }

                #[cfg(not(wasm))]
                {
                    context.swap_buffers().unwrap();
                    let context = context.treat_as_not_current();
                    self.window_contexts.insert(window.id(), context);
                }
            }
        }
    }
}

impl Drop for RetroRenderer {
    /// Clean up our rendering resources
    fn drop(&mut self) {
        unsafe {
            for (window_id, gl) in &self.gl_handles {
                let gl_objects = self.gl_objects.get(window_id).unwrap();

                #[cfg(not(wasm))]
                let context = {
                    let context = self.window_contexts.remove(&window_id).unwrap();
                    let context = context.make_current().unwrap();

                    context
                };

                gl.delete_vertex_array(gl_objects.vao);
                gl.delete_program(gl_objects.shader_program);

                #[cfg(not(wasm))]
                {
                    context.treat_as_not_current();
                }
            }
        }
    }
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

fn load_and_bind_texture(
    gl: &glow::Context,
    texture_id: u32,
    render_image: &RetroRenderImage,
) -> <glow::Context as HasContext>::Texture {
    unsafe {
        // Select the texture unit
        gl.active_texture(texture_id);

        // Get the texture image
        let image = &render_image.0;
        let (width, height, pixels) = (image.width(), image.height(), image.clone().into_raw());

        // Create the GL texture for our rectangle
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        // Set our texure parameters
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );

        // Set our image data
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            width as i32,
            height as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(&pixels),
        );

        texture
    }
}
