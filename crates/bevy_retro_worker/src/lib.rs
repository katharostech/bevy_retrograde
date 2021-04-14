#[cfg(wasm)]
mod wasm;
#[cfg(wasm)]
pub use wasm::*;

#[cfg(not(wasm))]
mod native;
#[cfg(not(wasm))]
pub use native::*;
