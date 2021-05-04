use std::collections::HashMap;

use bevy::{
    app::ManualEventReader,
    asset::{AssetPath, HandleId},
    core::Time,
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    },
    math::{Mat4, Vec3},
    prelude::{AssetServer, Assets, Handle, World},
    window::Windows,
};
use bevy_retro_core::{
    graphics::{
        Program, RenderHook, RenderHookRenderableHandle, SceneFramebuffer, Surface, Tess,
        TextureCache,
    },
    luminance::{
        self,
        blending::{Blending, Equation, Factor},
        context::GraphicsContext,
        face_culling::FaceCulling,
        pipeline::{PipelineState, TextureBinding},
        pixel::{NormRGBA8UI, NormUnsigned},
        render_state::RenderState,
        scissor::ScissorRegion,
        shader::Uniform,
        tess::View,
        texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler, Wrap},
        Semantics, UniformInterface, Vertex,
    },
    prelude::{Camera, Color, Image},
};
use bevy_retro_text::{prelude::*, rasterize_text_block};
use raui::{
    prelude::{
        CoordsMapping, DefaultInteractionsEngine, DefaultInteractionsEngineResult,
        DefaultLayoutEngine, InteractionsEngine, Rect, Renderer, TesselateRenderer,
    },
    renderer::tesselate::tesselation::{Batch, Tesselation, TesselationVerticesFormat},
};

use crate::UiApplication;

trait AssetPathExt {
    fn format_as_load_path(&self) -> String;
}

impl<'a> AssetPathExt for AssetPath<'a> {
    fn format_as_load_path(&self) -> String {
        self.path()
            .to_str()
            .expect("Only valid unicode paths are supported")
            .to_string()
            + &self
                .label()
                .map(|x| format!("#{}", x))
                .unwrap_or(String::from(""))
    }
}

/// The render hook responsible for rendering the UI
pub struct UiRenderHook {
    window_id: bevy::window::WindowId,
    current_ui_tesselation: Option<Tesselation>,
    text_tess: Tess<UiVert>,
    shader_program: Program<(), (), UiUniformInterface>,
    /// Cache of image handles that the UI is using
    image_cache: Vec<Handle<Image>>,
    handle_to_path: HashMap<HandleId, String>,
    /// Cache of fonts that the UI is using
    font_cache: Vec<Handle<Font>>,
    interactions: BevyInteractionsEngine,
    has_shown_clipping_warning: bool,
}

impl RenderHook for UiRenderHook {
    fn init(window_id: bevy::window::WindowId, surface: &mut Surface) -> Box<dyn RenderHook>
    where
        Self: Sized,
    {
        Box::new(Self {
            window_id,
            current_ui_tesselation: None,
            shader_program: surface
                .new_shader_program::<(), (), UiUniformInterface>()
                .from_strings(
                    include_str!("render_hook/ui.vert"),
                    None,
                    None,
                    include_str!("render_hook/ui.frag"),
                )
                .unwrap()
                .program,
            text_tess: surface
                .new_tess()
                .set_vertices(&QUAD_VERTS[..])
                .set_mode(luminance::tess::Mode::TriangleFan)
                .build()
                .unwrap(),

            // Font & Image handle cache
            font_cache: Default::default(),
            image_cache: Default::default(),
            handle_to_path: Default::default(),
            interactions: Default::default(),
            has_shown_clipping_warning: false,
        })
    }

    // Make sure the ui system runs before the sprite hook by making its priority higher ( the
    // default, and the priority of the sprite hook is 0 ). This way the text sprites we create in
    // this hook will be present when the sprite hook runs.
    fn priority(&self) -> i32 {
        1
    }

