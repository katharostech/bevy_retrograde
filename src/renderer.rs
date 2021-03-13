use bevy::{
    app::{Events, ManualEventReader},
    math::clamp,
    prelude::*,
    utils::HashMap,
    window::{WindowCreated, WindowResized},
    winit::WinitWindows,
};

#[cfg(not(wasm))]
use glutin::{ContextBuilder, NotCurrent, RawContext};

#[cfg(unix)]
use glutin::platform::unix::{RawContextExt, WindowExtUnix};

#[cfg(windows)]
use glutin::platform::windows::{RawContextExt, WindowExtWindows};

use glow::HasContext;
use image::RgbaImage;

use crate::{Camera, CameraSize, Color, Position, SpriteImage, Visible};

#[derive(Clone, Debug)]
pub struct RetroRenderOptions {
    pub pixel_aspect_raio: f32,
}

impl Default for RetroRenderOptions {
    fn default() -> Self {
        Self {
            pixel_aspect_raio: 1.0,
        }
    }
}

/// The information that is sent to the renderer every frame so that it can render the frame
#[derive(Default, Debug, Clone)]
pub(crate) struct RenderFrame {
    image: RgbaImage,
    background_color: Color,
}

/// This system is the system that takes all of the sprites in the scene and produces the final RGBA
/// image that is rendered.
pub(crate) fn pre_render_system(
    sprites: Query<(&Handle<SpriteImage>, &Visible, &Position)>,
    cameras: Query<(&Camera, &Position)>,
    sprite_image_assets: Res<Assets<SpriteImage>>,
    windows: Res<Windows>,
    winit_windows: Res<WinitWindows>,
    mut render_image_out: ResMut<RenderFrame>,
) {
    use image::*;

    // Get the camera
    let mut camera_iter = cameras.iter();
    let (camera, camera_pos) = if let Some(camera_components) = camera_iter.next() {
        camera_components
    } else {
        return;
    };
    if camera_iter.next().is_some() {
        panic!("Only one Retro camera is supported");
    }

    // Get the current window
    let window = if let Some(window) = windows.get_primary() {
        window
    } else {
        return;
    };
    let winit_window = winit_windows.get_window(window.id()).unwrap();
    // Get the window's aspect ratio
    let aspect_ratio =
        winit_window.inner_size().width as f32 / winit_window.inner_size().height as f32;

    // Get the camera height and width
    let camera_height = if let CameraSize::FixedHeight(height) = camera.size {
        height
    } else {
        panic!("Only fixed height camera size is support, open an issue and I'll fix it!");
    };
    let camera_width = (camera_height as f32 * aspect_ratio).floor() as u32;

    // Get the offset of the center of the camera in camera space
    let camera_center_offset_x = (camera_width as f32 / 2.0).floor() as i32;
    let camera_center_offset_y = (camera_height as f32 / 2.0).floor() as i32;

    // Create the render image
    let mut render_image = RgbaImage::new(camera_width, camera_height);

    // Add sprites to the render image
    for (sprite_handle, visible, sprite_pos) in sprites.iter() {
        // Skip invisible sprites
        if !**visible {
            return;
        }

        if let Some(sprite) = sprite_image_assets.get(sprite_handle) {
            let (width, height) = sprite.image.dimensions();

            // Get the offset to the center of the sprite
            let sprite_center_offset_x = (width as f32 / 2.0).floor() as i32;
            let sprite_center_offset_y = (height as f32 / 2.0).floor() as i32;

            // Get the sprite position in camera space
            let sprite_camera_space_x = sprite_pos.x - camera_pos.x;
            let sprite_camera_space_y = sprite_pos.y - camera_pos.y;

            // Get the sprite position in image space
            let sprite_image_space_x = camera_center_offset_x + sprite_camera_space_x;
            let sprite_image_space_y = camera_center_offset_y + sprite_camera_space_y;

            // Get the min and max x and y screen position of the sprite in image space
            let sprite_image_space_min_x = clamp(
                sprite_image_space_x - sprite_center_offset_x,
                0,
                camera_width as i32,
            ) as u32;
            let sprite_image_space_max_x = clamp(
                sprite_image_space_x + sprite_center_offset_x,
                0,
                camera_width as i32,
            ) as u32;
            let sprite_image_space_min_y = clamp(
                sprite_image_space_y - sprite_center_offset_y,
                0,
                camera_height as i32,
            ) as u32;
            let sprite_image_space_max_y = clamp(
                sprite_image_space_y + sprite_center_offset_y,
                0,
                camera_height as i32,
            ) as u32;

            // Calculate height and width of the visible portion of the sprite
            let sprite_visible_width = sprite_image_space_max_x - sprite_image_space_min_x;
            let sprite_visible_height = sprite_image_space_max_y - sprite_image_space_min_y;

            // Cull the sprite if it's clamped width or height is 0
            if sprite_visible_width == 0 || sprite_visible_height == 0 {
                continue;
            }

            // Get a view into the visible portion ov the sprite image
            let sprite_image_view = &sprite.image.view(
                // If the sprite is cut off at the left, then the x should be equal to how much it
                // is cut off.
                if sprite_image_space_min_x == 0 {
                    width - sprite_image_space_max_x

                // Otherwise it is zero
                } else {
                    0
                },
                // And the same for the y
                if sprite_image_space_min_y == 0 {
                    height - sprite_image_space_max_y

                // Otherwise it is zero
                } else {
                    0
                },
                sprite_visible_width,
                sprite_visible_height,
            );

            // Get a sub-image of the camera for where our sprite is to be rendered
            let mut render_sub_image = render_image.sub_image(
                sprite_image_space_min_x,
                sprite_image_space_min_y,
                sprite_visible_width,
                sprite_visible_height,
            );

            // Copy the sprite image onto our render image
            render_sub_image.copy_from(sprite_image_view, 0, 0).unwrap();
        }
    }

    *render_image_out = RenderFrame {
        image: render_image,
        background_color: camera.background_color,
    };
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
    pub window_aspect_ratios: HashMap<bevy::window::WindowId, f32>,
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
    render_texture: <glow::Context as HasContext>::Texture,
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

                self.window_aspect_ratios.insert(
                    window_id,
                    window.physical_width() as f32 / window.physical_height() as f32,
                );

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
                let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
                let winit_window = winit_windows.get_window(window.id()).unwrap();

                // Set the window aspect ratio
                self.window_aspect_ratios.insert(
                    window.id(),
                    winit_window.inner_size().width as f32
                        / winit_window.inner_size().height as f32,
                );

                #[cfg(not(wasm))]
                let (gl, context) = {
                    // Create the raw context from the winit window
                    #[cfg(unix)]
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

                    #[cfg(windows)]
                    let context = ContextBuilder::new()
                        .build_raw_context(winit_window.hwnd())
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
                    winit_window.set_inner_size(winit::dpi::Size::Logical(
                        winit::dpi::LogicalSize {
                            width: window_width,
                            height: window_height,
                        },
                    ));

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

                // Make a square
                #[rustfmt::skip]
                const QUAD_VERTICES: &[f32] = &[
                    // Positions (3)    // UV (2)
                    -1.0, -1.0,  0.0,   0., 1.,          // bottom left
                     1.0, -1.0,  0.0,   1., 1.,          // bottom right
                     1.0,  1.0,  0.0,   1., 0.,          // top right
                    -1.0,  1.0,  0.0,   0., 0.,          // top left
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

                // Create a texture that we will render to
                gl.active_texture(glow::TEXTURE0);
                let render_texture = gl.create_texture().unwrap();
                gl.bind_texture(glow::TEXTURE_2D, Some(render_texture));

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

                self.gl_objects.insert(
                    window.id(),
                    GlObjects {
                        shader_program,
                        vao,
                        render_texture,
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

        let render_options = world.get_resource::<RetroRenderOptions>().unwrap();
        let render_image = world.get_resource::<RenderFrame>().unwrap();

        for window in world.get_resource::<Windows>().unwrap().iter() {
            let window_aspect_ratio = *self.window_aspect_ratios.get(&window.id()).unwrap();

            // TODO: Find out a way to detect browser resize events and recalculate the canvas size when the browser
            // resizes:

            // // Set the WASM canvas to the size of the window
            // #[cfg(wasm)]
            // let window_aspect_ratio = {
            //     let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
            //     let winit_window = winit_windows.get_window(window.id()).unwrap();

            //     // Get the browser window size
            //     let browser_window = web_sys::window().unwrap();
            //     let window_width = browser_window.inner_width().unwrap().as_f64().unwrap();
            //     let window_height = browser_window.inner_height().unwrap().as_f64().unwrap();

            //     // Set the canvas to the browser size
            //     winit_window.set_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize {
            //         width: window_width as u32,
            //         height: window_height as u32,
            //     }));

            //     winit_window.inner_size().width as f32 / winit_window.inner_size().height as f32
            // };

            unsafe {
                let gl_objects = self.gl_objects.get(&window.id()).unwrap();
                let gl = self.gl_handles.get(&window.id()).unwrap();

                #[cfg(not(wasm))]
                let context = {
                    let context = self.window_contexts.remove(&window.id()).unwrap();
                    let context = context.make_current().unwrap();

                    context
                };

                // Set the clear color and clear the screen
                let background_color = render_image.background_color;
                gl.clear_color(
                    background_color.r,
                    background_color.g,
                    background_color.b,
                    background_color.a,
                );
                gl.clear(glow::COLOR_BUFFER_BIT);

                // Get the render image aspect ratio
                let image_aspect_ratio =
                    render_image.image.width() as f32 / render_image.image.height() as f32;

                // Load the texture image
                load_render_texture(&gl, gl_objects.render_texture, &render_image);

                // Set the texture uniform
                gl.uniform_1_i32(
                    gl.get_uniform_location(gl_objects.shader_program, "renderTexture")
                        .as_ref(),
                    0,
                );

                // Set the window aspect ratio uniform
                gl.uniform_1_f32(
                    gl.get_uniform_location(gl_objects.shader_program, "aspectCorrectionFactor")
                        .as_ref(),
                    window_aspect_ratio / image_aspect_ratio / render_options.pixel_aspect_raio,
                );

                // Draw the quad
                gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);

                #[cfg(wasm)]
                gl.flush();

                // Print any debug messages
                #[cfg(all(not(wasm), debug_assertions))]
                for log in gl.get_debug_message_log(10) {
                    bevy::log::debug!("GL debug: {:?}", log);
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

fn load_render_texture(
    gl: &glow::Context,
    texture: <glow::Context as HasContext>::Texture,
    render_image: &RenderFrame,
) {
    unsafe {
        // Get the texture image
        let image = &render_image.image;
        let (width, height, pixels) = (image.width(), image.height(), image.clone().into_raw());

        // Bind the texture
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

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
    }
}
