varying vec2 uv;

uniform ivec3 sprite_position;
uniform int sprite_flip;
uniform ivec2 sprite_tileset_grid_size;
uniform int sprite_tileset_index;
uniform ivec2 camera_position;
uniform ivec2 camera_size;
uniform sampler2D sprite_texture;

void main() {
  gl_FragColor = texture2D(sprite_texture, uv);
}
