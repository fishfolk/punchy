use bevy::prelude::*;

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
