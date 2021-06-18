use std::usize;

use bevy::{
    app::{Events, ManualEventReader},
    prelude::*,
    winit::WinitWindows,
};
use luminance::{
    blending::{Blending, Equation, Factor},
    context::GraphicsContext,
    pipeline::{PipelineState, TextureBinding},
    pixel::NormRGBA8UI,
    render_state::RenderState,
    shader::Uniform,
    texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler, Wrap},
    Semantics, UniformInterface, Vertex,
};

use crate::{graphics::*, prelude::*};

/// The default custom camera shader string
const DEFAULT_CUSTOM_SHADER: &str = r#"
    uniform sampler2D screen_texture;

    varying vec2 uv;

    void main() {
        gl_FragColor = texture2D(screen_texture, uv);
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
const SCREEN_VERTS: [QuadVert; 4] = [
    QuadVert::new(VertexPosition::new([-1.0, 1.0])),
    QuadVert::new(VertexPosition::new([1.0, 1.0])),
    QuadVert::new(VertexPosition::new([1.0, -1.0])),
    QuadVert::new(VertexPosition::new([-1.0, -1.0])),
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
struct QuadVert {
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

#[derive(UniformInterface)]
struct SceneUniformInterface {
    #[uniform(unbound)]
    #[cfg(not(wasm))]
    texture: Uniform<TextureBinding<Dim2, luminance::pixel::Floating>>,
    #[uniform(unbound)]
    #[cfg(wasm)]
    texture: Uniform<TextureBinding<Dim2, luminance::pixel::Unsigned>>,
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
    low_res_scene_framebuffer: SceneFramebuffer,
    high_res_scene_framebuffer: SceneFramebuffer,
    quad: Tess<QuadVert>,
    low_to_high_program: Program<(), (), SceneUniformInterface>,
    screen_program: Program<(), (), ScreenUniformInterface>,

    /// The user's custom camera shader
    custom_shader: Option<String>,

    /// The list of render hooks
    render_hooks: Vec<Box<dyn RenderHook>>,

    // The texture cache
    texture_cache: TextureCache,
    image_asset_event_reader: ManualEventReader<AssetEvent<Image>>,
    pending_textures: Vec<Handle<Image>>,
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
            intern("texture");
            intern("time");
        }

        let scene_program = surface
            .new_shader_program::<(), (), SceneUniformInterface>()
            .from_strings(
                r#"
                    precision highp float;
                    precision highp int;

                    attribute vec2 v_pos;
                    varying vec2 uv;

                    void main() {
                        gl_Position = vec4(v_pos, 0., 1.);
                        uv = v_pos * .5 + .5;
                    }
                "#,
                None,
                None,
                r#"
                    uniform sampler2D texture;
                    varying vec2 uv;

                    void main() {
                        gl_FragColor = texture2D(texture, uv);
                    }
                "#,
            )
            .unwrap()
            .program;
        let screen_program = build_screen_program(&mut surface, None);

        // Create the low-res scene framebuffer that we will render the scene to
        let low_res_scene_framebuffer = surface
            // Because we are just initializing, we don't know what the framebuffer size should be
            // so we set it to 1x1
            .new_framebuffer([1, 1], 0, PIXELATED_SAMPLER)
            .expect("Create framebuffer");

        let high_res_scene_framebuffer = surface
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
            quad: screen_tess,
            low_to_high_program: scene_program,
            screen_program,
            low_res_scene_framebuffer,
            high_res_scene_framebuffer,
            custom_shader: None,
            render_hooks: Vec::new(),

            texture_cache: Default::default(),
            image_asset_event_reader: Default::default(),
            pending_textures: Default::default(),
        }
    }

    #[tracing::instrument(skip(self, world))]
    pub fn update(&mut self, world: &mut World) {
        // Check for any new render hooks and add them to our render hook list
        self.add_render_hooks(world);

        let Self {
            screen_program,
            quad: screen_tess,
            low_res_scene_framebuffer,
            high_res_scene_framebuffer,
            surface,
            window_id,
            render_hooks,
            pending_textures,
            texture_cache,
            image_asset_event_reader,
            low_to_high_program,
            ..
        } = self;
        let mut low_res_scene_framebuffer = low_res_scene_framebuffer;
        let mut high_res_scene_framebuffer = high_res_scene_framebuffer;

        // Upload any textures that have been created to the GPU
        Self::handle_image_asset_event(
            pending_textures,
            texture_cache,
            image_asset_event_reader,
            surface,
            world,
        );

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
        let low_res_target_size = camera.get_target_size(bevy_window);
        let low_res_target_size = [low_res_target_size.x, low_res_target_size.y];

        // Recreate the low-res scene framebuffer if its size does not match our target size
        let fbsize = low_res_scene_framebuffer.size();
        if low_res_target_size != fbsize {
            *low_res_scene_framebuffer = surface
                .new_framebuffer(low_res_target_size, 0, PIXELATED_SAMPLER)
                .expect("Create framebuffer");
        }

        // Recreate the high-res scene framebuffer if its size does not match our target size
        let fbsize = high_res_scene_framebuffer.size();
        let multiple = (window_size.width as f32 / low_res_target_size[0] as f32).ceil() as u32;
        let high_res_target_size = [
            low_res_target_size[0] * multiple,
            low_res_target_size[1] * multiple,
        ];
        if high_res_target_size != fbsize {
            *high_res_scene_framebuffer = surface
                .new_framebuffer(high_res_target_size, 0, PIXELATED_SAMPLER)
                .expect("Create framebuffer");
        }

        let mut renderables = Vec::new();
        // Loop through our render hooks and run their prepare functions
        for (i, hook) in render_hooks.iter_mut().enumerate() {
            for handle in hook.prepare(world, texture_cache, surface) {
                // Add all the renderables from this render hook to our renderables list
                renderables.push(Renderable {
                    hook_idx: i,
                    handle,
                });
            }
        }

        let bevy_time = world.get_resource::<Time>().unwrap();
        let seconds_since_startup = bevy_time.seconds_since_startup() as f32;

        // Sort renderables before rendering
        renderables.sort();

        let mut current_batch = Vec::new();
        let mut current_batch_render_hook_idx = 0;
        let mut current_batch_low_res = false;

        // Clear the back buffer
        surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color(color_to_array(camera.letterbox_color)),
                |_, _| Ok(()),
            )
            .assume();

        // Clear our high-res scene buffer
        surface
            .new_pipeline_gate()
            .pipeline(
                &high_res_scene_framebuffer,
                &PipelineState::default().set_clear_color(color_to_array(camera.background_color)),
                |_, _| Ok(()),
            )
            .assume();

        // Helper to render current batch
        macro_rules! render_current_batch {
            () => {
                let fb = if current_batch_low_res {
                    &mut low_res_scene_framebuffer
                } else {
                    &mut high_res_scene_framebuffer
                };

                // If this is a low-resolution pass, clear the low res framebuffer before rendering
                // to it
                if current_batch_low_res {
                    surface
                        .new_pipeline_gate()
                        .pipeline(
                            &fb,
                            &PipelineState::default().set_clear_color([0., 0., 0., 0.]),
                            |_, _| Ok(()),
                        )
                        .assume();
                }

                // Render the current batch to the target framebuffer
                let batch_renderables: Vec<_> = current_batch.iter().map(|x| x.handle).collect();
                render_hooks
                    .get_mut(current_batch_render_hook_idx)
                    .unwrap()
                    .render(world, surface, texture_cache, fb, &batch_renderables);

                // If this is a low-resolution pass, render it onto the high-resolution framebuffer
                // that represents our scene
                if current_batch_low_res {
                    surface
                        .new_pipeline_gate()
                        .pipeline(
                            &high_res_scene_framebuffer,
                            &PipelineState::default().enable_clear_color(false),
                            |pipeline, mut shd_gate| {
                                let bound_texture = pipeline
                                    .bind_texture(low_res_scene_framebuffer.color_slot())?;

                                shd_gate.shade(
                                    low_to_high_program,
                                    |mut interface, uniforms, mut rdr_gate| {
                                        interface.set(&uniforms.texture, bound_texture.binding());

                                        let render_state = &RenderState::default()
                                            .set_blending(Blending {
                                                equation: Equation::Additive,
                                                src: Factor::SrcAlpha,
                                                dst: Factor::SrcAlphaComplement,
                                            })
                                            .set_depth_test(None);

                                        rdr_gate.render(&render_state, |mut tess_gate| {
                                            tess_gate.render(&*screen_tess)
                                        })
                                    },
                                )
                            },
                        )
                        .assume();
                }
            };
        }

        // Loop through our renderers and render them
        for renderable in renderables {
            // If our current batch of renderables is empty
            if current_batch.is_empty() {
                // Add this renderable to the current batch
                current_batch_render_hook_idx = renderable.hook_idx;
                current_batch_low_res = renderable.handle.low_resolution;
                current_batch.push(renderable);

            // If we are in the middle of creating a batch
            } else {
                // If this renderable is for the same hook as the current batch and has the same
                // resolution setting
                if renderable.hook_idx == current_batch_render_hook_idx
                    && renderable.handle.low_resolution == current_batch_low_res
                {
                    // Add it to the currrent batch
                    current_batch.push(renderable);

                // If the current renderable is not for the same hook as the
                // current batch.
                } else {
                    render_current_batch!();

                    // And start a new batch
                    current_batch.clear();
                    current_batch.push(renderable);
                    current_batch_render_hook_idx = renderable.hook_idx;
                    current_batch_low_res = renderable.handle.low_resolution;
                }
            }
        }

        // Render the final batch
        render_current_batch!();

        // Render our high-res scene framebuffer onto the backbuffer
        surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().enable_clear_color(false),
                |pipeline, mut shd_gate| {
                    // we must bind the offscreen framebuffer color content so that we can pass
                    // it to a shader
                    let bound_texture =
                        pipeline.bind_texture(high_res_scene_framebuffer.color_slot())?;

                    shd_gate.shade(screen_program, |mut interface, uniforms, mut rdr_gate| {
                        interface.set(
                            &uniforms.camera_size,
                            [low_res_target_size[0] as i32, low_res_target_size[1] as i32],
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
                        interface.set(&uniforms.time, seconds_since_startup);

                        let render_state = &RenderState::default()
                            .set_blending(Blending {
                                equation: Equation::Additive,
                                src: Factor::SrcAlpha,
                                dst: Factor::SrcAlphaComplement,
                            })
                            .set_depth_test(None);

                        rdr_gate.render(&render_state, |mut tess_gate| {
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

    #[tracing::instrument(skip(
        pending_textures,
        texture_cache,
        image_asset_event_reader,
        surface,
        world
    ))]
    pub(crate) fn handle_image_asset_event(
        pending_textures: &mut Vec<Handle<Image>>,
        texture_cache: &mut TextureCache,
        image_asset_event_reader: &mut ManualEventReader<AssetEvent<Image>>,
        surface: &mut Surface,
        world: &mut World,
    ) {
        let image_asset_events = world.get_resource::<Events<AssetEvent<Image>>>().unwrap();
        let image_assets = world.get_resource::<Assets<Image>>().unwrap();

        let mut upload_texture = |image: &Image| {
            // Get the sprite image info
            let (sprite_width, sprite_height) = image.dimensions();
            let sprite_size = [sprite_width, sprite_height];
            let pixels = image.as_raw();

            // Upload the sprite to the GPU
            let mut texture = surface
                .new_texture::<Dim2, NormRGBA8UI>(sprite_size, 0, PIXELATED_SAMPLER)
                .unwrap();
            texture.upload_raw(GenMipmaps::No, pixels).unwrap();

            texture
        };

        // Attempt to load pending textures
        let mut new_pending_textures = Vec::new();
        for handle in &*pending_textures {
            if let Some(image) = image_assets.get(handle) {
                upload_texture(image);
            } else {
                new_pending_textures.push(handle.clone());
            }
        }
        *pending_textures = new_pending_textures;

        // for every image asset event
        for event in image_asset_event_reader.iter(&image_asset_events) {
            match event {
                AssetEvent::Created { handle } => {
                    if let Some(image) = image_assets.get(handle) {
                        texture_cache.insert(handle.clone(), upload_texture(image));
                    } else {
                        pending_textures.push(handle.clone());
                    }
                }
                AssetEvent::Modified { handle } => {
                    if let Some(image) = image_assets.get(handle) {
                        texture_cache.insert(handle.clone(), upload_texture(image));
                    } else {
                        pending_textures.push(handle.clone());
                    }
                }
                AssetEvent::Removed { handle } => {
                    texture_cache.remove(handle);
                }
            }
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
