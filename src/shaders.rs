pub struct CrtShader {
    pub curvature: f32,
    pub color_bleeding: f32,
    pub bleeding_range_x: f32,
    pub bleeding_range_y: f32,
    pub lines_distance: f32,
    pub scan_size: f32,
    pub scanline_alpha: f32,
    pub lines_velocity: f32,
}

impl Default for CrtShader {
    fn default() -> Self {
        Self {
            lines_velocity: 2.0,
            bleeding_range_x: 2.0,
            bleeding_range_y: 2.0,
            curvature: 1.07,
            lines_distance: 3.0,
            color_bleeding: 1.2,
            scan_size: 2.0,
            scanline_alpha: 0.9,
        }
    }
}

impl CrtShader {
    pub fn get_shader(&self) -> String {
        include_str!("./shaders/crt_shader.glsl")
            .replace("{{CURVATURE}}", &format!("{:.2}", self.curvature))
            .replace("{{COLOR_BLEEDING}}", &format!("{:.2}", self.color_bleeding))
            .replace(
                "{{BLEEDING_RANGE_X}}",
                &format!("{:.2}", self.bleeding_range_x),
            )
            .replace(
                "{{BLEEDING_RANGE_Y}}",
                &format!("{:.2}", self.bleeding_range_y),
            )
            .replace("{{LINES_DISTANCE}}", &format!("{:.2}", self.lines_distance))
            .replace("{{SCAN_SIZE}}", &format!("{:.2}", self.scan_size))
            .replace("{{SCANLINE_ALPHA}}", &format!("{:.2}", self.scanline_alpha))
            .replace("{{LINES_VELOCITY}}", &format!("{:.2}", self.lines_velocity))
    }
}
