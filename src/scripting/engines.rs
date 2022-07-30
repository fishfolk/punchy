#[cfg(target_arch = "wasm32")]
pub mod wasm_javascript;
#[cfg(target_arch = "wasm32")]
pub use wasm_javascript as javascript;

#[cfg(not(target_arch = "wasm32"))]
pub mod native_javascript;
#[cfg(not(target_arch = "wasm32"))]
pub use native_javascript as javascript;
