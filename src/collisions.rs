use heron::PhysicsLayer;

#[derive(PhysicsLayer)]
pub enum BodyLayers {
    Enemy,
    Player,
    PlayerAttack,
}
