use crate::menu::GameState;
use crate::player::Player;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct ShootingPlugin;

impl Plugin for ShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HitEvent>()
            .add_systems(Update, shoot.run_if(in_state(GameState::Playing)))
            .add_systems(Update, update_debug_rays);
    }
}

/// Add this component to any entity that can be shot
#[derive(Component)]
pub struct Shootable;

/// Add this component to any entity that can shoot (player, turret, enemy, etc.)
#[derive(Component)]
pub struct Weapon {
    pub damage: f32,
    pub fire_rate: f32, // Shots per second (for future use)
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            damage: 25.0,
            fire_rate: 2.0,
        }
    }
}

impl Weapon {
    pub fn new(damage: f32) -> Self {
        Self {
            damage,
            ..default()
        }
    }

    pub fn with_fire_rate(mut self, fire_rate: f32) -> Self {
        self.fire_rate = fire_rate;
        self
    }
}

/// Event sent when something is shot - each system can listen and handle its own logic
#[derive(Message)]
pub struct HitEvent {
    pub entity: Entity,
    pub damage: f32,
}

#[derive(Component)]
pub struct DebugRay {
    pub timer: Timer,
}

fn shoot(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    player_q: Query<(Entity, &Transform, &Weapon), With<Player>>,
    rapier_context: ReadRapierContext,
    shootables: Query<Entity, With<Shootable>>,
    mut hit_events: MessageWriter<HitEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((player_entity, player_transform, weapon)) = player_q.single() else {
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

    // Check if we hit a shootable entity and send event
    if let Some((entity, _)) = hit_entity {
        if shootables.get(entity).is_ok() {
            hit_events.write(HitEvent {
                entity,
                damage: weapon.damage,
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
