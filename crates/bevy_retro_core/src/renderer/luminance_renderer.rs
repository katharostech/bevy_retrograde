use luminance::{
    blending::{Blending, Equation, Factor},
    context::GraphicsContext,
    pipeline::{PipelineState, TextureBinding},
    pixel::{NormRGBA8UI, NormUnsigned},
    render_state::RenderState,
    shader::Uniform,
    tess::Interleaved,
    texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler, Wrap},
    Semantics, UniformInterface, Vertex,
};

#[cfg(not(wasm))]
use luminance::pixel::{Floating, RGBA32F};
#[cfg(wasm)]
use luminance::pixel::{Unsigned, RGBA8UI};

use luminance_glow::Glow;

pub type Framebuffer<D, CS, DS> = luminance::framebuffer::Framebuffer<Glow, D, CS, DS>;
pub type Program<Sem, Out, Uni> = luminance::shader::Program<Glow, Sem, Out, Uni>;
pub type Tess<V, I = (), W = (), S = Interleaved> = luminance::tess::Tess<Glow, V, I, W, S>;
pub type Texture<D, P> = luminance::texture::Texture<Glow, D, P>;

use parking_lot::Mutex;

use super::*;
use crate::{starc::Starc, *};

/// The scene framebuffer sampler
const PIXELATED_SAMPLER: Sampler = Sampler {
    wrap_r: Wrap::ClampToEdge,
    wrap_s: Wrap::ClampToEdge,
    wrap_t: Wrap::ClampToEdge,
    min_filter: MinFilter::Nearest,
    mag_filter: MagFilter::Nearest,
    depth_comparison: None,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "v_pos", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "v_uv", repr = "[f32; 2]", wrapper = "VertexUv")]
    Uv,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "VertexSemantics")]
struct SpriteVert {
    pos: VertexPosition,
    uv: VertexUv,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "VertexSemantics")]
struct ScreenVert {
    pos: VertexPosition,
}

// Quad vertices in a triangle fan
const SPRITE_VERTS: [SpriteVert; 4] = [
    SpriteVert::new(VertexPosition::new([0.0, 1.0]), VertexUv::new([0.0, 1.0])),
    SpriteVert::new(VertexPosition::new([1.0, 1.0]), VertexUv::new([1.0, 1.0])),
    SpriteVert::new(VertexPosition::new([1.0, 0.0]), VertexUv::new([1.0, 0.0])),
    SpriteVert::new(VertexPosition::new([0.0, 0.0]), VertexUv::new([0.0, 0.0])),
];

// Quad vertices in a triangle fan
const SCREEN_VERTS: [ScreenVert; 4] = [
    ScreenVert::new(VertexPosition::new([-1.0, 1.0])),
    ScreenVert::new(VertexPosition::new([1.0, 1.0])),
    ScreenVert::new(VertexPosition::new([1.0, -1.0])),
    ScreenVert::new(VertexPosition::new([-1.0, -1.0])),
];

#[derive(UniformInterface)]
struct SpriteUniformInterface {
    camera_position: Uniform<[i32; 2]>,
    camera_size: Uniform<[i32; 2]>,
    camera_centered: Uniform<i32>,

    sprite_texture: Uniform<TextureBinding<Dim2, NormUnsigned>>,
    sprite_texture_size: Uniform<[i32; 2]>,
    sprite_flip: Uniform<i32>,
    sprite_centered: Uniform<i32>,
    sprite_tileset_grid_size: Uniform<[i32; 2]>,
    sprite_tileset_index: Uniform<i32>,
    sprite_position: Uniform<[i32; 3]>,
    sprite_offset: Uniform<[i32; 2]>,
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
    screen_texture: Uniform<TextureBinding<Dim2, Floating>>,
    #[cfg(wasm)]
    screen_texture: Uniform<TextureBinding<Dim2, Unsigned>>,
    /// The number of seconds since startup
    #[uniform(unbound)]
    time: Uniform<f32>,
}

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

pub(crate) struct LuminanceRenderer {
    pub(crate) surface: Surface,
    window_id: bevy::window::WindowId,
    sprite_program: Program<(), (), SpriteUniformInterface>,
    sprite_tess: Tess<SpriteVert>,
    #[cfg(not(wasm))]
    scene_framebuffer: Framebuffer<Dim2, RGBA32F, ()>,
    #[cfg(wasm)]
    scene_framebuffer: Framebuffer<Dim2, RGBA8UI, ()>,
    screen_tess: Tess<ScreenVert>,
    screen_program: Program<(), (), ScreenUniformInterface>,

    texture_cache: HashMap<Handle<Image>, Starc<Mutex<Texture<Dim2, NormRGBA8UI>>>>,

