use leafwing_input_manager::Actionlike;
use serde::Deserialize;

#[derive(Debug, Copy, Clone, Actionlike, Deserialize, Eq, PartialEq, Hash)]
pub enum PlayerAction {
    Move,
    // Attacks
    FlopAttack,
    Throw,
    Shoot,
}

#[derive(Debug, Copy, Clone, Actionlike, Deserialize, Eq, PartialEq, Hash)]
pub enum CameraAction {
    Up,
    Down,
    Right,
    Left,
    ZoomIn,
    ZoomOut,
}

#[derive(Debug, Copy, Clone, Actionlike, Deserialize, Eq, PartialEq, Hash)]
pub enum MenuAction {
    Confirm,
    Forward,
    Backward,
    Pause,
    ToggleFullscreen,
}
