use bevy::prelude::GlobalTransform;
use bevy::prelude::Transform;
use bevy_retrograde_core::luminance;
use bevy_retrograde_core::luminance::blending::Blending;
use bevy_retrograde_core::luminance::blending::Equation;
use bevy_retrograde_core::luminance::blending::Factor;
use bevy_retrograde_core::luminance::pipeline::PipelineState;
use bevy_retrograde_core::luminance::pipeline::TextureBinding;
use bevy_retrograde_core::luminance::pixel::NormRGBA8UI;
use bevy_retrograde_core::luminance::render_state::RenderState;
use bevy_retrograde_core::luminance::shader::Uniform;
use bevy_retrograde_core::luminance::texture::Dim2;
use bevy_retrograde_core::luminance::texture::GenMipmaps;
use bevy_retrograde_core::luminance::{Semantics, UniformInterface, Vertex};
use bevy_retrograde_core::{
    graphics::{RenderHook, *},
    luminance::{
        context::GraphicsContext,
        texture::{MagFilter, MinFilter, Sampler, Wrap},
    },
};
use heron::rapier_plugin::rapier::geometry::{ColliderHandle, ColliderSet};
use heron::CollisionShape;
use raqote::DrawOptions;
use raqote::PathBuilder;
use raqote::SolidSource;
use raqote::Source;

/// The scene framebuffer sampler
pub(crate) const SAMPLER: Sampler = Sampler {
    wrap_r: Wrap::ClampToEdge,
    wrap_s: Wrap::ClampToEdge,
    wrap_t: Wrap::ClampToEdge,
    min_filter: MinFilter::Linear,
    mag_filter: MagFilter::Linear,
    depth_comparison: None,
};

use crate::PhysicsDebugRendering;

#[derive(UniformInterface)]
struct ShaderUniformInterface {
    texture: Uniform<TextureBinding<Dim2, luminance::pixel::NormUnsigned>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "v_pos", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "VertexSemantics")]
struct Vert {
    pos: VertexPosition,
}

// Quad vertices in a triangle fan
const QUAD_VERTS: [Vert; 4] = [
    Vert::new(VertexPosition::new([-1.0, 1.0])),
    Vert::new(VertexPosition::new([1.0, 1.0])),
    Vert::new(VertexPosition::new([1.0, -1.0])),
    Vert::new(VertexPosition::new([-1.0, -1.0])),
];

pub struct PhysicsDebugRenderHook {
    shader_program: Program<(), (), ShaderUniformInterface>,
    texture: Texture<Dim2, NormRGBA8UI>,
    tesselator: Tess<Vert>,
}

