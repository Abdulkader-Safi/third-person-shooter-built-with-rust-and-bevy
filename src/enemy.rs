use crate::menu::GameState;
use crate::nav_grid::NavGrid;
use crate::player::{Player, PlayerHealth};
use crate::shooting::{HitEvent, Shootable};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameCounter>()
            .add_systems(Startup, spawn_zombies)
            .add_systems(
                Update,
                (
                    increment_frame_counter,
                    update_zombie_paths,
                    move_zombies,
                    zombie_attack,
                    handle_zombie_hits,
                    update_zombie_health_bars,
                    despawn_dead_zombies,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Frame counter for staggered updates
#[derive(Resource, Default)]
pub struct FrameCounter(pub u32);

/// Zombie enemy component
#[derive(Component)]
pub struct Zombie {
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub damage: f32,
    pub attack_cooldown: Timer,
    pub path_update_offset: u32, // Stagger offset (0-19)
}

impl Zombie {
    pub fn new(path_offset: u32) -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            speed: 3.0,
            damage: 10.0,
            attack_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
            path_update_offset: path_offset % 20,
        }
    }
}

/// Path component for zombie navigation
#[derive(Component, Default)]
pub struct ZombiePath {
    pub waypoints: Vec<Vec3>,
    pub current_index: usize,
}

/// Marker for zombie health bar
#[derive(Component)]
pub struct ZombieHealthBar;

#[derive(Component)]
pub struct ZombieHealthBarFill;

#[derive(Component)]
struct ZombieChildOf(Entity);

fn spawn_zombies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();

    let zombie_mesh = meshes.add(Capsule3d::new(0.4, 1.2));
    let zombie_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.3),
        ..default()
    });

    // Health bar meshes
    let health_bar_bg_mesh = meshes.add(Cuboid::new(0.8, 0.1, 0.05));
    let health_bar_fill_mesh = meshes.add(Cuboid::new(0.75, 0.08, 0.06));
    let health_bar_bg_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),
        unlit: true,
        ..default()
    });
    let health_bar_fill_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.2, 0.2),
        unlit: true,
        ..default()
    });

    // Spawn 40 zombies around the edges of the map
    let zombie_count = 40;

    for i in 0..zombie_count {
        // Spawn zombies at edges of the map
        let (x, z) = match i % 4 {
            0 => (rng.random_range(-45.0..-20.0), rng.random_range(-45.0..45.0)), // West
            1 => (rng.random_range(20.0..45.0), rng.random_range(-45.0..45.0)),   // East
            2 => (rng.random_range(-45.0..45.0), rng.random_range(-45.0..-20.0)), // North
            _ => (rng.random_range(-45.0..45.0), rng.random_range(20.0..45.0)),   // South
        };

        let pos = Vec3::new(x, 1.0, z);
        let path_offset = i as u32; // Distribute offsets evenly

        let zombie_entity = commands
            .spawn((
                Mesh3d(zombie_mesh.clone()),
                MeshMaterial3d(zombie_material.clone()),
                Transform::from_translation(pos),
                Zombie::new(path_offset),
                ZombiePath::default(),
                Shootable,
                RigidBody::KinematicPositionBased,
                Collider::capsule_y(0.6, 0.4),
                KinematicCharacterController {
                    filter_flags: QueryFilterFlags::EXCLUDE_KINEMATIC,
                    ..default()
                },
            ))
            .id();

        // Health bar background
        commands.spawn((
            Mesh3d(health_bar_bg_mesh.clone()),
            MeshMaterial3d(health_bar_bg_material.clone()),
            Transform::from_translation(pos + Vec3::Y * 1.5),
            ZombieHealthBar,
            ZombieChildOf(zombie_entity),
        ));

        // Health bar fill
        commands.spawn((
            Mesh3d(health_bar_fill_mesh.clone()),
            MeshMaterial3d(health_bar_fill_material.clone()),
            Transform::from_translation(pos + Vec3::Y * 1.5),
            ZombieHealthBar,
            ZombieHealthBarFill,
            ZombieChildOf(zombie_entity),
        ));
    }
}

fn increment_frame_counter(mut counter: ResMut<FrameCounter>) {
    counter.0 = counter.0.wrapping_add(1);
}

