ivec2[4] quad_vert_positions = ivec2[](
  // Bottom left
  ivec2(0, 1),
  // Bottom right
  ivec2(1, 1),
  // Top Right
  ivec2(1, 0),
  // Top left
  ivec2(0, 0)
);

vec2[4] quad_vert_uvs = vec2[](
  // Bottom left
  vec2(0.0, 1.0),
  // Bottom right
  vec2(1.0, 1.0),
  // Top right
  vec2(1.0, 0.0),
  // top left
  vec2(0.0, 0.0)
);

out vec2 uv;

uniform uvec2 camera_size;
uniform ivec2 camera_position;
uniform bool camera_centered;

uniform sampler2D sprite_texture;
uniform bool sprite_centered;
uniform uint sprite_flip;
uniform uvec2 sprite_tileset_grid_size;
uniform uint sprite_tileset_index;
uniform ivec3 sprite_position;
uniform ivec2 sprite_offset;

struct SpriteUvAndSize {
  vec2 uv;
  ivec2 size;
};

SpriteUvAndSize calculate_sprite_uv_and_size() {
  vec2 uv = quad_vert_uvs[gl_VertexID];

  // Get the size of the sprite
  ivec2 sprite_sheet_size = textureSize(sprite_texture, 0);
  
  // Flip sprite UVs if necessary
  uint x_flip_bit = uint(1);
  uint y_flip_bit = uint(2);
  if ((sprite_flip & x_flip_bit) == x_flip_bit) {
    uv = vec2(1.0 - uv.x, uv.y);
  }
  if ((sprite_flip & y_flip_bit) == y_flip_bit) {
    uv = vec2(uv.x, 1.0 - uv.y);
  }

  // If the sprite is a tileset ( we detect this by checking the
  // tilesheet grid size is not 0 )
  if (sprite_tileset_grid_size.x != uint(0) && sprite_tileset_grid_size.y != uint(0)) {
    // Get the number of tiles in the sheet
    uvec2 tile_count = uvec2(sprite_sheet_size) / sprite_tileset_grid_size;

    // Get the position of the tile in the sprite sheet
    uvec2 tile_pos = uvec2(
      sprite_tileset_index % tile_count.x,
      sprite_tileset_index / tile_count.x
    );

    // Adjust the uv to select the correct portion of the tileset
    uv = uv / vec2(tile_count) + 1.0 / vec2(tile_count) * vec2(tile_pos);

    // Return the UV and the size of the sprite
    return SpriteUvAndSize(uv, ivec2(sprite_tileset_grid_size));

  } else {
    // Return the size of the sprite
    return SpriteUvAndSize(uv, sprite_sheet_size);
  }
}

void main() {
  // Calculate sprite UVs
  SpriteUvAndSize sprite_uv_and_size = calculate_sprite_uv_and_size();
  ivec2 sprite_size = sprite_uv_and_size.size;
  uv = sprite_uv_and_size.uv;

  // Get the camera position, possibly adjusted to center the view
  ivec2 adjusted_camera_pos = camera_position;
  if (camera_centered) {
    adjusted_camera_pos -= ivec2(camera_size) / 2;
  }

  // Get the pixel screen position of the center of the sprite
  ivec2 screen_pos = sprite_position.xy - adjusted_camera_pos + sprite_offset;

  // Get the vertex position in the quad
  ivec2 vertex_base_pos = quad_vert_positions[gl_VertexID];

  // Get the local position of the vertex in pixels
  ivec2 vertex_pos = vertex_base_pos * sprite_size;

  // Center the sprite if necessary
  if (sprite_centered) {
    vertex_pos -= sprite_size / 2;
  }

  // Calculate the normalized coordinate of this vertice
  vec2 norm_pos = (vec2(vertex_pos + screen_pos) / vec2(camera_size) - 0.5) * 2.0;

  // Normalize the sprite Z component, allocating 2048 layers -1023 to 1024
  float norm_z = float(-sprite_position.z + 1024) / float(2048.0);

  // Invert the y component
  vec2 pos = norm_pos * vec2(1.0, -1.0);

  gl_Position = vec4(pos, norm_z, 1.);
}