impl RenderHook for PhysicsDebugRenderHook {
    fn init(_window_id: bevy::window::WindowId, surface: &mut Surface) -> Box<dyn RenderHook>
    where
        Self: Sized,
    {
        Box::new(PhysicsDebugRenderHook {
            shader_program: surface
                .new_shader_program::<(), (), ShaderUniformInterface>()
                .from_strings(
                    r#"
                    attribute vec2 v_pos;
                    varying vec2 uv;
                    void main() {
                        gl_Position = vec4(v_pos, 0., 1.);
                        uv = v_pos * 0.5 + 0.5;
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
                .expect("Create shader program")
                .program,
            tesselator: surface
                .new_tess()
                .set_vertices(&QUAD_VERTS[..])
                .set_mode(luminance::tess::Mode::TriangleFan)
                .build()
                .expect("Create tesselator"),
            texture: surface
                .new_texture([1, 1], 0, SAMPLER)
                .expect("Create texture"),
        })
    }

    fn prepare(
        &mut self,
        world: &mut bevy::prelude::World,
        _surface: &mut Surface,
        _texture_cache: &mut TextureCache,
        _frame_context: &FrameContext,
    ) -> Vec<RenderHookRenderableHandle> {
        let config = world.get_resource_or_insert_with(PhysicsDebugRendering::default);
        if let PhysicsDebugRendering::Enabled { .. } = *config {
            // We will render once on top of everything if debug rendering is enabled
            vec![RenderHookRenderableHandle {
                depth: f32::MAX,
                entity: None,
                identifier: 0,
                is_transparent: true,
            }]
        } else {
            // Don't render anything if debug rendering is disabled
            vec![]
        }
    }

    fn render(
        &mut self,
        world: &mut bevy::prelude::World,
        surface: &mut Surface,
        _texture_cache: &mut TextureCache,
        frame_context: &FrameContext,
        target_framebuffer: &SceneFramebuffer,
        _renderables: &[RenderHookRenderableHandle],
    ) {
        let Self {
            shader_program,
            tesselator,
            texture,
        } = self;

        // Collect the collision body list
        let mut collision_shapes =
            world.query::<(&CollisionShape, &ColliderHandle, &GlobalTransform)>();

        // Get the colliders resource
        let colliders = world.get_resource::<ColliderSet>().unwrap();

        // Get the rendering config
        let render_color = if let &PhysicsDebugRendering::Enabled { color } =
            world.get_resource::<PhysicsDebugRendering>().unwrap()
        {
            color
        } else {
            unreachable!("Tried to debug render physics objects when debug rendering is disabled");
        };

        // Create a raqote draw target to render to
        let [width, height] = target_framebuffer.size();
        let mut dt = raqote::DrawTarget::new(width as i32, height as i32);

        // Transform the draw target according to the camera position
        let scale = width as f32 / frame_context.target_sizes.low.x as f32;
        let pos = frame_context.camera_pos;
        let transform = raqote::Transform::identity();
        let transform = transform.post_scale(scale, scale);
        let transform = transform.post_translate(-raqote::Vector::new(pos.x, pos.y));
        dt.clear(SolidSource::from_unpremultiplied_argb(0, 0, 0, 0));
        dt.set_transform(&transform);

        // Create drawing source
        let draw_source = Source::Solid(SolidSource::from_unpremultiplied_argb(
            (render_color.a * 255.).round() as u8,
            (render_color.r * 255.).round() as u8,
            (render_color.g * 255.).round() as u8,
            (render_color.b * 255.).round() as u8,
        ));

        // Loop through collision shapes
        for (shape, collider_handle, transform) in collision_shapes.iter(world) {
            let shape: &CollisionShape = shape;
            let transform: &GlobalTransform = transform;
            let collider_handle: &ColliderHandle = collider_handle;
            let collider = if let Some(collider) = colliders.get(*collider_handle) {
                collider
            } else {
                continue;
            };

            let pos = transform.translation;

            match shape {
                CollisionShape::Sphere { radius } => todo!(),
                CollisionShape::Capsule {
                    half_segment,
                    radius,
                } => todo!(),
                CollisionShape::Cuboid { border_radius, .. } => todo!(),
                CollisionShape::ConvexHull { border_radius, .. } => {
                    if let Some(_border_radius) = border_radius {
                        let mut pb = PathBuilder::new();
                        let shape = collider.shape().as_round_convex_polygon().unwrap();
                        let aabb = shape.base_shape.local_aabb();
                        let width = aabb.extents().x;
                        let height = aabb.extents().y;
                        pb.rect(pos.x, -pos.y, width, height);

                        dt.fill(&pb.finish(), &draw_source, &DrawOptions::default());
                    }
                }
                CollisionShape::HeightField { size, heights } => todo!(),
            }
        }

        // Re-create the texture if the size is different
        if texture.size() != [width, height] {
            *texture = surface
                .new_texture::<Dim2, NormRGBA8UI>([width, height], 0, SAMPLER)
                .expect("Create graphics texture");
        }

        dt.write_png("test.gitignore.png").unwrap();

        // Convert pixels from BGRA to RGBA
        let textels = dt
            .get_data_u8()
            .iter()
            .fold(Vec::new(), |mut acc: Vec<Vec<u8>>, next| {
                if let Some(last) = acc.last_mut() {
                    if last.len() < 4 {
                        last.push(*next);
                    } else {
                        acc.push(vec![*next]);
                    }
                } else {
                    acc.push(vec![*next]);
                }

                acc
            })
            .iter()
            .map(|x| vec![x[2], x[1], x[0], x[3]])
            .flatten()
            .collect::<Vec<u8>>();

        // Upload pixels to the GPU texture
        texture
            .upload_raw(GenMipmaps::No, &textels)
            .expect("Upload GPU texture");

        // Render the texture to the framebuffer
        surface
            .new_pipeline_gate()
            .pipeline(
                &target_framebuffer,
                &PipelineState::default()
                    .enable_clear_color(false)
                    .enable_clear_depth(false),
                |pipeline, mut shd_gate| {
                    // we must bind the offscreen framebuffer color content so that we can pass it to a shader
                    let bound_texture = pipeline.bind_texture(texture).expect("Bind texture");

                    shd_gate.shade(shader_program, |mut interface, uniforms, mut rdr_gate| {
                        interface.set(&uniforms.texture, bound_texture.binding());

                        rdr_gate.render(
                            &RenderState::default().set_blending(Blending {
                                equation: Equation::Additive,
                                src: Factor::SrcAlpha,
                                dst: Factor::SrcAlphaComplement,
                            }),
                            |mut tess_gate| tess_gate.render(&*tesselator),
                        )
                    })
                },
            )
            .assume();
    }
}
