use luminance::{
    blending::{Blending, Equation, Factor},
    context::GraphicsContext,
    depth_test::DepthComparison,
    pipeline::{PipelineState, TextureBinding},
    pixel::NormUnsigned,
    render_state::RenderState,
    shader::Uniform,
    UniformInterface, Vertex,
};

use crate::{graphics::*, prelude::*, renderer::backend::*};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "VertexSemantics")]
struct SpriteVert {
    pos: VertexPosition,
    uv: VertexUv,
}

// Quad vertices in a triangle fan
const SPRITE_VERTS: [SpriteVert; 4] = [
    SpriteVert::new(VertexPosition::new([0.0, 1.0]), VertexUv::new([0.0, 1.0])),
    SpriteVert::new(VertexPosition::new([1.0, 1.0]), VertexUv::new([1.0, 1.0])),
    SpriteVert::new(VertexPosition::new([1.0, 0.0]), VertexUv::new([1.0, 0.0])),
    SpriteVert::new(VertexPosition::new([0.0, 0.0]), VertexUv::new([0.0, 0.0])),
];

#[derive(UniformInterface)]
struct SpriteUniformInterface {
    camera_position: Uniform<[f32; 2]>,
    camera_size: Uniform<[i32; 2]>,
    camera_centered: Uniform<i32>,

    pixel_perfect: Uniform<i32>,

    sprite_texture: Uniform<TextureBinding<Dim2, NormUnsigned>>,
    sprite_texture_size: Uniform<[i32; 2]>,
    sprite_flip: Uniform<i32>,
    sprite_centered: Uniform<i32>,
    sprite_tileset_grid_size: Uniform<[i32; 2]>,
    sprite_tileset_index: Uniform<i32>,
    sprite_position: Uniform<[f32; 3]>,
    sprite_offset: Uniform<[f32; 2]>,
}

pub(crate) struct SpriteHook {
    sprite_program: Program<(), (), SpriteUniformInterface>,
    sprite_tess: Tess<SpriteVert>,
    current_sprite_batch: Option<Vec<Entity>>,
    window_id: bevy::window::WindowId,
}

impl RenderHook for SpriteHook {
    fn init(window_id: bevy::window::WindowId, surface: &mut Surface) -> Box<dyn RenderHook> {
        // Intern shader uniform names
        #[cfg(wasm)]
        {
            use wasm_bindgen::intern;
            intern("camera_position");
            intern("camera_size");
            intern("camera_centered");
            intern("sprite_texture");
            intern("sprite_texture_size");
            intern("sprite_flip");
            intern("sprite_centered");
            intern("sprite_tileset_grid_size");
            intern("sprite_tileset_index");
            intern("sprite_tileset_index");
            intern("sprite_position");
            intern("sprite_offset");
        }

        // Create the tesselator for the sprites
        let sprite_tess = surface
            .new_tess()
            .set_vertices(&SPRITE_VERTS[..])
            .set_mode(luminance::tess::Mode::TriangleFan)
            .build()
            .unwrap();

        // Create the shader program for the sprite instances
        let built_sprite_program = surface
            .new_shader_program::<(), (), SpriteUniformInterface>()
            .from_strings(
                include_str!("sprite_hook/sprite_quad.vert"),
                None,
                None,
                include_str!("sprite_hook/sprite_quad.frag"),
            )
            .unwrap();

        Box::new(Self {
            sprite_program: built_sprite_program.program,
            sprite_tess,
            current_sprite_batch: None,
            window_id,
        }) as Box<dyn RenderHook>
    }

    fn prepare(
        &mut self,
        world: &mut World,
        _texture_cache: &mut TextureCache,
        _surface: &mut Surface,
    ) -> Vec<RenderHookRenderableHandle> {
        self.current_sprite_batch = None;

        // Create the sprite query
        let mut sprites = world
            .query_filtered::<(Entity, &Visible, &WorldPosition, &Sprite), (With<Handle<Image>>, With<Sprite>)>();

        // Loop through and collect sprites
        let sprite_iter = sprites.iter(world);
        let mut sprite_entities = Vec::new();
        let mut renderables = Vec::new();

        for (ent, visible, pos, sprite) in sprite_iter {
            // Skip invisible sprites
            if !**visible {
                continue;
            }

            sprite_entities.push(ent);
            renderables.push(RenderHookRenderableHandle {
                // Set the identifier to the index of the sprite entity in the sprite entities list
                identifier: sprite_entities.len() - 1,
                depth: pos.z,
                // Any sprite could be transparent so we just mark it as such
                is_transparent: true,
                low_resolution: sprite.pixel_perfect,
                entity: Some(ent),
            });
        }

        self.current_sprite_batch = Some(sprite_entities);

        renderables
    }

