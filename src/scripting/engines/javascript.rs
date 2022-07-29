#[cfg(not(target = "wasm32"))]
mod native;

#[cfg(not(target = "wasm32"))]
pub use native::JavaScriptEngine;

#[cfg(target = "wasm32")]
mod wasm;

#[cfg(target = "wasm32")]
pub use native::JavaScriptEngine;
