//! Pre-made [camera pos-processing][`crate::components::Camera::custom_shader`] shaders

/// A CRT shader that can be used as a custom shader for a camera.
///
/// ```
/// // Spawn the camera
/// commands.spawn().insert_bundle(CameraBundle {
///     camera: Camera {
///         // Set our camera to have a fixed height and an auto-resized width
///         size: CameraSize::FixedHeight(100),
///         background_color: Color::new(0.2, 0.2, 0.2, 1.0),
///         custom_shader: Some(
///             CrtShader {
///                 // You can configure shader options here
///                 ..Default::default()
///             }
///             .get_shader(),
///         ),
///         ..Default::default()
///     },
///     ..Default::default()
/// });
/// ```
pub struct CrtShader {
    pub curvature_x: f32,
    pub curvature_y: f32,
    pub scan_line_amount: f32,
    pub scan_line_opacity: f32,
    pub aberration_amount: f32,
}

impl Default for CrtShader {
    fn default() -> Self {
        Self {
            curvature_x: 6.0,
            curvature_y: 4.0,
            scan_line_amount: 370.0,
            scan_line_opacity: 0.2,
            aberration_amount: 1.0,
        }
    }
}

impl CrtShader {
    pub fn get_shader(&self) -> String {
        // TODO: Use uniforms instead of string substitution
        include_str!("./shaders/crt_shader.glsl")
            .replace("{{CURVATURE_X}}", &format!("{:.6}", self.curvature_x))
            .replace("{{CURVATURE_Y}}", &format!("{:.6}", self.curvature_y))
            .replace(
                "{{ABERRATION_AMOUNT}}",
                &format!("{:.6}", self.aberration_amount),
            )
            .replace(
                "{{SCAN_LINE_AMOUNT}}",
                &format!("{:.6}", self.scan_line_amount),
            )
            .replace(
                "{{SCAN_LINE_OPACITY}}",
                &format!("{:.6}", self.scan_line_opacity),
            )
    }
}
