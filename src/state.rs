#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum State {
    IDLE,
    RUNNING,
    ATTACKING,
    KNOCKED,
    DYING,
}
