in vec2 uv;

out vec4 frag_color;

uniform sampler2D screen_texture;

void main() {
  frag_color = vec4(texture(screen_texture, uv).rgb, 1.);
}
