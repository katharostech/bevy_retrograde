use luminance::{context::GraphicsContext, pipeline::PipelineState, render_state::RenderState};
use luminance_derive::*;
use luminance_front::{shader::Program, tess::Tess};

use super::*;

#[derive(Copy, Clone, Debug, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "position", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    #[sem(name = "color", repr = "[u8; 3]", wrapper = "VertexRGB")]
    Color,
}

#[derive(Copy, Clone, Debug, Vertex)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
    #[allow(dead_code)]
    position: VertexPosition,

    #[allow(dead_code)]
    #[vertex(normalized = "true")]
    color: VertexRGB,
}

const VERTICES: [Vertex; 3] = [
    Vertex::new(
        VertexPosition::new([-0.5, -0.5]),
        VertexRGB::new([255, 0, 0]),
    ),
    Vertex::new(
        VertexPosition::new([0.5, -0.5]),
        VertexRGB::new([0, 255, 0]),
    ),
    Vertex::new(VertexPosition::new([0., 0.5]), VertexRGB::new([0, 0, 255])),
];

pub(crate) struct LuminanceRenderer {
    pub(crate) surface: Surface,
    program: Program<VertexSemantics, (), ()>,
    triangle: Tess<Vertex>,
}

impl Renderer for LuminanceRenderer {
    fn init(mut surface: Surface) -> Self {
        let triangle = surface
            .new_tess()
            .set_vertices(&VERTICES[..])
            .set_mode(luminance::tess::Mode::Triangle)
            .build()
            .unwrap();

        let program = surface
            .new_shader_program::<VertexSemantics, (), ()>()
            .from_strings(
                include_str!("shaders/viewport.vert"),
                None,
                None,
                include_str!("shaders/viewport.frag"),
            )
            .unwrap()
            .ignore_warnings();

        Self {
            surface,
            triangle,
            program,
        }
    }

    fn update(&mut self, render_options: &RetroRenderOptions, render_image: &RenderFrame) {
        let back_buffer = self.surface.back_buffer().unwrap();

        let Self {
            program, triangle, ..
        } = self;

        self.surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default().set_clear_color([0.1, 0.1, 0.2, 1.0]),
                |_, _| Ok(()),
            )
            .assume()
            .into_result()
            .expect("Could not render");

        let _render = self
            .surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default(),
                |_, mut shading_gate| {
                    shading_gate.shade(program, |_, _, mut render_gate| {
                        render_gate.render(&RenderState::default(), |mut tess_gate| {
                            tess_gate.render(&*triangle)
                        })
                    })
                },
            )
            .assume()
            .into_result()
            .expect("Could not render");

        #[cfg(not(wasm))]
        self.surface.swap_buffers();
    }
}