    fn prepare_low_res(
        &mut self,
        world: &mut World,
        texture_cache: &mut TextureCache,
        _surface: &mut Surface,
    ) -> Vec<RenderHookRenderableHandle> {
        // Get the camera
        let mut cameras_query = world.query::<&Camera>();
        let camera = cameras_query.iter(world).next().unwrap().clone();

        // Scope the borrow of the world and its resources
        let ui_tesselation = {
            // Update interactions
            self.interactions.update(world);

            // Get our bevy resources from the world
            let world_cell = world.cell();
            let bevy_windows = world_cell.get_resource::<Windows>().unwrap();
            let bevy_window = bevy_windows.get(self.window_id).unwrap();
            let time = world_cell.get_resource::<Time>().unwrap();
            let mut app = world_cell.get_resource_mut::<UiApplication>().unwrap();

            // Process the UI application
            app.animations_delta_time = time.delta_seconds();
            app.process();
            app.interact(&mut self.interactions)
                .expect("Couldn't run UI interactions");
            app.consume_signals();

            // For now we don't do image atlasses
            let atlases = HashMap::default();

            // Collect image sizes from the textures in the texture cache
            let image_sizes = texture_cache
                .iter()
                .filter_map(|(handle, texture)| {
                    let asset_path = self.handle_to_path.get(&handle.id)?;
                    let size = texture.size();
                    Some((
                        asset_path.clone(),
                        raui::prelude::Vec2 {
                            x: size[0] as f32,
                            y: size[1] as f32,
                        },
                    ))
                })
                .collect();

            // Get the coordinate mapping based on the size of the screen
            let target_size = camera.get_target_size(bevy_window);
            let coords_mapping = CoordsMapping::new(Rect {
                left: 0.,
                top: 0.,
                right: target_size.x as f32,
                bottom: target_size.y as f32,
            });

            // Calculate app layout
            app.layout(&coords_mapping, &mut DefaultLayoutEngine)
                .expect("Could not layout UI");

            // Tesselate the UI
            let ui_tesselation = TesselateRenderer::new(
                TesselationVerticesFormat::Interleaved,
                (),
                &atlases,
                &image_sizes,
            )
            .render(&app.rendered_tree(), &coords_mapping, &app.layout_data())
            .expect("Could not tesselate UI");

            ui_tesselation
        };

        // Store the UI tesselation in preparation for rendering
        self.current_ui_tesselation = Some(ui_tesselation);

        vec![
            // We only do one render pass so we create one renderable
            RenderHookRenderableHandle {
                identifier: 0,
                depth: i32::MAX, // We render on top of everything else
                is_transparent: true,
            },
        ]
    }

