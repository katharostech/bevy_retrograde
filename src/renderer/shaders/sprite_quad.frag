in vec2 uv;

out vec4 frag_color;

uniform ivec3 sprite_position;
uniform uint sprite_flip;
uniform uvec2 sprite_tileset_grid_size;
uniform uint sprite_tileset_index;
uniform ivec2 camera_position;
uniform uvec2 camera_size;
uniform sampler2D sprite_texture;

void main() {
  frag_color = texture(sprite_texture, uv);
}
