use crate::player::Player;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_pitch, camera_zoom, follow_player).chain());
    }
}

#[derive(Component)]
pub struct ThirdPersonCamera {
    pub distance: f32,
    pub pitch: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub zoom_speed: f32,
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            distance: 8.0,
            pitch: -0.3,
            min_pitch: -1.4,
            max_pitch: -0.1,
            min_distance: 3.0,
            max_distance: 20.0,
            zoom_speed: 1.0,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ThirdPersonCamera::default(),
    ));
}

fn camera_pitch(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera_q: Query<&mut ThirdPersonCamera>,
) {
    let Ok(mut camera) = camera_q.single_mut() else {
        return;
    };

    let sensitivity = 0.003;
    camera.pitch -= mouse_motion.delta.y * sensitivity;
    camera.pitch = camera.pitch.clamp(camera.min_pitch, camera.max_pitch);
}

fn camera_zoom(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_q: Query<&mut ThirdPersonCamera>,
) {
    let Ok(mut camera) = camera_q.single_mut() else {
        return;
    };

    for event in scroll_events.read() {
        let scroll = match event.unit {
            MouseScrollUnit::Line => event.y * camera.zoom_speed,
            MouseScrollUnit::Pixel => event.y * camera.zoom_speed * 0.01,
        };
        camera.distance -= scroll;
        camera.distance = camera
            .distance
            .clamp(camera.min_distance, camera.max_distance);
    }
}

fn follow_player(
    player_q: Query<(&Transform, &Player)>,
    mut camera_q: Query<(&mut Transform, &ThirdPersonCamera), Without<Player>>,
) {
    let Ok((player_transform, player)) = player_q.single() else {
        return;
    };

    for (mut cam_transform, camera) in camera_q.iter_mut() {
        // Use player's yaw for horizontal rotation, camera's pitch for vertical
        let rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, camera.pitch, 0.0);
        let offset = rotation * Vec3::new(0.0, 0.0, camera.distance);

        cam_transform.translation = player_transform.translation + offset + Vec3::Y * 1.5;
        cam_transform.look_at(player_transform.translation + Vec3::Y * 1.0, Vec3::Y);
    }
}
