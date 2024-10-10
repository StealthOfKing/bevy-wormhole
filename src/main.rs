//! Main executable for bevy-wormhole.

use bevy::prelude::*;

mod console;
use console::ConsolePlugin;

mod crt;
use crt::ConsolePostProcessPlugin;
use crt::PostProcessSettings;

mod fo3;
use fo3::Fallout3Plugin;
use fo3::CameraUi;

// load dev console and placeholder fo3 plugin
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((ConsolePlugin, ConsolePostProcessPlugin))
        .add_plugins(Fallout3Plugin)
        .add_systems(PostStartup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    query_camera: Query<Entity, With<CameraUi>>,
) {
    println!("main::setup()");
    // add crt post process effect
    for entity_id in query_camera.iter() {
        println!("  found ui camera");
        commands.entity(entity_id)
            .insert(PostProcessSettings::default());
    }
}
