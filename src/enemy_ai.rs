//! Enemy fighter AI

use bevy::prelude::*;
use iyes_loopless::prelude::*;
use rand::{prelude::SliceRandom, Rng};

use crate::{
    consts,
    enemy::{Enemy, TripPointX},
    fighter_state::{FighterStateCollectSystems, Idling},
    player::Player,
    GameState,
};

pub struct EnemyAiPlugin;

impl Plugin for EnemyAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            set_target_near_player
                .run_in_state(GameState::InGame)
                .before(FighterStateCollectSystems),
        );
    }
}

/// A place that an enemy fighter is going to move to
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct EnemyTarget {
    pub position: Vec2,
}

// For enemys without current target, pick a new spot near the player as target
fn set_target_near_player(
    mut commands: Commands,
    mut enemies_query: Query<
        (Entity, &mut TripPointX),
        (With<Enemy>, With<Idling>, Without<EnemyTarget>),
    >,
    player_query: Query<&Transform, With<Player>>,
) {
    let mut rng = rand::thread_rng();
    let p_transforms = player_query.iter().collect::<Vec<_>>();
    let max_player_x = p_transforms
        .iter()
        .map(|transform| transform.translation.x)
        .max_by(f32::total_cmp);

    if let Some(max_player_x) = max_player_x {
        for (e_entity, mut e_trip_point_x) in enemies_query.iter_mut() {
            if let Some(p_transform) = p_transforms.choose(&mut rng) {
                if max_player_x > e_trip_point_x.0 {
                    e_trip_point_x.0 = f32::MIN;

                    let x_offset = rng.gen_range(-100.0..100.);
                    let y_offset = rng.gen_range(-100.0..100.);
                    commands.entity(e_entity).insert(EnemyTarget {
                        position: Vec2::new(
                            p_transform.translation.x + x_offset,
                            (p_transform.translation.y + y_offset)
                                .clamp(consts::MIN_Y, consts::MAX_Y),
                        ),
                    });
                }
            }
        }
    }
}
