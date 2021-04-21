use super::*;

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

pub(crate) struct SpriteHook {
    sprite_program: Program<(), (), SpriteUniformInterface>,
    sprite_tess: Tess<SpriteVert>,
    texture_cache: HashMap<Handle<Image>, Starc<Mutex<Texture<Dim2, NormRGBA8UI>>>>,
    image_asset_event_reader: ManualEventReader<AssetEvent<Image>>,
    pending_textures: Vec<Handle<Image>>,
    current_sprite_batch: Option<Vec<Entity>>,
}

impl SpriteHook {
    #[tracing::instrument(skip(self, world, surface))]
    pub(crate) fn handle_image_asset_event(&mut self, world: &mut World, surface: &mut Surface) {
        let Self {
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

        // for every image asset event
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

impl RenderHook for SpriteHook {
    fn init(_window_id: bevy::window::WindowId, surface: &mut Surface) -> Box<dyn RenderHook> {
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
            texture_cache: Default::default(),
            image_asset_event_reader: Default::default(),
            pending_textures: Default::default(),
            current_sprite_batch: None,
        }) as Box<dyn RenderHook>
    }

    fn prepare_low_res(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
    ) -> Vec<RenderHookRenderableHandle> {
        // Upload any textures that have been created to the GPU
        self.handle_image_asset_event(world, surface);

        // Create the sprite query
        let mut sprites = world
            .query_filtered::<(Entity, &Visible, &WorldPosition), (With<Handle<Image>>, With<Sprite>)>();

        // Loop through and collect sprites
        let sprite_iter = sprites.iter(world);
        let mut sprite_entities = Vec::new();
        let mut renderables = Vec::new();
        for (ent, visible, pos) in sprite_iter {
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
            });
        }

        self.current_sprite_batch = Some(sprite_entities);

        renderables
    }

    fn render_low_res(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
        target_framebuffer: &SceneFramebuffer,
        renderables: &[RenderHookRenderableHandle],
    ) {
        let Self {
            sprite_program,
            sprite_tess,
            texture_cache,
            current_sprite_batch,
            ..
        } = self;
        let target_size = target_framebuffer.size();

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

        // Do the render
        surface
            .new_pipeline_gate()
            .pipeline(
                // Render to the scene framebuffer
                &target_framebuffer,
                &PipelineState::default().enable_clear_color(false),
                |pipeline, mut shading_gate| {
                    shading_gate.shade(
                        sprite_program,
                        |mut interface, uniforms, mut render_gate| {
                            // Set the camera uniforms
                            interface.set(&uniforms.camera_position, [camera_pos.x, camera_pos.y]);
                            interface.set(&uniforms.camera_size, [target_size[0] as i32, target_size[1] as i32]);
                            interface.set(&uniforms.camera_centered, if camera.centered { 1 } else { 0 });

                            for renderable in
                                renderables
                            {
                                let sprite_entity = current_sprite_batch.as_ref().unwrap().get(renderable.identifier).unwrap();
                                let (image_handle, sprite, sprite_sheet_handle, world_position) = sprites.get(world, *sprite_entity).unwrap();

                                let sprite_sheet = sprite_sheet_handle.map(|x| sprite_sheet_assets.get(x)).flatten();

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

        self.current_sprite_batch = None;
    }
}