    image_asset_event_reader: ManualEventReader<AssetEvent<Image>>,
    pending_textures: Vec<Handle<Image>>,

    /// The user's custom camera shader
    custom_shader: Option<String>,
}

impl LuminanceRenderer {
    #[tracing::instrument(skip(surface))]
    pub fn init(window_id: bevy::window::WindowId, mut surface: Surface) -> Self {
        // Create the tesselator for the sprites
        let sprite_tess = surface
            .new_tess()
            .set_vertices(&SPRITE_VERTS[..])
            .set_mode(luminance::tess::Mode::TriangleFan)
            .build()
            .unwrap();

        // Create the tesselator for the screen quad
        let screen_tess = surface
            .new_tess()
            .set_vertices(&SCREEN_VERTS[..])
            .set_mode(luminance::tess::Mode::TriangleFan)
            .build()
            .unwrap();

        // Create the shader program for the sprite instances
        let built_sprite_program = surface
            .new_shader_program::<(), (), SpriteUniformInterface>()
            .from_strings(
                include_str!("shaders/sprite_quad.vert"),
                None,
                None,
                include_str!("shaders/sprite_quad.frag"),
            )
            .unwrap();

        let screen_program = build_screen_program(&mut surface, None);

        // Create the scene framebuffer that we will render the scene to
        let scene_framebuffer = surface
            // Because we are just initializing, we don't know what the framebuffer size should be
            // so we set it to zero
            .new_framebuffer([1, 1], 0, PIXELATED_SAMPLER)
            .expect("Create framebuffer");

        Self {
            window_id,
            surface,
            sprite_tess,
            sprite_program: built_sprite_program.program,
            screen_tess,
            screen_program,
            scene_framebuffer,
            texture_cache: Default::default(),
            image_asset_event_reader: Default::default(),
            pending_textures: Default::default(),
            custom_shader: None,
        }
    }

