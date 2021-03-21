in vec2 uv;

out vec4 frag_color;

uniform sampler2D sprite_texture;

void main() {
  frag_color = texture(sprite_texture, uv);
}
