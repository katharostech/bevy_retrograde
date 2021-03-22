ivec2[4] quad_vert_positions = ivec2[](
  // Bottom left
  ivec2(-1, -1),
  // Bottom right
  ivec2( 1, -1),
  // Top Right
  ivec2( 1,  1),
  // Top left
  ivec2(-1,  1)
);

vec2[4] quad_vert_uvs = vec2[](
  // Bottom left
  vec2(0.0, 0.0),
  // Bottom right
  vec2(1.0, 0.0),
  // Top right
  vec2(1.0, 1.0),
  // top left
  vec2(0.0, 1.0)
);

out vec2 uv;

uniform ivec3 sprite_position;
uniform uint sprite_flip;
uniform ivec2 camera_position;
uniform uvec2 camera_size;
uniform sampler2D sprite_texture;

void main() {
  // Calculate sprite UVs
  uv = quad_vert_uvs[gl_VertexID];
  uint x_flip_bit = uint(1);
  uint y_flip_bit = uint(2);
  if ((sprite_flip & x_flip_bit) == x_flip_bit) {
    uv = vec2(1.0 - uv.x, uv.y);
  }
  if ((sprite_flip & y_flip_bit) == y_flip_bit) {
    uv = vec2(uv.x, 1.0 - uv.y);
  }

  // Get the size of the sprite
  ivec2 sprite_size = textureSize(sprite_texture, 0);

  // Get the pixel screen position of the center of the sprite
  ivec2 screen_pos = sprite_position.xy - camera_position;

  // Get the vertex position in the quad
  ivec2 vertex_base_pos = quad_vert_positions[gl_VertexID];

  // Get the number of pixels offset this vertice should be along x
  int vertex_x_offset = 0;
  if (sprite_size.x % 2 == 0) {
    if (vertex_base_pos.x == -1) {
      vertex_x_offset = sprite_size.x / 2 - 1;
    } else {
      vertex_x_offset = sprite_size.x / 2;
    }
  } else {
      vertex_x_offset = (sprite_size.x - 1) / 2;
  }

  // Get the number of pixels offset this vertice should be along y
  int vertex_y_offset = 0;
  if (sprite_size.y % 2 == 0) {
    if (vertex_base_pos.y == -1) {
      vertex_y_offset = sprite_size.y / 2 - 1;
    } else {
      vertex_y_offset = sprite_size.y / 2;
    }
  } else {
      vertex_y_offset = (sprite_size.y - 1) / 2;
  }

  ivec2 vertex_offset = ivec2(vertex_x_offset, vertex_y_offset);

  // Calculate the normalized coordinate of this vertice
  vec2 norm_pos = vec2((vertex_offset * vertex_base_pos) + screen_pos) / vec2(camera_size) * 2.0;

  // Normalize the sprite Z component, allocating 2048 layers -1023 to 1024
  float norm_z = float(-sprite_position.z + 1024) / float(2048.0);

  // Invert the y component
  vec2 pos = norm_pos * vec2(1.0, -1.0);

  gl_Position = vec4(pos, norm_z, 1.);
}
