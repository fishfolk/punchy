use bevy::prelude::{Component, Query, Transform};

#[derive(Component, Default)]
pub struct YSort(f32);

pub fn y_sort(mut query: Query<(&mut Transform, &YSort)>) {
    for (mut transform, ysort) in query.iter_mut() {
        transform.translation.z = ysort.0 - transform.translation.y;
    }
}
