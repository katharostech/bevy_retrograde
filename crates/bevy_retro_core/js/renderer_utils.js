export function setup_canvas_resize_callback(resize_handle) {
  window.addEventListener("resize", () => {
    resize_handle.set_new_size(window.innerWidth, window.innerHeight);
  });
}
