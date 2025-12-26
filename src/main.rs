use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier3d::prelude::*;

mod camera;
mod menu;
mod player;
mod target;
mod world;

use camera::CameraPlugin;
use menu::MenuPlugin;
use player::PlayerPlugin;
use target::TargetPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "My Bevy Game".into(),
                    resolution: WindowResolution::new(1920, 1080),
                    ..default()
                }),
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            MenuPlugin,
            PlayerPlugin,
            CameraPlugin,
            WorldPlugin,
            TargetPlugin,
        ))
        .run();
}
