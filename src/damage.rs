use bevy::prelude::*;

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>();
    }
}

/// A component indicating how much health something has, or in other words, how much damage
/// something can take before being destroyed.
#[derive(Component, Deref, DerefMut)]
pub struct Health(pub i32);

/// A component that indicates that an entity can currently be damaged.
///
/// In other words, something that has [`Health`] but isn't [`Damageable`] is currently invincible.
#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct Damageable;

/// Event emitted when an entity is damaged
pub struct DamageEvent {
    pub attack_entity: Entity,
    pub damaged_entity: Entity,
    pub damage: i32,
}
