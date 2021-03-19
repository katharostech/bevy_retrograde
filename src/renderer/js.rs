use super::BrowserResizeHandle;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js/renderer_utils.js")]
extern "C" {
    pub fn setup_canvas_resize_callback(canvas: BrowserResizeHandle);
}
