attribute vec2 v_pos;
attribute vec2 v_uv;
attribute vec4 v_color;

varying vec2 uv;
varying vec4 color;

uniform vec2 target_size;
uniform int widget_type;
uniform vec2 text_box_size;
uniform mat4 text_box_transform;

const int WIDGET_COLORED_TRIS = 0;
const int WIDGET_IMAGE_TRIS = 1;
const int WIDGET_TEXT = 2;

void main() {
  uv = v_uv;
  color = v_color;

  vec4 y_invert = vec4(1., -1., 1., 1.); // The y direction needs to be flipped

  if (widget_type == WIDGET_COLORED_TRIS || widget_type == WIDGET_IMAGE_TRIS) {
    gl_Position = vec4(v_pos / target_size * 2.0 - 1., 0., 1.) * y_invert;
  } else if (widget_type == WIDGET_TEXT) {
    vec4 base_pos = vec4(v_pos * text_box_size, 0., 1.) * text_box_transform;
    gl_Position = vec4(base_pos.xy / target_size * 2.0 - 1., 0., 1.) * y_invert;
  }
}
