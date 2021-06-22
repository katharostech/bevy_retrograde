varying vec2 uv;

uniform sampler2D sprite_texture;

void main() {
  gl_FragColor = texture2D(sprite_texture, uv);
}