    fn render_low_res(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
        texture_cache: &mut TextureCache,
        target_framebuffer: &SceneFramebuffer,
        // We only have one renderable for everything so we don't need to read this
        _renderables: &[RenderHookRenderableHandle],
    ) {
        let Self {
            current_ui_tesselation,
            shader_program,
            font_cache,
            image_cache,
            handle_to_path,
            text_tess,
            has_shown_clipping_warning,
            ..
        } = self;

        // Get world resources
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let font_assets = world.get_resource::<Assets<Font>>().unwrap();

        // Get the UI tesselation
        let ui_tesselation = current_ui_tesselation.take().unwrap();

        // Collect vertices
        let vertices = ui_tesselation
            .vertices
            .as_interleaved()
            .unwrap()
            .iter()
            .map(|(pos, uv, color)| UiVert {
                pos: VertexPosition::new([pos.0.floor(), pos.1.floor()]),
                uv: VertexUv::new([uv.0, uv.1]),
                color: VertexColor::new([color.0, color.1, color.2, color.3]),
            })
            .collect::<Vec<_>>();

        // Upload the vertices to the GPU
        let tess = surface
            .new_tess()
            .set_mode(luminance::tess::Mode::Triangle)
            .set_vertices(vertices)
            .set_indices(ui_tesselation.indices)
            .build()
            .unwrap();
        let batches = ui_tesselation.batches;

        // Create the render state
        let mut render_state = RenderState::default()
            .set_blending_separate(
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
            )
            .set_face_culling(Some(FaceCulling {
                order: luminance::face_culling::FaceCullingOrder::CW,
                mode: luminance::face_culling::FaceCullingMode::Back,
            }))
            .set_depth_test(None); // Disable depth test so the UI always renders on top

        // Get list of image handles used by the UI
        let mut image_handles = Vec::new();
        for image_path in batches.iter().filter_map(|x| match x {
            Batch::ImageTriangles(image, _) => Some(image),
            _ => None,
        }) {
            // Get the texture handle
            let texture_handle: Handle<Image> =
                asset_server.get_handle(HandleId::from(AssetPath::from(image_path.as_str())));

            // Map the handle ID to the handle path if necessary
            //
            // TODO: This is just waiting on this Bevy PR to be merged:
            // https://github.com/bevyengine/bevy/pull/1290
            handle_to_path
                .entry(texture_handle.id)
                .or_insert(image_path.clone());

            // Load the texture if loading has not started yet
            match asset_server.get_load_state(&texture_handle) {
                bevy::asset::LoadState::NotLoaded => {
                    image_cache.push(asset_server.load(image_path.as_str()));
                }
                _ => (),
            }
            image_handles.push(texture_handle);
        }
        // Update the image cache with the new handle list
        *image_cache = image_handles;

        // Get list of font handles used by the UI
        let mut font_handles = Vec::new();
        for font_path in batches.iter().filter_map(|x| match x {
            Batch::ExternalText(_, batch) => Some(&batch.font),
            _ => None,
        }) {
            // Get the font handle
            let font_handle: Handle<Font> =
                asset_server.get_handle(HandleId::from(AssetPath::from(font_path.as_str())));

            // Load the font if loading has not started yet
            match asset_server.get_load_state(&font_handle) {
                bevy::asset::LoadState::NotLoaded => {
                    font_cache.push(asset_server.load(font_path.as_str()));
                }
                _ => (),
            }
            font_handles.push(font_handle);
        }
        // Update the image cache with the new handle list
        *font_cache = font_handles;

        // Raterize text blocks to textures
        // TODO: Cache text block rasterizations and reuse if they haven't been changed
        let mut text_block_textures = HashMap::new();
        for (widget, batch) in batches.iter().filter_map(|x| match x {
            Batch::ExternalText(widget, batch) => Some((widget, batch)),
            _ => None,
        }) {
            // Get the font handle
            let font_handle: Handle<Font> =
                asset_server.get_handle(HandleId::from(AssetPath::from(batch.font.as_str())));
            // Load the font
            let font = if let Some(font) = font_assets.get(font_handle) {
                font
            } else {
                continue;
            };

            // Collect text info
            let text = Text {
                text: batch.text.clone(),
                color: Color {
                    r: batch.color.0,
                    g: batch.color.1,
                    b: batch.color.2,
                    a: batch.color.3,
                },
            };
            let text_block = TextBlock {
                width: batch.box_size.0.round() as u32,
                align: match batch.alignment {
                    raui::prelude::TextBoxAlignment::Left => TextAlign::Left,
                    raui::prelude::TextBoxAlignment::Center => TextAlign::Center,
                    raui::prelude::TextBoxAlignment::Right => TextAlign::Right,
                },
            };

            // Rasterize the text block
            let image = rasterize_text_block(&text, font, Some(&text_block));

            // Upload the image to the GPU
            let (sprite_width, sprite_height) = image.dimensions();
            let sprite_size = [sprite_width, sprite_height];
            let pixels = image.as_raw();

            // Upload the sprite to the GPU
            let mut texture = surface
                .new_texture::<Dim2, NormRGBA8UI>(sprite_size, 0, PIXELATED_SAMPLER)
                .unwrap();
            texture.upload_raw(GenMipmaps::No, pixels).unwrap();

            text_block_textures.insert(widget.clone(), texture);
        }

        // Do the render
        surface
            .new_pipeline_gate()
            .pipeline(
                // Render to the scene framebuffer
                &target_framebuffer,
                &PipelineState::default().enable_clear_color(false),
                |pipeline, mut shading_gate| {
                    shading_gate.shade(
                        shader_program,
                        |mut interface, uniforms, mut render_gate| {
                            // Set the target size uniform
                            let target_size = target_framebuffer.size();
                            interface.set(
                                &uniforms.target_size,
                                [target_size[0] as f32, target_size[1] as f32],
                            );

                            for batch in batches {
                                match batch {
                                    Batch::ColoredTriangles(tris) => {
                                        // Set widget type uniform
                                        interface.set(&uniforms.widget_type, WIDGET_COLORED_TRIS);

                                        render_gate.render(&render_state, |mut tess_gate| {
                                            tess_gate.render(tess.view(tris).unwrap())
                                        })?;
                                    }
                                    Batch::ImageTriangles(texture_path, tris) => {
                                        let texture_handle = asset_server.get_handle(
                                            HandleId::from(AssetPath::from(texture_path.as_str())),
                                        );

                                        // Get the texture using the image handle
                                        let texture = if let Some(texture) =
                                            texture_cache.get_mut(&texture_handle)
                                        {
                                            texture
                                        } else {
                                            // Skip for this frame
                                            continue;
                                        };

                                        // Bind our texture
                                        let bound_texture = pipeline.bind_texture(texture).unwrap();

                                        // Set the texture uniforms
                                        interface.set(&uniforms.texture, bound_texture.binding());
                                        interface.set(&uniforms.widget_type, WIDGET_IMAGE_TRIS);

                                        // Render the block
                                        render_gate.render(&render_state, |mut tess_gate| {
                                            tess_gate.render(tess.view(tris).unwrap())
                                        })?;
                                    }
                                    Batch::ExternalText(widget, batch) => {
                                        // Get the texture
                                        let texture = if let Some(tex) =
                                            text_block_textures.get_mut(&widget)
                                        {
                                            tex
                                        } else {
                                            continue;
                                        };

                                        // Bind our texture
                                        let tex_size = texture.size();
                                        let bound_texture = pipeline.bind_texture(texture).unwrap();
                                        interface.set(&uniforms.widget_type, WIDGET_TEXT);

                                        let m = batch.matrix;

                                        // Set the text block transform
                                        interface.set(
                                            &uniforms.text_box_transform,
                                            [
                                                [m[0], m[4], m[8], m[12].round()],
                                                [m[1], m[5], m[9], m[13].round()],
                                                [m[2], m[6], m[10], m[14].round()],
                                                [m[3], m[7], m[11], m[15]],
                                            ],
                                        );
                                        // Set the text block size
                                        interface.set(
                                            &uniforms.text_box_size,
                                            [tex_size[0] as f32, tex_size[1] as f32],
                                        );

                                        // Set the texture uniform
                                        interface.set(&uniforms.texture, bound_texture.binding());

                                        // Render the block
                                        render_gate.render(&render_state, |mut tess_gate| {
                                            tess_gate.render(&*text_tess)
                                        })?;
                                    }
                                    Batch::FontTriangles(_, _, _) => {
                                        unimplemented!("Tesselated font rendering not implemented")
                                    }
                                    Batch::ClipPush(clip) => {
                                        // Calculate clipping rectangle x and y
                                        let matrix = Mat4::from_cols_array(&clip.matrix);

                                        // tl, tr, bl, br == top_left, top_right, bottom_left, bottom_right
                                        let tl = matrix.project_point3(Vec3::new(0.0, 0.0, 0.0));
                                        let tr = matrix.project_point3(Vec3::new(
                                            clip.box_size.0,
                                            0.0,
                                            0.0,
                                        ));
                                        let br = matrix.project_point3(Vec3::new(
                                            clip.box_size.0,
                                            clip.box_size.1,
                                            0.0,
                                        ));
                                        let bl = matrix.project_point3(Vec3::new(
                                            0.0,
                                            clip.box_size.1,
                                            0.0,
                                        ));
                                        let x1 = tl.x.min(tr.x).min(br.x).min(bl.x).round();
                                        let y1 = tl.y.min(tr.y).min(br.y).min(bl.y).round();
                                        let x2 = tl.x.max(tr.x).max(br.x).max(bl.x).round();
                                        let y2 = tl.y.max(tr.y).max(br.y).max(bl.y).round();
                                        let width = x2 - x1;
                                        let height = y2 - y1;

                                        // Set the clipping section for future renders
                                        if !*has_shown_clipping_warning {
                                            bevy::log::warn!(
                                            "Detected UI elements that use clipping, there are \
                                            some bugs in either RAUI or Bevy Retro under \
                                            certain circumstances where the clipping region \
                                            is incorrect. You may want to disable clipping if \
                                            the UI element fails to render correctly"
                                            );

                                            *has_shown_clipping_warning = true;
                                        }
                                        render_state =
                                            render_state.set_scissor(Some(ScissorRegion {
                                                x: x1 as u32,
                                                y: y1 as u32,
                                                width: width as u32,
                                                height: height as u32,
                                            }));
                                    }
                                    Batch::ClipPop => {
                                        // Clear the render clipping area
                                        render_state = render_state.set_scissor(None);
                                    }
                                    Batch::None => (),
                                }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "v_pos", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "v_uv", repr = "[f32; 2]", wrapper = "VertexUv")]
    Uv,
    #[sem(name = "v_color", repr = "[f32; 4]", wrapper = "VertexColor")]
    Color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "VertexSemantics")]
struct UiVert {
    pos: VertexPosition,
    uv: VertexUv,
    color: VertexColor,
}

#[derive(UniformInterface)]
struct UiUniformInterface {
    target_size: Uniform<[f32; 2]>,

    texture: Uniform<TextureBinding<Dim2, NormUnsigned>>,

    /// Should be on eof the widget type constants below
    widget_type: Uniform<i32>,

    #[uniform(unbound)]
    text_box_transform: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    text_box_size: Uniform<[f32; 2]>,
}

/// Uniform widget type constant
const WIDGET_COLORED_TRIS: i32 = 0;
/// Uniform widget type constant
const WIDGET_IMAGE_TRIS: i32 = 1;
/// Uniform widget type constant
const WIDGET_TEXT: i32 = 2;

const PIXELATED_SAMPLER: Sampler = Sampler {
    wrap_r: Wrap::ClampToEdge,
    wrap_s: Wrap::ClampToEdge,
    wrap_t: Wrap::ClampToEdge,
    min_filter: MinFilter::Nearest,
    mag_filter: MagFilter::Nearest,
    depth_comparison: None,
};

// Quad vertices in a triangle fan
const QUAD_VERTS: [UiVert; 4] = [
    UiVert::new(
        VertexPosition::new([0.0, 0.0]),
        VertexUv::new([0.0, 0.0]),
        VertexColor::new([1., 1., 1., 1.]),
    ),
    UiVert::new(
        VertexPosition::new([1.0, 0.0]),
        VertexUv::new([1.0, 0.0]),
        VertexColor::new([1., 1., 1., 1.]),
    ),
    UiVert::new(
        VertexPosition::new([1.0, 1.0]),
        VertexUv::new([1.0, 1.0]),
        VertexColor::new([1., 1., 1., 1.]),
    ),
    UiVert::new(
        VertexPosition::new([0.0, 1.0]),
        VertexUv::new([0.0, 1.0]),
        VertexColor::new([1., 1., 1., 1.]),
    ),
];

#[derive(Default)]
struct BevyInteractionsEngine {
    engine: DefaultInteractionsEngine,
    _keyboard_event_reader: ManualEventReader<KeyboardInput>,
    _cursor_moved_event_reader: ManualEventReader<MouseMotion>,
    _mouse_motion_event_reader: ManualEventReader<MouseMotion>,
    _mouse_button_event_reader: ManualEventReader<MouseButtonInput>,
    _mouse_scroll_event_reader: ManualEventReader<MouseWheel>,
}

impl BevyInteractionsEngine {
    fn update(&mut self, _world: &mut World) {}
}

impl InteractionsEngine<DefaultInteractionsEngineResult, ()> for BevyInteractionsEngine {
    fn perform_interactions(
        &mut self,
        app: &mut raui::prelude::Application,
    ) -> Result<DefaultInteractionsEngineResult, ()> {
        self.engine.perform_interactions(app)
    }
}
