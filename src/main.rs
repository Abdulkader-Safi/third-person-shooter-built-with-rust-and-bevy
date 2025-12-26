use bevy::prelude::*;

mod camera;
mod player;
mod target;
mod world;

use camera::CameraPlugin;
use player::PlayerPlugin;
use target::TargetPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PlayerPlugin,
            CameraPlugin,
            WorldPlugin,
            TargetPlugin,
        ))
        .run();
}
