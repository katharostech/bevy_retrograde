use std::usize;

use bevy::{prelude::*, winit::WinitWindows};
use luminance::{
    context::GraphicsContext,
    pipeline::{PipelineState, TextureBinding},
    render_state::RenderState,
    shader::Uniform,
    texture::{Dim2, MagFilter, MinFilter, Sampler, Wrap},
    Semantics, UniformInterface, Vertex,
};

use crate::{graphics::*, prelude::*};

/// The default custom camera shader string
const DEFAULT_CUSTOM_SHADER: &str = r#"
    uniform sampler2D screen_texture;
    uniform float time;
    uniform ivec2 window_size;

    varying vec2 uv;

    void main() {
        gl_FragColor = vec4(texture2D(screen_texture, uv).rgb, 1.);
    }
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "v_pos", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "v_uv", repr = "[f32; 2]", wrapper = "VertexUv")]
    Uv,
}

// Quad vertices in a triangle fan
const SCREEN_VERTS: [ScreenVert; 4] = [
    ScreenVert::new(VertexPosition::new([-1.0, 1.0])),
    ScreenVert::new(VertexPosition::new([1.0, 1.0])),
    ScreenVert::new(VertexPosition::new([1.0, -1.0])),
    ScreenVert::new(VertexPosition::new([-1.0, -1.0])),
];

/// The scene framebuffer sampler
pub(crate) const PIXELATED_SAMPLER: Sampler = Sampler {
    wrap_r: Wrap::ClampToEdge,
    wrap_s: Wrap::ClampToEdge,
    wrap_t: Wrap::ClampToEdge,
    min_filter: MinFilter::Nearest,
    mag_filter: MagFilter::Nearest,
    depth_comparison: None,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "VertexSemantics")]
struct ScreenVert {
    pos: VertexPosition,
}

#[derive(UniformInterface)]
struct ScreenUniformInterface {
    camera_size: Uniform<[i32; 2]>,
    /// Indicates whether or not the width or height of the camera is supposed to be fixed:
    ///
    /// - `camera_size_fixed == 0` means both the width and the height are fixed
    /// - `camera_size_fixed == 1` means the width is fixed
    /// - `camera_size_fixed == 2` means the height is fixed
    camera_size_fixed: Uniform<i32>,
    pixel_aspect_ratio: Uniform<f32>,

    window_size: Uniform<[i32; 2]>,
    #[cfg(not(wasm))]
    screen_texture: Uniform<TextureBinding<Dim2, luminance::pixel::Floating>>,
    #[cfg(wasm)]
    screen_texture: Uniform<TextureBinding<Dim2, luminance::pixel::Unsigned>>,
    /// The number of seconds since startup
    #[uniform(unbound)]
    time: Uniform<f32>,
}

/// Utility struct used to keep track of and sort renderable objects provided by
/// [`RenderHook`] implementations.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Renderable {
    // NOTE: The order of these fields are important! We want to sort based on the handle first and the
    // hook idx second.
    handle: RenderHookRenderableHandle,
    hook_idx: usize,
}

pub(crate) struct Renderer {
    pub(crate) surface: Surface,
    window_id: bevy::window::WindowId,
    scene_framebuffer: SceneFramebuffer,
    screen_tess: Tess<ScreenVert>,
    screen_program: Program<(), (), ScreenUniformInterface>,

    /// The user's custom camera shader
    custom_shader: Option<String>,

    /// The list of render hooks
    render_hooks: Vec<Box<dyn RenderHook>>,
}

impl Renderer {
    #[tracing::instrument(skip(surface))]
    pub fn init(window_id: bevy::window::WindowId, mut surface: Surface) -> Self {
        // Intern shader uniform names
        #[cfg(wasm)]
        {
            use wasm_bindgen::intern;
            intern("camera_size");
            intern("camera_size_fixed");
            intern("pixel_aspect_ratio");
            intern("window_size");
            intern("screen_texture");
            intern("time");
        }

        let screen_program = build_screen_program(&mut surface, None);

        // Create the scene framebuffer that we will render the scene to
        let scene_framebuffer = surface
            // Because we are just initializing, we don't know what the framebuffer size should be
            // so we set it to zero
            .new_framebuffer([1, 1], 0, PIXELATED_SAMPLER)
            .expect("Create framebuffer");

        // Create the tesselator for the screen quad
        let screen_tess = surface
            .new_tess()
            .set_vertices(&SCREEN_VERTS[..])
            .set_mode(luminance::tess::Mode::TriangleFan)
            .build()
            .unwrap();

        Self {
            window_id,
            surface,
            screen_tess,
            screen_program,
            scene_framebuffer,
            custom_shader: None,
            render_hooks: Vec::new(),
        }
    }

