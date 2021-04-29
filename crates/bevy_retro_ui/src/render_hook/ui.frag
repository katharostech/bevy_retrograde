varying vec4 color;
varying vec2 uv;

uniform sampler2D texture;
uniform int widget_type;

const int WIDGET_COLORED_TRIS = 0;
const int WIDGET_IMAGE_TRIS = 1;
const int WIDGET_TEXT = 2;


void main() {
  if (widget_type == WIDGET_IMAGE_TRIS) {
    gl_FragColor = color * texture2D(texture, uv);
  } else if (widget_type == WIDGET_COLORED_TRIS) {
    gl_FragColor = color;
  } else if (widget_type == WIDGET_TEXT) {
    gl_FragColor = color * texture2D(texture, uv);
  }
}
