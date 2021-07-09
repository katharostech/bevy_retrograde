use std::ops::Range;

use bevy::{
    math::Vec3,
    prelude::{Entity, GlobalTransform, World},
};
use bevy_retrograde_core::{
    graphics::{
        FrameContext, Program, RenderHook, RenderHookRenderableHandle, SceneFramebuffer, Surface,
        Tess, TextureCache,
    },
    luminance::{
        self,
        blending::{Blending, Equation, Factor},
        context::GraphicsContext,
        depth_test::DepthComparison,
        pipeline::PipelineState,
        render_state::RenderState,
        shader::Uniform,
        tess::View,
        Semantics, UniformInterface, Vertex,
    },
};
use epaint::{ClippedShape, Shape};

/// The render hook responsible for rendering the UI
pub struct EpaintRenderHook {
    // egui_font_texture: Texture<Dim2, SRGBA8UI>,
    current_shape_batch: Option<Vec<(Range<usize>, GlobalTransform)>>,
    shape_program: Program<(), (), ShapeUniformInterface>,
    shape_tess: Tess<ShapeVert, u32>,
}

impl RenderHook for EpaintRenderHook {
    fn init(_window_id: bevy::window::WindowId, surface: &mut Surface) -> Box<dyn RenderHook>
    where
        Self: Sized,
    {
        // // Allocate texture to use for EGUI font
        // let egui_font_texture = surface
        //     .new_texture::<Dim2, SRGBA8UI>([1, 1], 0, PIXELATED_SAMPLER)
        //     .unwrap();

        let shape_program = surface
            .new_shader_program::<(), (), ShapeUniformInterface>()
            .from_strings(
                include_str!("render_hook/shape.vert"),
                None,
                None,
                include_str!("render_hook/shape.frag"),
            )
            .unwrap()
            .program;

        let shape_tess = surface
            .new_tess()
            .set_mode(luminance::tess::Mode::Triangle)
            .set_vertices(Vec::new())
            .set_indices(Vec::new())
            .build()
            .unwrap();

        Box::new(Self {
            // egui_font_texture,
            current_shape_batch: None,
            shape_program,
            shape_tess,
        })
    }

    fn prepare(
        &mut self,
        world: &mut World,
        surface: &mut Surface,
        _texture_cache: &mut TextureCache,
        _frame_context: &FrameContext,
    ) -> Vec<RenderHookRenderableHandle> {
        let Self {
            // egui_font_texture,
            current_shape_batch,
            shape_tess,
            ..
        } = self;

        // let fonts = world.get_resource::<epaint::text::Fonts>().unwrap();

        // // Update the EGUI font texture
        // let target_texture = fonts.texture();
        // let target_size_usize = target_texture.size();
        // let target_size = [target_size_usize[0] as u32, target_size_usize[1] as u32];
        // let actual_size = egui_font_texture.size();
        // // If sizes don't match, recreate the texture
        // if target_size != actual_size {
        //     *egui_font_texture = surface
        //         .new_texture::<Dim2, SRGBA8UI>([1, 1], 0, PIXELATED_SAMPLER)
        //         .unwrap();
        // }
        // egui_font_texture
        //     .upload_raw(GenMipmaps::No, &target_texture.pixels)
        //     .expect("Upload texture");

        // Query the world for shapes to render
        let mut shape_query = world.query::<(Entity, &Shape, &GlobalTransform)>();

        // Collect shapes into renderables
        let mut shape_batch = Vec::new();
        let mut renderables = Vec::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for (ent, shape, transform) in shape_query.iter(world) {
            // These are just to fix rust-analyzer inferrence
            let entity: Entity = ent;
            let shape: &Shape = shape;
            let transform: &GlobalTransform = transform;

            // Get shape index
            let index = shape_batch.len();

            // Tesselate the shape to convert it to a triangle mesh
            let mut mesh = epaint::tessellator::tessellate_shapes(
                vec![ClippedShape(epaint::emath::Rect::EVERYTHING, shape.clone())],
                epaint::tessellator::TessellationOptions {
                    // TODO: Make anti-aliasing settings configurable through a resource
                    anti_alias: true,
                    aa_size: 0.2,
                    pixels_per_point: 1.,
                    ..Default::default()
                },
                [1, 1],
            );

            // I think there's only one mesh if there's only one shape, but change this if there are
            // more
            debug_assert_eq!(mesh.len(), 1);
            let mesh = mesh.remove(0).1;

            // Push the mesh vertices into the vertices list
            let tri_idx_start = indices.len();
            let tri_idx_end = tri_idx_start + mesh.indices.len();
            indices.extend(mesh.indices.into_iter().map(|i| i + vertices.len() as u32));
            vertices.extend(mesh.vertices.into_iter().map(|v| ShapeVert {
                pos: VertexPosition::new([v.pos.x, v.pos.y]),
                uv: VertexUv::new([v.uv.x, v.uv.y]),
                color: VertexColor::new([
                    v.color.r() as f32 / 256.,
                    v.color.g() as f32 / 256.,
                    v.color.b() as f32 / 256.,
                    v.color.a() as f32 / 256.,
                ]),
            }));

            // Add the vertice range to the list of renderables
            shape_batch.push((tri_idx_start..tri_idx_end, *transform));

            // Add the renderable
            renderables.push(RenderHookRenderableHandle {
                identifier: index,
                is_transparent: true, // Just assume it could be transparent
                depth: transform.translation.z,
                entity: Some(entity),
            })
        }

        *current_shape_batch = Some(shape_batch);

        // Upload the vertices to the GPU
        *shape_tess = surface
            .new_tess()
            .set_mode(luminance::tess::Mode::Triangle)
            .set_vertices(vertices)
            .set_indices(indices)
            .build()
            .unwrap();

        renderables
    }

