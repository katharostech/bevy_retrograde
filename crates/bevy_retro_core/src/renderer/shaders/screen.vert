precision highp float;
precision highp int;

attribute vec2 v_pos;

varying vec2 uv;

uniform ivec2 camera_size;
uniform int camera_size_fixed;
uniform ivec2 window_size;
uniform float pixel_aspect_ratio;

void main() {
  float screen_aspect_ratio = float(window_size.x) / float(window_size.y);
  float camera_aspect_ratio = float(camera_size.x) / float(camera_size.y);

  vec2 pos = vec2(0, 0);

  // If the camera has a fixed aspect ratio
  if (camera_size_fixed == 0) {
    if (screen_aspect_ratio > camera_aspect_ratio * pixel_aspect_ratio) {
      pos = v_pos * vec2(camera_aspect_ratio / screen_aspect_ratio, 1.0)
        * vec2(pixel_aspect_ratio, 1.0);

    } else {
      pos = v_pos * vec2(1.0, screen_aspect_ratio / camera_aspect_ratio)
        / vec2(1.0, pixel_aspect_ratio);
    }

  // If the camera width is fixed
  } else if (camera_size_fixed == 1) {
    pos = v_pos * vec2(camera_aspect_ratio / screen_aspect_ratio, 1.0)
      * vec2(pixel_aspect_ratio, 1.0);

  // If the camera height is fixed
  } else if (camera_size_fixed == 2) {
    pos = v_pos * vec2(1.0, screen_aspect_ratio / camera_aspect_ratio)
      / vec2(1.0, pixel_aspect_ratio);
  }

  gl_Position = vec4(pos, 0., 1.);
  uv = v_pos * .5 + .5;
}
