use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier3d::prelude::*;

mod combat;
mod enemies;
mod player;
mod ui;
mod world;

use combat::{ShootingPlugin, WeaponUiPlugin};
use enemies::{EnemyPlugin, TargetPlugin};
use player::{CameraPlugin, PlayerPlugin};
use ui::MenuPlugin;
use world::{NavGridPlugin, WorldPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "My Bevy Game".into(),
                        resolution: WindowResolution::new(1920, 1080),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            NavGridPlugin,
            MenuPlugin,
            PlayerPlugin,
            CameraPlugin,
            WorldPlugin,
            ShootingPlugin,
            TargetPlugin,
            EnemyPlugin,
            WeaponUiPlugin,
        ))
        .run();
}
