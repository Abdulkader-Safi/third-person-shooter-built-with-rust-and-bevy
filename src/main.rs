use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier3d::prelude::*;

mod camera;
mod enemy;
mod menu;
mod nav_grid;
mod player;
mod shooting;
mod target;
mod weapon_ui;
mod world;

use camera::CameraPlugin;
use enemy::EnemyPlugin;
use menu::MenuPlugin;
use nav_grid::NavGridPlugin;
use player::PlayerPlugin;
use shooting::ShootingPlugin;
use target::TargetPlugin;
use weapon_ui::WeaponUiPlugin;
use world::WorldPlugin;

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
