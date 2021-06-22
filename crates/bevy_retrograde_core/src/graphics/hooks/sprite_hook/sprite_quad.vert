attribute vec2 v_pos;
attribute vec2 v_uv;

varying vec2 uv;

uniform ivec2 camera_size;
uniform vec2 camera_position;
uniform bool camera_centered;

uniform bool pixel_perfect;

uniform sampler2D sprite_texture;
uniform ivec2 sprite_texture_size;
uniform bool sprite_centered;
uniform int sprite_flip;
uniform ivec2 sprite_tileset_grid_size;
uniform int sprite_tileset_index;
uniform vec3 sprite_position;
uniform vec2 sprite_offset;

struct SpriteUvAndSize {
  vec2 uv;
  ivec2 size;
};

SpriteUvAndSize calculate_sprite_uv_and_size() {
  vec2 uv = v_uv;
  
  // Flip sprite UVs if necessary
  if (sprite_flip == 1 || sprite_flip == 3) {
    uv = vec2(1.0 - uv.x, uv.y);
  }
  if (sprite_flip == 2 || sprite_flip == 3) {
    uv = vec2(uv.x, 1.0 - uv.y);
  }

  // If the sprite is a tileset ( we detect this by checking the
  // tilesheet grid size is not 0 )
  if (sprite_tileset_grid_size.x != 0 && sprite_tileset_grid_size.y != 0) {
    // Get the number of tiles in the sheet
    ivec2 tile_count = sprite_texture_size / sprite_tileset_grid_size;

    // Get the position of the tile in the sprite sheet
    int y = sprite_tileset_index / tile_count.x;
    ivec2 tile_pos = ivec2(
      sprite_tileset_index - y * tile_count.x,
      y
    );

    // Adjust the uv to select the correct portion of the tileset
    uv = uv / vec2(tile_count) + 1.0 / vec2(tile_count) * vec2(tile_pos);

    // Return the UV and the size of the sprite
    return SpriteUvAndSize(uv, ivec2(sprite_tileset_grid_size));

  } else {
    // Return the size of the sprite
    return SpriteUvAndSize(uv, sprite_texture_size);
  }
}

// Define our own round function because WebGL1 doesn't come with
float round_f(float num) {
  float fractional = num - floor(num);
  if (fractional >= 0.5) {
    return ceil(num);
  } else {
    return floor(num);
  }
}

void main() {
  // Calculate sprite UVs
  SpriteUvAndSize sprite_uv_and_size = calculate_sprite_uv_and_size();
  ivec2 sprite_size = sprite_uv_and_size.size;
  uv = sprite_uv_and_size.uv;

  // Get the camera position, possibly adjusted to center the view
  vec2 adjusted_camera_pos = camera_position;
  if (camera_centered) {
    adjusted_camera_pos -= vec2(camera_size) / 2.0;
  }

  // Round the sprite position if it is in pixel-perfect mode
  vec3 sprite_pos_adjusted = sprite_position;
  if (pixel_perfect) {
    sprite_pos_adjusted = vec3(
      round_f(sprite_position.x),
      round_f(sprite_position.y),
      sprite_position.z
    );
  }

  // Get the pixel screen position of the center of the sprite
  vec2 screen_pos = sprite_pos_adjusted.xy - adjusted_camera_pos + sprite_offset;

  // Get the local position of the vertex in pixels
  vec2 vertex_pos = v_pos * vec2(sprite_size);

  // Center the sprite if necessary
  if (sprite_centered) {
    vertex_pos -= vec2(sprite_size) / 2.0;
  }

  // Calculate the normalized coordinate of this vertice
  vec2 norm_pos = ((vertex_pos + screen_pos) / vec2(camera_size) - 0.5) * 2.0;

  // Normalize the sprite Z component, allocating 2048 layers -1023 to 1024
  float norm_z = (-sprite_position.z + 1024.0) / 2048.0;

  // Invert the y component
  vec2 pos = norm_pos * vec2(1.0, -1.0);

  gl_Position = vec4(pos, norm_z, 1.);
}
