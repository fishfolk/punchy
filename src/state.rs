#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum State {
    Idle,
    Running,
    Attacking,
    KnockedLeft,
    KnockedRight,
    Dying,
}
