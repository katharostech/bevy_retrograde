use luminance::{
    blending::{Blending, Equation, Factor},
    context::GraphicsContext,
    pipeline::PipelineState,
    pixel::{NormRGBA8UI, RGBA32F},
    render_state::RenderState,
    texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler, Wrap},
};
use luminance_derive::*;
use luminance_front::{framebuffer::Framebuffer, shader::Program, tess::Tess};

use crate::components::{SpriteSheet, Visible};

use super::*;

#[derive(Copy, Clone, Debug, Semantics)]
pub enum SpriteVertexData {
    #[sem(name = "position", repr = "[i32; 2]", wrapper = "SpritePosition")]
    Position,
    #[sem(name = "size", repr = "[u32; 2]", wrapper = "SpriteSize")]
    Size,
    #[sem(name = "texture_index", repr = "u32", wrapper = "SpriteTextureIndex")]
    TextureIndex,
    #[sem(
        name = "atlas_offset",
        repr = "[u32; 2]",
        wrapper = "SpriteAtlasOffset"
    )]
    AtlasOffset,
    #[sem(name = "sprite_flip", repr = "u32", wrapper = "SpriteFlip")]
    SpriteFlip,
}

/// The scene framebuffer sampler
const PIXELATED_SAMPLER: Sampler = Sampler {
    wrap_r: Wrap::ClampToEdge,
    wrap_s: Wrap::ClampToEdge,
    wrap_t: Wrap::ClampToEdge,
    min_filter: MinFilter::Nearest,
    mag_filter: MagFilter::Nearest,
    depth_comparison: None,
};

pub(crate) struct LuminanceRenderer {
    pub(crate) surface: Surface,
    window_id: bevy::window::WindowId,
    sprite_program: Program<(), (), ()>,
    sprite_instance: Tess<()>,
    scene_framebuffer: Framebuffer<Dim2, RGBA32F, ()>,

    screen_program: Program<(), (), ()>,
}