    fn render(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
        texture_cache: &mut TextureCache,
        target_framebuffer: &SceneFramebuffer,
        renderables: &[RenderHookRenderableHandle],
    ) {
        let Self {
            sprite_program,
            sprite_tess,
            current_sprite_batch,
            window_id,
            ..
        } = self;

        // Calculate the low resolution target size from the camera and the bevy window
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
        let bevy_windows = world.get_resource::<Windows>().unwrap();
        let bevy_window = bevy_windows.get(*window_id).unwrap();
        let low_res_target_size = camera.get_target_size(&bevy_window);

        // Create the sprite query
        let mut sprites = world.query::<(
            &Handle<Image>,
            &Sprite,
            Option<&Handle<SpriteSheet>>,
            &WorldPosition,
        )>();

        // Get the camera
        let mut cameras = world.query::<(&Camera, &WorldPosition)>();
        let mut camera_iter = cameras.iter(world);
        let (camera, camera_pos) = if let Some(camera_components) = camera_iter.next() {
            camera_components
        } else {
            return;
        };
        if camera_iter.next().is_some() {
            panic!("Only one Retro camera is supported");
        }

        // Get the spritesheet assets
        let sprite_sheet_assets = world.get_resource::<Assets<SpriteSheet>>().unwrap();

        // Create the render state
        let render_state = &RenderState::default()
            .set_blending_separate(
                Blending {
                    equation: Equation::Additive,
                    src: Factor::SrcAlpha,
                    dst: Factor::SrcAlphaComplement,
                },
                Blending {
                    equation: Equation::Additive,
                    src: Factor::SrcAlpha,
                    dst: Factor::SrcAlphaComplement,
                },
            )
            .set_depth_test(Some(DepthComparison::LessOrEqual));

        // Do the render
        surface
            .new_pipeline_gate()
            .pipeline(
                // Render to the scene framebuffer
                &target_framebuffer,
                &PipelineState::default()
                    .enable_clear_color(false)
                    .enable_clear_depth(false),
                |pipeline, mut shading_gate| {
                    shading_gate.shade(
                        sprite_program,
                        |mut interface, uniforms, mut render_gate| {
                            // Set the camera and window uniforms
                            interface.set(
                                &uniforms.camera_position,
                                [camera_pos.x.floor(), camera_pos.y.floor()],
                            );
                            interface.set(
                                &uniforms.camera_size,
                                [low_res_target_size[0] as i32, low_res_target_size[1] as i32],
                            );
                            interface.set(
                                &uniforms.camera_centered,
                                if camera.centered { 1 } else { 0 },
                            );

                            for renderable in renderables {
                                let sprite_entity = current_sprite_batch
                                    .as_ref()
                                    .expect("Missing sprite batch!")
                                    .get(renderable.identifier)
                                    .expect("Tried to render non-existent renderable");

                                let (image_handle, sprite, sprite_sheet_handle, world_position) =
                                    sprites.get(world, *sprite_entity).unwrap();

                                let sprite_sheet = sprite_sheet_handle
                                    .map(|x| sprite_sheet_assets.get(x))
                                    .flatten();

                                // Get the texture using the image handle
                                let texture =
                                    if let Some(texture) = texture_cache.get_mut(image_handle) {
                                        texture
                                    } else {
                                        // Skip it if the texture has not loaded
                                        continue;
                                    };

                                // Bind our texture
                                let bound_texture = pipeline.bind_texture(texture).unwrap();

                                // Set the texture uniform
                                interface.set(&uniforms.sprite_texture, bound_texture.binding());

                                // Set the pixel perfect mode
                                interface.set(
                                    &uniforms.pixel_perfect,
                                    if sprite.pixel_perfect { 1 } else { 0 },
                                );

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
                                interface.set(
                                    &uniforms.sprite_centered,
                                    if sprite.centered { 1 } else { 0 },
                                );

                                // Set the sprite tileset uniforms
                                let grid_size = sprite_sheet
                                    .map(|x| [x.grid_size.x as i32, x.grid_size.y as i32])
                                    .unwrap_or([0; 2]);
                                interface.set(&uniforms.sprite_tileset_grid_size, grid_size);
                                interface.set(
                                    &uniforms.sprite_tileset_index,
                                    sprite_sheet.map(|x| x.tile_index as i32).unwrap_or(0),
                                );

                                // Set sprite position and offset
                                debug_assert!(
                                    -1024. < world_position.z && world_position.z <= 1024.,
                                    "Sprite world Z position must be between -1024 and 1024. \
                                    Please open an issue if this is a problem for you: \
                                    https://github.com/katharostech/bevy_retrograde/issues"
                                );

                                let pos = if sprite.pixel_perfect {
                                    world_position.0.floor()
                                } else {
                                    world_position.0
                                };
                                interface.set(&uniforms.sprite_position, [pos.x, pos.y, pos.z]);
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
    }
}
