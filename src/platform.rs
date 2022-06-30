//! Systems and utilities related to specific platform support or platform abstractions

use bevy::prelude::*;

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    #[cfg(not(target_arch = "wasm32"))]
    fn build(&self, _app: &mut App) {}

    #[cfg(target_arch = "wasm32")]
    fn build(&self, app: &mut App) {
        app.add_system(wasm::update_canvas_size);
    }
}

/// WASM platform support
#[cfg(target_arch = "wasm32")]
mod wasm {
    use bevy::prelude::*;

    /// System to update the canvas size to match the size of the browser window
    pub fn update_canvas_size(mut windows: ResMut<Windows>) {
        // Get the browser window size
        let browser_window = web_sys::window().unwrap();
        let window_width = browser_window.inner_width().unwrap().as_f64().unwrap();
        let window_height = browser_window.inner_height().unwrap().as_f64().unwrap();

        let window = windows.get_primary_mut().unwrap();

        // Set the canvas to the browser size
        window.set_resolution(window_width as f32, window_height as f32);
    }
}