impl LuminanceRenderer {
    pub fn init(window_id: bevy::window::WindowId, mut surface: Surface) -> Self {
        // Create the tesselator for the sprite instances
        let sprite_instance = surface
            .new_tess()
            .set_vertex_nb(4)
            .set_mode(luminance::tess::Mode::TriangleFan)
            .build()
            .unwrap();

        // Create the shader program for the sprite instances
        let built_sprite_program = surface
            .new_shader_program::<(), (), ()>()
            .from_strings(
                include_str!("shaders/sprite_quad.vert"),
                None,
                None,
                include_str!("shaders/sprite_quad.frag"),
            )
            .unwrap();

        // Create the shader program for the sprite instances
        let built_screen_program = surface
            .new_shader_program::<(), (), ()>()
            .from_strings(
                include_str!("shaders/screen.vert"),
                None,
                None,
                include_str!("shaders/screen.frag"),
            )
            .unwrap();

        // Log any shader compilation warnings
        for warning in built_sprite_program.warnings {
            warn!("Shader compile arning: {}", warning);
        }

        // Create the scene framebuffer that we will render the scene to
        let scene_framebuffer = surface
            // Because we are just initializing, we don't know what the framebuffer size should be
            // so we set it to zero
            .new_framebuffer([1, 1], 0, PIXELATED_SAMPLER)
            .expect("Create framebuffer");

        Self {
            window_id,
            surface,
            sprite_instance,
            sprite_program: built_sprite_program.program,
            screen_program: built_screen_program.program,
            scene_framebuffer,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        let Self {
            sprite_program,
            screen_program,
            sprite_instance,
            scene_framebuffer,
            surface,
            window_id,
        } = self;

        // Get the back buffer
        let back_buffer = surface.back_buffer().unwrap();

        // Build the queries and get the resources that we will need
        let mut cameras = world.query::<(&Camera, &WorldPosition)>();
        let mut sprites = world.query::<(
            &Handle<Image>,
            Option<&Handle<SpriteSheet>>,
            &Visible,
            &WorldPosition,
        )>();

        let image_assets = world.get_resource::<Assets<Image>>().unwrap();
        let sprite_sheet_assets = world.get_resource::<Assets<Image>>().unwrap();

        // Get the window this renderer is supposed to render to
        let winit_windows = world.get_resource::<WinitWindows>().unwrap();
        let window = winit_windows.get_window(*window_id).unwrap();

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

        // Calculate the target size of our scene framebuffer
        let size = window.inner_size();
        let aspect_ratio = size.width as f32 / size.height as f32;
        let target_size = match camera.size {
            CameraSize::FixedHeight(height) => {
                [(aspect_ratio * height as f32).floor() as u32, height]
            }
            _ => todo!(
                "Camera modes other than `FixedHeight` are not implemented yet. Open an issue to \
                 help prioritize it."
            ),
        };

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

        for (image_handle, sprite_sheet_handle, visible, world_position) in sprite_iter {
            // Skip invisible sprites
            if !**visible {
                continue;
            }

            // Get the loaded image, or else skip the sprite
            let image = if let Some(image) = image_assets.get(image_handle) {
                image
            } else {
                continue;
            };

            // Load the sprite sheet if any
            // TODO: Use this
            let _sprite_sheet = sprite_sheet_handle
                .map(|x| sprite_sheet_assets.get(x))
                .flatten();

            // Get the sprite image info
            let (sprite_width, sprite_height) = image.image.dimensions();
            let sprite_size = [sprite_width, sprite_height];
            let pixels = image.image.as_raw();

            // Upload the sprite to the GPU
            let mut texture = surface
                .new_texture::<Dim2, NormRGBA8UI>(sprite_size, 0, PIXELATED_SAMPLER)
                .unwrap();
            texture.upload_raw(GenMipmaps::No, pixels).unwrap();

            sprite_data.push((texture, world_position));
        }

        // Sort by depth
        sprite_data.sort_by(|(_, pos1), (_, pos2)| pos1.z.cmp(&pos2.z));

        // Do the render
        surface
            .new_pipeline_gate()
            .pipeline(
                // Render to the scene framebuffer
                &scene_framebuffer,
                &PipelineState::default().set_clear_color(color_to_array(camera.background_color)),
                |pipeline, mut shading_gate| {
                    shading_gate.shade(sprite_program, |mut interface, _, mut render_gate| {
                        // Set the camera position uniform
                        if let Ok(ref u) = interface.query().unwrap().ask("camera_position") {
                            interface.set(u, [camera_pos.x, camera_pos.y]);
                        }

                        // Set the camera size uniform
                        if let Ok(ref u) = interface.query().unwrap().ask("camera_size") {
                            interface.set(u, target_size);
                        }

                        for (texture, world_position) in &mut sprite_data {
                            // Bind our texture
                            let bound_texture = pipeline.bind_texture(texture).unwrap();

                            // Set the texture uniform
                            if let Ok(ref u) = interface.query().unwrap().ask("sprite_texture") {
                                interface.set(u, bound_texture.binding());
                            }

                            // Set the sprite position uniform
                            if let Ok(ref u) = interface.query().unwrap().ask("sprite_position") {
                                let pos = [world_position.x, world_position.y, world_position.z];
                                interface.set(u, pos);
                            }

                            // Render the sprite
                            render_gate.render(&render_state, |mut tess_gate| {
                                tess_gate.render(&*sprite_instance)
                            })?;
                        }

                        Ok(())
                    })
                },
            )
            .assume()
            .into_result()
            .expect("Could not render");

        // Render the scene framebuffer to the back buffer on a quad
        surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default(),
                |pipeline, mut shd_gate| {
                    // we must bind the offscreen framebuffer color content so that we can pass it to a shader
                    let bound_texture = pipeline.bind_texture(scene_framebuffer.color_slot())?;

                    shd_gate.shade(screen_program, |mut interface, _, mut rdr_gate| {
                        // Set the texture uniform
                        if let Ok(ref u) = interface.query().unwrap().ask("screen_texture") {
                            interface.set(u, bound_texture.binding());
                        }

                        rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                            tess_gate.render(&*sprite_instance)
                        })
                    })
                },
            )
            .assume();

        #[cfg(not(wasm))]
        self.surface.swap_buffers().unwrap();
    }
}

fn color_to_array(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}
