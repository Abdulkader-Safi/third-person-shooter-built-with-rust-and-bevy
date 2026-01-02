use crate::menu::GameState;
use crate::shooting::{ShootCooldown, WeaponInventory};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (player_rotation, player_movement)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Player {
    pub yaw: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self { yaw: 0.0 }
    }
}

#[derive(Component)]
struct Speed {
    value: f32,
}

fn player_rotation(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut player_q: Query<(&mut Transform, &mut Player)>,
) {
    let sensitivity = 0.003;

    for (mut transform, mut player) in player_q.iter_mut() {
        player.yaw -= mouse_motion.delta.x * sensitivity;
        transform.rotation = Quat::from_rotation_y(player.yaw);
    }
}

fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_q: Query<(&Transform, &Speed, &mut KinematicCharacterController), With<Player>>,
) {
    for (player_transform, player_speed, mut controller) in player_q.iter_mut() {
        let forward = player_transform.forward();
        let right = player_transform.right();

        let mut direction = Vec3::ZERO;

        if keys.pressed(KeyCode::KeyW) {
            direction += *forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction -= *forward;
        }
        if keys.pressed(KeyCode::KeyD) {
            direction += *right;
        }
        if keys.pressed(KeyCode::KeyA) {
            direction -= *right;
        }

        direction.y = 0.0;
        let movement = direction.normalize_or_zero() * player_speed.value * time.delta_secs();
        controller.translation = Some(movement);
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Speed { value: 5.0 },
        Player::default(),
        WeaponInventory::default(),
        ShootCooldown::default(),
        // Physics components
        RigidBody::KinematicPositionBased,
        Collider::cuboid(0.5, 0.5, 0.5),
        KinematicCharacterController::default(),
    ));
}