    #[tracing::instrument(skip(self, world))]
    pub fn update(&mut self, world: &mut World) {
        // Check for any new render hooks and add them to our render hook list
        self.add_render_hooks(world);

        let Self {
            screen_program,
            screen_tess,
            scene_framebuffer,
            surface,
            window_id,
            render_hooks,
            ..
        } = self;

        // Get the back buffer
        let back_buffer = surface.back_buffer().unwrap();

        // Get the camera
        let mut cameras = world.query::<&Camera>();
        let mut camera_iter = cameras.iter(world);
        let camera = if let Some(camera_components) = camera_iter.next() {
            camera_components.clone()
        } else {
            return;
        };
        if camera_iter.next().is_some() {
            panic!("Only one Retro camera is supported");
        }

        // Get the window this renderer is supposed to render to
        let bevy_windows = world.get_resource::<Windows>().unwrap();
        let bevy_window = bevy_windows.get(*window_id).unwrap();
        let winit_windows = world.get_resource::<WinitWindows>().unwrap();
        let winit_window = winit_windows.get_window(*window_id).unwrap();
        let window_size = winit_window.inner_size();

        // If the camera has a different custom shader, rebuild our screen shader program
        if camera.custom_shader != self.custom_shader {
            self.custom_shader = camera.custom_shader.clone();

            *screen_program = build_screen_program(surface, camera.custom_shader.as_deref());
        }

        // Calculate the target size of our scene framebuffer
        let target_size = camera.get_target_size(bevy_window);
        let target_size = [target_size.x, target_size.y];

        // Recreate the scene framebuffer if its size does not match our target size
        let fbsize = scene_framebuffer.size();
        if target_size != fbsize {
            *scene_framebuffer = surface
                .new_framebuffer(target_size, 0, PIXELATED_SAMPLER)
                .expect("Create framebuffer");
        }

        // Clear the screen
        surface
            .new_pipeline_gate()
            .pipeline(
                &scene_framebuffer,
                &PipelineState::default().set_clear_color(color_to_array(camera.background_color)),
                |_, _| Ok(()),
            )
            .assume();

        let mut renderables = Vec::new();
        // Loop through our render hooks and run their render functions
        for (i, hook) in render_hooks.iter_mut().enumerate() {
            for handle in hook.prepare_low_res(world, surface) {
                // Add all the renderables from this render hook to our renderables list
                renderables.push(Renderable {
                    hook_idx: i,
                    handle,
                });
            }
        }
        // Sort renderables before rendering
        renderables.sort();

        // Loop through our renderers and render them
        let mut current_batch = Vec::new();
        let mut current_batch_render_hook_idx = 0;
        for renderable in renderables {
            // If our current batch of renderables is empty
            if current_batch.len() == 0 {
                // Add this renderable to the current batch
                current_batch_render_hook_idx = renderable.hook_idx;
                current_batch.push(renderable);

            // If we are in the middle of creating a batch
            } else {
                // If this renderable is for the same hook as the current batch
                if renderable.hook_idx == current_batch_render_hook_idx {
                    // Add it to the currrent batch
                    current_batch.push(renderable);

                // If the current renderable is not for the same hook as the
                // current batch.
                } else {
                    // Render the current batch
                    let batch_renderables: Vec<_> =
                        current_batch.iter().map(|x| x.handle).collect();
                    render_hooks
                        .get_mut(current_batch_render_hook_idx)
                        .unwrap()
                        .render_low_res(world, surface, &scene_framebuffer, &batch_renderables);

                    // And start a new batch
                    current_batch.clear();
                    current_batch.push(renderable);
                }
            }
        }

        // Render the final batch
        let batch_renderables: Vec<_> = current_batch.iter().map(|x| x.handle).collect();
        render_hooks
            .get_mut(current_batch_render_hook_idx)
            .unwrap()
            .render_low_res(world, surface, &scene_framebuffer, &batch_renderables);

        let bevy_time = world.get_resource::<Time>().unwrap();

        // Render the scene framebuffer to the back buffer on a quad
        surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color(color_to_array(camera.letterbox_color)),
                |pipeline, mut shd_gate| {
                    // we must bind the offscreen framebuffer color content so that we can pass it to a shader
                    let bound_texture = pipeline.bind_texture(scene_framebuffer.color_slot())?;

                    shd_gate.shade(screen_program, |mut interface, uniforms, mut rdr_gate| {
                        interface.set(
                            &uniforms.camera_size,
                            [target_size[0] as i32, target_size[1] as i32],
                        );
                        interface.set(
                            &uniforms.window_size,
                            [window_size.width as i32, window_size.height as i32],
                        );
                        interface.set(&uniforms.screen_texture, bound_texture.binding());
                        interface.set(&uniforms.pixel_aspect_ratio, camera.pixel_aspect_ratio);
                        interface.set(
                            &uniforms.camera_size_fixed,
                            match camera.size {
                                CameraSize::LetterBoxed { .. } => 0,
                                CameraSize::FixedWidth(_) => 1,
                                CameraSize::FixedHeight(_) => 2,
                            },
                        );
                        interface.set(&uniforms.time, bevy_time.seconds_since_startup() as f32);

                        rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                            tess_gate.render(&*screen_tess)
                        })
                    })
                },
            )
            .assume();

        #[cfg(not(wasm))]
        self.surface.swap_buffers().unwrap();
    }

    /// Check for render hook events and add them to the renderer
    fn add_render_hooks(&mut self, world: &mut World) {
        // Get the render hooks resource
        let mut render_hooks = world.get_resource_mut::<RenderHooks>().unwrap();

        // Initialize each new render hook
        for hook_init in render_hooks.new_hooks.drain(0..) {
            self.render_hooks
                .push(hook_init(self.window_id, &mut self.surface));
        }
    }
}

fn color_to_array(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

fn build_screen_program(
    surface: &mut Surface,
    custom_shader: Option<&str>,
) -> Program<(), (), ScreenUniformInterface> {
    let built_program = surface
        .new_shader_program::<(), (), ScreenUniformInterface>()
        .from_strings(
            include_str!("shaders/screen.vert"),
            None,
            None,
            custom_shader.unwrap_or(DEFAULT_CUSTOM_SHADER),
        )
        .unwrap();

    // Log any shader compilation warnings
    for warning in built_program.warnings {
        warn!("Shader compile arning: {}", warning);
    }

    built_program.program
}