    #[tracing::instrument(skip(self, world))]
    pub fn update(&mut self, world: &mut World) {
        // Handle image asset events
        self.handle_image_asset_event(world);

        let Self {
            sprite_program,
            screen_program,
            sprite_tess,
            screen_tess,
            scene_framebuffer,
            surface,
            window_id,
            texture_cache,
            ..
        } = self;

        let span_setup = info_span!("setup");
        let span_setup_guard = span_setup.enter();

        // Get the back buffer
        #[cfg(wasm)]
        let back_buffer = surface.back_buffer().unwrap();
        #[cfg(not(wasm))]
        let back_buffer = surface.back_buffer().unwrap();

        // Build the queries and get the resources that we will need
        let mut cameras = world.query::<(&Camera, &WorldPosition)>();
        let mut sprites = world.query::<(
            &Handle<Image>,
            &Sprite,
            Option<&Handle<SpriteSheet>>,
            &Visible,
            &WorldPosition,
        )>();

        let sprite_sheet_assets = world.get_resource::<Assets<SpriteSheet>>().unwrap();

        // Get the window this renderer is supposed to render to
        let bevy_windows = world.get_resource::<Windows>().unwrap();
        let bevy_window = bevy_windows.get(*window_id).unwrap();
        let winit_windows = world.get_resource::<WinitWindows>().unwrap();
        let winit_window = winit_windows.get_window(*window_id).unwrap();

        // Get the camera
        let mut camera_iter = cameras.iter(world);
        let (camera, camera_pos) = if let Some(camera_components) = camera_iter.next() {
            camera_components
        } else {
            return;
        };
        if camera_iter.next().is_some() {
            panic!("Only one Retro camera is supported");
        }

        // If the camera has a different custom shader, rebuild our screen shader program
        if camera.custom_shader != self.custom_shader {
            self.custom_shader = camera.custom_shader.clone();

            *screen_program =
                build_screen_program(surface, camera.custom_shader.as_ref().map(String::as_str));
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

        // Create the render state
        let render_state = &RenderState::default().set_blending_separate(
            Blending {
                equation: Equation::Additive,
                src: Factor::SrcAlpha,
                dst: Factor::SrcAlphaComplement,
            },
            Blending {
                equation: Equation::Additive,
                src: Factor::One,
                dst: Factor::Zero,
            },
        );

        let sprite_iter = sprites.iter(world);
        let mut sprite_data = Vec::new();

        for (image_handle, sprite_flip, sprite_sheet_handle, visible, world_position) in sprite_iter
        {
            // Skip invisible sprites
            if !**visible {
                continue;
            }

            // Load the sprite sheet if any
            let sprite_sheet = sprite_sheet_handle
                .map(|x| sprite_sheet_assets.get(x))
                .flatten();

            sprite_data.push((image_handle, sprite_flip, sprite_sheet, world_position));
        }

        // Sort by depth
        sprite_data.sort_by(|(_, _, _, pos1), (_, _, _, pos2)| pos1.z.cmp(&pos2.z));

        drop(span_setup_guard);

        let span_render = info_span!("render");
        let span_render_guard = span_render.enter();

        // Do the render
        surface
            .new_pipeline_gate()
            .pipeline(
                // Render to the scene framebuffer
                &scene_framebuffer,
                &PipelineState::default().set_clear_color(color_to_array(camera.background_color)),
                |pipeline, mut shading_gate| {
                    shading_gate.shade(
                        sprite_program,
                        |mut interface, uniforms, mut render_gate| {
                            // Set the camera uniforms
                            interface.set(&uniforms.camera_position, [camera_pos.x, camera_pos.y]);
                            interface.set(&uniforms.camera_size, [target_size[0] as i32, target_size[1] as i32]);
                            interface.set(&uniforms.camera_centered, if camera.centered { 1 } else { 0 });

                            for (image_handle, sprite, sprite_sheet, world_position) in
                                &mut sprite_data
                            {
                                // Get the texture using the image handle
                                let texture =
                                    if let Some(texture) = texture_cache.get_mut(image_handle) {
                                        texture
                                    } else {
                                        // Skip it if the texture has not loaded
                                        continue;
                                    };

                                // Bind our texture
                                let mut texture = texture.lock();
                                let bound_texture = pipeline.bind_texture(&mut *texture).unwrap();

                                // Set the texture uniform
                                interface.set(&uniforms.sprite_texture, bound_texture.binding());

                                // Set the texture size uniform
                                let size = texture.size();
                                let size = [size[0] as i32, size[1] as i32];
                                interface.set(&uniforms.sprite_texture_size, size);

                                // Set the sprite uniforms
                                interface.set(
                                    &uniforms.sprite_flip,
                                    if sprite.flip_x { 0b01 } else { 0 } as i32
                                        | if sprite.flip_y { 0b10 } else { 0 } as i32,
                                );
                                interface.set(&uniforms.sprite_centered, if sprite.centered { 1 } else { 0 });

                                // Set the sprite tileset uniforms
                                let grid_size = 
                                    sprite_sheet
                                        .map(|x| [x.grid_size.x as i32, x.grid_size.y as i32])
                                        .unwrap_or([0; 2]);
                                interface.set(
                                    &uniforms.sprite_tileset_grid_size,
                                    grid_size
                                );
                                interface.set(
                                    &uniforms.sprite_tileset_index,
                                    sprite_sheet.map(|x| x.tile_index as i32).unwrap_or(0),
                                );

                                // Set sprite position and offset
                                debug_assert!(
                                    -1024 < world_position.z && world_position.z <= 1024,
                                    "Sprite world Z position must be between -1024 and 1024. Please \
                                    open an issue if this is a problem for you: \
                                    https://github.com/katharostech/bevy_retro/issues"
                                );
                                interface.set(
                                    &uniforms.sprite_position,
                                    [world_position.x, world_position.y, world_position.z],
                                );
                                interface.set(
                                    &uniforms.sprite_offset,
                                    [sprite.offset.x, sprite.offset.y],
                                );

                                // Render the sprite
                                render_gate.render(&render_state, |mut tess_gate| {
                                    tess_gate.render(&*sprite_tess)
                                })?;
                            }

                            Ok(())
                        },
                    )
                },
            )
            .assume()
            .into_result()
            .expect("Could not render");

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
                        let window_size = winit_window.inner_size();
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

        drop(span_render_guard);

        let span_swap_buffers = info_span!("swap_buffers");
        let span_swap_buffers_guard = span_swap_buffers.enter();

        #[cfg(not(wasm))]
        self.surface.swap_buffers().unwrap();

        drop(span_swap_buffers_guard);
    }

    #[tracing::instrument(skip(self, world))]
    pub(crate) fn handle_image_asset_event(&mut self, world: &mut World) {
        let Self {
            surface,
            pending_textures,
            texture_cache,
            image_asset_event_reader,
            ..
        } = self;

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

        // for every window resize event
        for event in image_asset_event_reader.iter(&image_asset_events) {
            match event {
                AssetEvent::Created { handle } => {
                    if let Some(image) = image_assets.get(handle) {
                        texture_cache.insert(
                            handle.clone(),
                            Starc::new(Mutex::new(upload_texture(image))),
                        );
                    } else {
                        pending_textures.push(handle.clone());
                    }
                }
                AssetEvent::Modified { handle } => {
                    if let Some(image) = image_assets.get(handle) {
                        texture_cache.insert(
                            handle.clone(),
                            Starc::new(Mutex::new(upload_texture(image))),
                        );
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
