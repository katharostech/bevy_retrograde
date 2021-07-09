attribute vec2 v_pos;
attribute vec2 v_uv;
attribute vec4 v_color;

varying vec2 uv;
varying vec4 color;

uniform ivec2 camera_size;
uniform vec2 camera_position;
uniform bool camera_centered;

// uniform sampler2D texture;
uniform vec3 position;
uniform mat4 rot_scale;

void main() {
  uv = v_uv;
  color = v_color;

  // Get the camera position, possibly adjusted to center the view
  vec2 adjusted_camera_pos = camera_position;
  if (camera_centered) {
    adjusted_camera_pos -= vec2(camera_size) / 2.0;
  }

  // Get the pixel screen position of the center of the sprite
  vec2 screen_pos = position.xy - adjusted_camera_pos;

  // Apply rotation and scale to vertex
  vec2 v_transformed = (vec4(v_pos, 0., 0.) * rot_scale).xy;

  // Calculate the normalized coordinate of this vertice
  vec2 norm_pos = ((v_transformed + screen_pos) / vec2(camera_size) - 0.5) * 2.0;

  // Normalize the sprite Z component, allocating 2048 layers -1023 to 1024
  float norm_z = (-position.z + 1024.0) / 2048.0;

  // Invert the y component
  vec2 pos = norm_pos * vec2(1.0, -1.0);

  gl_Position = vec4(pos, norm_z, 1.);
}
