#[cfg(target = "wasm32")]
mod wasm_javascript;
#[cfg(target = "wasm32")]
pub use wasm_javascript as javascript;

#[cfg(not(target = "wasm32"))]
pub mod native_javascript;
#[cfg(not(target = "wasm32"))]
pub use native_javascript as javascript;
