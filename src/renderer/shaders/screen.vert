out vec2 uv;

uniform uvec2 camera_size;
uniform uvec2 window_size;
uniform float pixel_aspect_ratio;

vec2[4] QUAD_VERTS = vec2[](
  vec2(-1., -1.),
  vec2( 1., -1.),
  vec2( 1.,  1.),
  vec2(-1.,  1.)
);

void main() {
  vec2 vertex_pos_base = QUAD_VERTS[gl_VertexID];

  float screen_aspect_ratio = float(window_size.x) / float(window_size.y);
  float camera_aspect_ratio = float(camera_size.x) / float(camera_size.y);

  vec2 pos = vertex_pos_base * vec2(1.0, screen_aspect_ratio / camera_aspect_ratio)
    / vec2(1.0, pixel_aspect_ratio);

  gl_Position = vec4(pos, 0., 1.);
  uv = vertex_pos_base * .5 + .5;
}
