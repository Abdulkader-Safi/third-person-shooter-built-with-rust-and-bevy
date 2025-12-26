use crate::menu::GameState;
use crate::player::Player;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HitMessage>()
            .add_systems(Startup, spawn_targets)
            .add_systems(Update, shoot.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                (
                    handle_hits,
                    update_health_bars,
                    update_hit_flash,
                    despawn_dead_targets,
                    billboard_health_bars,
                    update_debug_rays,
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

#[derive(Component)]
pub struct DebugRay {
    pub timer: Timer,
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
                // Rapier physics components
                RigidBody::Fixed,
                Collider::cuboid(0.75, 1.0, 0.75),
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
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    player_q: Query<(Entity, &Transform), With<Player>>,
    rapier_context: ReadRapierContext,
    targets: Query<Entity, With<Target>>,
    mut hit_messages: MessageWriter<HitMessage>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((player_entity, player_transform)) = player_q.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let player_forward = *player_transform.forward();

    // Shoot from player position in player's forward direction
    let ray_origin = player_pos + Vec3::Y * 0.5;
    let ray_direction = player_forward;
    let max_distance = 100.0;

    // Use rapier's raycasting
    let Ok(context) = rapier_context.single() else {
        return;
    };

    // Exclude player from raycast
    let filter = QueryFilter::default().exclude_rigid_body(player_entity);

    let mut hit_entity: Option<(Entity, f32)> = None;
    context.with_query_pipeline(filter, |query_pipeline| {
        hit_entity = query_pipeline.cast_ray(ray_origin, ray_direction, max_distance, true);
    });

    // Determine ray end point for debug visualization
    let ray_end = if let Some((_, distance)) = hit_entity {
        ray_origin + ray_direction * distance
    } else {
        ray_origin + ray_direction * max_distance
    };

    // Spawn debug ray visualization
    let ray_length = (ray_end - ray_origin).length();
    let ray_center = (ray_origin + ray_end) / 2.0;
    let ray_rotation = Quat::from_rotation_arc(Vec3::Y, ray_direction);

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.02, ray_length, 0.02))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.0),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(ray_center).with_rotation(ray_rotation),
        DebugRay {
            timer: Timer::from_seconds(1.5, TimerMode::Once),
        },
    ));

    // Check if we hit a target
    if let Some((entity, _)) = hit_entity {
        if targets.get(entity).is_ok() {
            hit_messages.write(HitMessage {
                target: entity,
                damage: 25.0,
            });
        }
    }
}

fn update_debug_rays(
    mut commands: Commands,
    time: Res<Time>,
    mut rays: Query<(Entity, &mut DebugRay)>,
) {
    for (entity, mut ray) in rays.iter_mut() {
        ray.timer.tick(time.delta());
        if ray.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
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
