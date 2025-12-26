use crate::player::Player;
use bevy::prelude::*;

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HitMessage>()
            .add_systems(Startup, spawn_targets)
            .add_systems(
                Update,
                (
                    shoot,
                    handle_hits,
                    update_health_bars,
                    update_hit_flash,
                    despawn_dead_targets,
                    billboard_health_bars,
                ),
            );
    }
}

#[derive(Component)]
pub struct Target {
    pub max_health: f32,
    pub current_health: f32,
}

impl Target {
    pub fn new(health: f32) -> Self {
        Self {
            max_health: health,
            current_health: health,
        }
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarBackground;

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct HitFlash {
    pub timer: Timer,
    pub original_color: Color,
}

#[derive(Message)]
pub struct HitMessage {
    pub target: Entity,
    pub damage: f32,
}

fn spawn_targets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let target_positions = [
        Vec3::new(5.0, 1.0, 0.0),
        Vec3::new(-5.0, 1.0, 3.0),
        Vec3::new(0.0, 1.0, -6.0),
        Vec3::new(3.0, 1.0, 5.0),
        Vec3::new(-4.0, 1.0, -4.0),
    ];

    let target_mesh = meshes.add(Cuboid::new(1.5, 2.0, 1.5));

    let health_bar_bg_mesh = meshes.add(Cuboid::new(1.2, 0.15, 0.05));
    let health_bar_fill_mesh = meshes.add(Cuboid::new(1.1, 0.1, 0.06));
    let health_bar_bg_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),
        unlit: true,
        ..default()
    });
    let health_bar_fill_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.8, 0.1),
        unlit: true,
        ..default()
    });

    for pos in target_positions {
        // Create a unique material for each target so they can flash independently
        let target_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        });

        let target_entity = commands
            .spawn((
                Mesh3d(target_mesh.clone()),
                MeshMaterial3d(target_material),
                Transform::from_translation(pos),
                Target::new(100.0),
            ))
            .id();

        // Health bar background
        commands.spawn((
            Mesh3d(health_bar_bg_mesh.clone()),
            MeshMaterial3d(health_bar_bg_material.clone()),
            Transform::from_translation(pos + Vec3::Y * 1.5),
            HealthBar,
            HealthBarBackground,
            ChildOf(target_entity),
        ));

        // Health bar fill
        commands.spawn((
            Mesh3d(health_bar_fill_mesh.clone()),
            MeshMaterial3d(health_bar_fill_material.clone()),
            Transform::from_translation(pos + Vec3::Y * 1.5),
            HealthBar,
            HealthBarFill,
            ChildOf(target_entity),
        ));
    }
}

#[derive(Component)]
struct ChildOf(Entity);

fn shoot(
    mouse_button: Res<ButtonInput<MouseButton>>,
    player_q: Query<&Transform, With<Player>>,
    targets: Query<(Entity, &Transform), With<Target>>,
    mut hit_messages: MessageWriter<HitMessage>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(player_transform) = player_q.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let player_forward = *player_transform.forward();

    // Shoot from player position in player's forward direction
    let ray_origin = player_pos + Vec3::Y * 0.5; // Shoot from player center
    let ray_direction = player_forward;

    // Simple ray-box intersection for each target, but only if in front of player
    for (entity, transform) in targets.iter() {
        // Check if target is in front of player (dot product > 0)
        let to_target = (transform.translation - player_pos).normalize_or_zero();
        let dot = player_forward.dot(to_target);

        if dot < 0.3 {
            // Target is not in front of player (need at least ~70 degree cone)
            continue;
        }

        let half_size = Vec3::new(0.75, 1.0, 0.75);
        let min = transform.translation - half_size;
        let max = transform.translation + half_size;

        if ray_box_intersection(ray_origin, ray_direction, min, max) {
            hit_messages.write(HitMessage {
                target: entity,
                damage: 25.0,
            });
            break; // Only hit one target per shot
        }
    }
}

fn ray_box_intersection(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3) -> bool {
    let inv_dir = Vec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z);

    let t1 = (min.x - origin.x) * inv_dir.x;
    let t2 = (max.x - origin.x) * inv_dir.x;
    let t3 = (min.y - origin.y) * inv_dir.y;
    let t4 = (max.y - origin.y) * inv_dir.y;
    let t5 = (min.z - origin.z) * inv_dir.z;
    let t6 = (max.z - origin.z) * inv_dir.z;

    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    tmax >= 0.0 && tmin <= tmax
}

fn handle_hits(
    mut commands: Commands,
    mut hit_messages: MessageReader<HitMessage>,
    mut targets: Query<(&mut Target, &MeshMaterial3d<StandardMaterial>)>,
    materials: Res<Assets<StandardMaterial>>,
) {
    for message in hit_messages.read() {
        if let Ok((mut target, material_handle)) = targets.get_mut(message.target) {
            target.current_health -= message.damage;
            target.current_health = target.current_health.max(0.0);

            // Get original color and add flash component
            let original_color = materials
                .get(&material_handle.0)
                .map(|m| m.base_color)
                .unwrap_or(Color::srgb(0.8, 0.2, 0.2));

            commands.entity(message.target).insert(HitFlash {
                timer: Timer::from_seconds(0.1, TimerMode::Once),
                original_color,
            });
        }
    }
}

fn update_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut HitFlash, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut flash, material_handle) in flash_query.iter_mut() {
        flash.timer.tick(time.delta());

        if let Some(material) = materials.get_mut(&material_handle.0) {
            if flash.timer.is_finished() {
                material.base_color = flash.original_color;
                commands.entity(entity).remove::<HitFlash>();
            } else {
                // Flash white
                material.base_color = Color::WHITE;
            }
        }
    }
}

fn update_health_bars(
    targets: Query<&Target>,
    mut health_bar_fills: Query<(&mut Transform, &ChildOf), With<HealthBarFill>>,
) {
    for (mut bar_transform, child_of) in health_bar_fills.iter_mut() {
        if let Ok(target) = targets.get(child_of.0) {
            let health_percent = target.current_health / target.max_health;
            bar_transform.scale.x = health_percent.max(0.01);
        }
    }
}

fn billboard_health_bars(
    camera_q: Query<&Transform, With<Camera3d>>,
    targets: Query<(Entity, &Transform), With<Target>>,
    mut health_bars: Query<
        (&mut Transform, &ChildOf),
        (With<HealthBar>, Without<Target>, Without<Camera3d>),
    >,
) {
    let Ok(camera_transform) = camera_q.single() else {
        return;
    };

    for (mut bar_transform, child_of) in health_bars.iter_mut() {
        if let Ok((_, target_transform)) = targets.get(child_of.0) {
            // Position above target
            bar_transform.translation = target_transform.translation + Vec3::Y * 1.5;

            // Face camera (billboard effect)
            let look_dir = camera_transform.translation - bar_transform.translation;
            if look_dir.length_squared() > 0.001 {
                bar_transform.look_to(-look_dir, Vec3::Y);
            }
        }
    }
}

fn despawn_dead_targets(
    mut commands: Commands,
    targets: Query<(Entity, &Target)>,
    health_bars: Query<(Entity, &ChildOf), With<HealthBar>>,
) {
    for (entity, target) in targets.iter() {
        if target.current_health <= 0.0 {
            // Despawn health bars first
            for (bar_entity, child_of) in health_bars.iter() {
                if child_of.0 == entity {
                    commands.entity(bar_entity).despawn();
                }
            }
            commands.entity(entity).despawn();
        }
    }
}