    fn render(
        &mut self,
        _world: &mut World,
        surface: &mut Surface,
        _texture_cache: &mut TextureCache,
        frame_context: &FrameContext,
        target_framebuffer: &SceneFramebuffer,
        // We only have one renderable for everything so we don't need to read this
        renderables: &[RenderHookRenderableHandle],
    ) {
        let Self {
            current_shape_batch,
            // egui_font_texture,
            shape_program,
            shape_tess,
            ..
        } = self;

        let shape_batch = current_shape_batch.as_ref().unwrap();

        // Create the render state
        let render_state = &RenderState::default()
            .set_face_culling(None)
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
                target_framebuffer,
                &PipelineState::default()
                    .enable_clear_color(false)
                    .enable_clear_depth(false),
                |_pipeline, mut shading_gate| {
                    shading_gate.shade(shape_program, |mut interface, uniforms, mut render_gate| {
                        // Set the camera and window uniforms
                        interface.set(
                            &uniforms.camera_position,
                            [frame_context.camera_pos.x, frame_context.camera_pos.y],
                        );
                        interface.set(
                            &uniforms.camera_size,
                            [
                                frame_context.target_sizes.low.x as i32,
                                frame_context.target_sizes.low.y as i32,
                            ],
                        );
                        interface.set(
                            &uniforms.camera_centered,
                            if frame_context.camera.centered { 1 } else { 0 },
                        );

                        // // Bind the egui texture
                        // let bound_texture = pipeline.bind_texture(egui_font_texture).unwrap();

                        // // Set the texture uniform
                        // interface.set(&uniforms.texture, bound_texture.binding());

                        for renderable in renderables {
                            let (vert_range, world_transform) = shape_batch
                                .get(renderable.identifier)
                                .expect("Tried to render non-existent renderable");

                            // Set sprite position and offset
                            debug_assert!(
                                -1024. < world_transform.translation.z
                                    && world_transform.translation.z <= 1024.,
                                "Shape world Z position ( {} ) must be between -1024 and \
                                1024. Please open an issue if this is a problem for you: \
                                https://github.com/katharostech/bevy_retrograde/issues",
                                world_transform.translation.z
                            );

                            let pos = world_transform.translation;
                            let mut rot_scale = *world_transform;
                            rot_scale.translation = Vec3::ZERO;
                            interface.set(&uniforms.position, [pos.x, pos.y, pos.z]);
                            interface.set(
                                &uniforms.rot_scale,
                                rot_scale.compute_matrix().to_cols_array_2d(),
                            );

                            // Render the shape
                            render_gate.render(render_state, |mut tess_gate| {
                                tess_gate.render(shape_tess.view(vert_range.clone()).unwrap())
                            })?;
                        }

                        Ok(())
                    })
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
struct ShapeVert {
    pos: VertexPosition,
    uv: VertexUv,
    color: VertexColor,
}

#[derive(UniformInterface)]
struct ShapeUniformInterface {
    // #[uniform(unbound)]
    // texture: Uniform<TextureBinding<Dim2, NormUnsigned>>,
    #[uniform(unbound)]
    position: Uniform<[f32; 3]>,
    #[uniform(unbound)]
    rot_scale: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    camera_position: Uniform<[f32; 2]>,
    #[uniform(unbound)]
    camera_size: Uniform<[i32; 2]>,
    #[uniform(unbound)]
    camera_centered: Uniform<i32>,
}
