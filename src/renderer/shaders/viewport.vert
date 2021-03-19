in vec2 position;
in vec3 color;

out vec3 v_color;

void main() {
  v_color = color;

  gl_Position = vec4(position, 0., 1.);
}
