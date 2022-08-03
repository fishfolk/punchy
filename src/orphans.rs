//! Old code before the refactor that needs to be either cut out or worked into the new design

#[derive(Component)]
pub struct Knockback {
    pub direction: Vec2,
    pub duration: Timer,
}

pub fn knockback_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Knockback, Option<&Player>)>,
    player_movement_clamper: PlayerMovementClamper,
    time: Res<Time>,
) {
    let mut all_knockbacks = query.iter_mut().collect::<Vec<_>>();

    // Separate the finished knockbacks, and despawn them.

    let (finished_knockbacks, mut current_knockbacks): (Vec<_>, Vec<_>) = all_knockbacks
        .iter_mut()
        .partition(|(_, _, knockback, _)| knockback.duration.finished());

    for (entity, _, _, _) in &finished_knockbacks {
        commands.entity(*entity).remove::<Knockback>();
    }

    // Tick the timer for the current knockbacks.

    for (_, _, knockback, _) in current_knockbacks.iter_mut() {
        knockback.duration.tick(time.delta());
    }

    // Separate the enemy knocbacks, and apply them, unclamped.

    let (mut enemy_knockbacks, mut player_knockbacks): (Vec<_>, Vec<_>) = current_knockbacks
        .iter_mut()
        .partition(|(_, _, _, player)| player.is_none());

    for (_, transform, knockback, _) in enemy_knockbacks.iter_mut() {
        transform.translation.x += knockback.direction.x * time.delta_seconds();
        transform.translation.y += knockback.direction.y * time.delta_seconds();
    }

    // Extract the players movement data, and apply the knockbacks, clamped.

    let player_movements = player_knockbacks
        .iter()
        .map(|(_, transform, knockback, _)| {
            (
                transform.translation,
                Some(knockback.direction * time.delta_seconds()),
            )
        })
        .collect::<Vec<_>>();

    let player_dirs = player_movement_clamper.clamp(player_movements);

    for ((_, transform, _, _), player_dir) in player_knockbacks.iter_mut().zip(player_dirs) {
        transform.translation += player_dir.unwrap().extend(0.);
    }
}

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component)]
pub struct MoveInArc {
    pub radius: Vec2,
    pub speed: f32,
    pub angle: f32,
    pub end_angle: f32,
    pub inverse_direction: bool,
    pub origin: Vec2,
}

pub fn move_in_arc_system(
    mut query: Query<(&mut Transform, &mut MoveInArc, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut transform, mut arc, entity) in &mut query.iter_mut() {
        if arc.inverse_direction {
            arc.angle += time.delta_seconds() * arc.speed;

            if arc.angle >= arc.end_angle {
                //TODO: Choose between removing the entity or the component
                // commands.entity(entity).despawn();
                commands.entity(entity).insert(DespawnMarker);
                // commands.entity(entity).remove::<MoveInArc>();
            }
        } else {
            arc.angle -= time.delta_seconds() * arc.speed;
            if arc.angle <= arc.end_angle {
                // commands.entity(entity).despawn();
                commands.entity(entity).insert(DespawnMarker);
                // commands.entity(entity).remove::<MoveInArc>();
            }
        }

        let dir = Vec2::new(
            arc.angle.to_radians().cos(),
            arc.angle.to_radians().sin(),
        )
        // .normalize()
            * arc.radius;

        transform.translation.x = arc.origin.x + dir.x;
        transform.translation.y = arc.origin.y + dir.y;
    }
}

#[derive(Component)]
pub struct Target {
    pub position: Vec2,
}
pub fn move_to_target(
    mut query: Query<(Entity, &mut Transform, &Stats, &Target, &mut Facing)>,
    mut commands: Commands,
    time: Res<Time>,
    mut event_writer: EventWriter<ArrivedEvent>,
) {
    for (entity, mut transform, stats, target, mut state, mut facing) in query.iter_mut() {
        if *state == State::Idle || *state == State::Running {
            let translation_old = transform.translation;
            transform.translation += (target.position.extend(0.) - translation_old).normalize()
                * stats.movement_speed
                * time.delta_seconds();
            if transform.translation.x > translation_old.x {
                *facing = Facing::Right;
            } else {
                *facing = Facing::Left;
            }
            if transform.translation.truncate().distance(target.position) <= 100. {
                commands.entity(entity).remove::<Target>();
                *state = State::Idle;
                event_writer.send(ArrivedEvent(entity))
            } else {
                *state = State::Running;
            }
        }
    }
}
