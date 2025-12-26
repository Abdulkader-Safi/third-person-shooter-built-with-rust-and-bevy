use bevy::prelude::*;

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
            DefaultPlugins,
            MenuPlugin,
            PlayerPlugin,
            CameraPlugin,
            WorldPlugin,
            TargetPlugin,
        ))
        .run();
}