fn update_zombie_paths(
    frame: Res<FrameCounter>,
    nav_grid: Res<NavGrid>,
    player_query: Query<&Transform, With<Player>>,
    mut zombies: Query<(&Transform, &Zombie, &mut ZombiePath)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let current_frame = frame.0 % 20;

    for (transform, zombie, mut path) in zombies.iter_mut() {
        // Only update if this zombie's offset matches current frame
        if zombie.path_update_offset != current_frame {
            continue;
        }

        // Find path to player
        if let Some(new_path) = nav_grid.find_path(transform.translation, player_pos) {
            path.waypoints = new_path;
            path.current_index = 0;
        }
    }
}

fn move_zombies(
    time: Res<Time>,
    mut zombies: Query<(
        &mut Transform,
        &Zombie,
        &mut ZombiePath,
        &mut KinematicCharacterController,
    )>,
) {
    for (mut transform, zombie, mut path, mut controller) in zombies.iter_mut() {
        if path.waypoints.is_empty() || path.current_index >= path.waypoints.len() {
            controller.translation = Some(Vec3::ZERO);
            continue;
        }

        let target = path.waypoints[path.current_index];
        let current_pos = transform.translation;
        let direction = (target - current_pos).with_y(0.0);
        let distance = direction.length();

        // If close enough to waypoint, move to next one
        if distance < 0.5 {
            path.current_index += 1;
            continue;
        }

        // Move towards waypoint
        let move_dir = direction.normalize_or_zero();
        let movement = move_dir * zombie.speed * time.delta_secs();

        controller.translation = Some(movement);

        // Rotate to face movement direction
        if move_dir.length_squared() > 0.001 {
            let target_rotation = Quat::from_rotation_y((-move_dir.x).atan2(-move_dir.z));
            transform.rotation = transform.rotation.slerp(target_rotation, 5.0 * time.delta_secs());
        }
    }
}

fn zombie_attack(
    time: Res<Time>,
    mut zombies: Query<(&Transform, &mut Zombie)>,
    mut player_query: Query<(&Transform, &mut PlayerHealth), With<Player>>,
) {
    let Ok((player_transform, mut player_health)) = player_query.single_mut() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (zombie_transform, mut zombie) in zombies.iter_mut() {
        zombie.attack_cooldown.tick(time.delta());

        let distance = (zombie_transform.translation - player_pos).with_y(0.0).length();

        // Attack if close enough and cooldown finished
        if distance < 1.5 && zombie.attack_cooldown.is_finished() {
            player_health.current -= zombie.damage;
            player_health.current = player_health.current.max(0.0);
            zombie.attack_cooldown.reset();
        }
    }
}

fn handle_zombie_hits(
    mut hit_events: MessageReader<HitEvent>,
    mut zombies: Query<&mut Zombie>,
) {
    for event in hit_events.read() {
        if let Ok(mut zombie) = zombies.get_mut(event.entity) {
            zombie.health -= event.damage;
            zombie.health = zombie.health.max(0.0);
        }
    }
}

fn update_zombie_health_bars(
    zombies: Query<(Entity, &Transform, &Zombie)>,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut health_bars: Query<
        (&mut Transform, &ZombieChildOf, Option<&ZombieHealthBarFill>),
        (With<ZombieHealthBar>, Without<Zombie>, Without<Camera3d>),
    >,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    for (mut bar_transform, child_of, is_fill) in health_bars.iter_mut() {
        if let Ok((_, zombie_transform, zombie)) = zombies.get(child_of.0) {
            // Position above zombie
            bar_transform.translation = zombie_transform.translation + Vec3::Y * 1.8;

            // Billboard effect
            let look_dir = camera_transform.translation - bar_transform.translation;
            if look_dir.length_squared() > 0.001 {
                bar_transform.look_to(-look_dir, Vec3::Y);
            }

            // Scale fill bar based on health
            if is_fill.is_some() {
                let health_percent = zombie.health / zombie.max_health;
                bar_transform.scale.x = health_percent.max(0.01);
            }
        }
    }
}

fn despawn_dead_zombies(
    mut commands: Commands,
    zombies: Query<(Entity, &Zombie)>,
    health_bars: Query<(Entity, &ZombieChildOf), With<ZombieHealthBar>>,
) {
    for (entity, zombie) in zombies.iter() {
        if zombie.health <= 0.0 {
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
