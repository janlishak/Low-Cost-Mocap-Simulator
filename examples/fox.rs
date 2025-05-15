use bevy::prelude::*;
use bevy_capture::animation::{keyboard_animation_control, setup_animation, setup_scene_once_loaded};

fn main() {
    App::new()
        .insert_resource(AmbientLight { color: Color::WHITE, brightness: 100., affects_lightmapped_meshes: false, })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_animation)
        .add_systems(Update, setup_scene_once_loaded)
        .add_systems(Update, (keyboard_animation_control))
        .run();
}