use crate::attack::Drop;
use bevy::prelude::*;

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Last, lifetime_system);
    }
}

/// Component added to entities that should despawn after a timer.
#[derive(Component, Deref, DerefMut, Debug, Clone)]
pub struct Lifetime(pub Timer);

/// Despawn entities who's lifetime has expired
fn lifetime_system(
    mut commands: Commands,
    mut entities: Query<(Entity, &mut Lifetime, Option<&mut Drop>)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime, drop) in &mut entities {
        lifetime.tick(time.delta());

        if lifetime.finished() {
            if let Some(mut drop) = drop {
                drop.drop = true;
            } else {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
