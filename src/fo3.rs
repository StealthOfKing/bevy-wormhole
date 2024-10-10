//! Placeholder for Fallout 3.

use bevy::prelude::*;

pub struct Fallout3Plugin;

impl Plugin for Fallout3Plugin {
    fn build(&self, app: &mut App) {
        println!("Fallout3Plugin::build()");
        app.add_systems(Startup, setup);
        app.add_systems(Update, rotate);
    }
}

//------------------------------------------------------------------------------

#[derive(Component)]
pub struct CameraUi;

/// Set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("Fallout3Plugin::setup()");
    // 3D camera is used to render 3D scenes.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 5.0)
                .looking_at(Vec3::default(), Vec3::Y),
            camera: Camera {
                order: 0,
                clear_color: Color::srgb(0.65, 0.65, 0.65).into(),
                ..default()
            },
            ..default()
        },
    ));

    // 2D camera is used for presentation.
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1, // the order is important
                ..default()
            },
            ..default()
        },
        CameraUi,
        // Apply CRT shader to everything (UI + scene).
//        PostProcessSettings {
//            intensity: 0.02,
//            ..default()
//        },
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Rotates,
    ));

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1_000.,
            ..default()
        },
        ..default()
    });
}

#[derive(Component)]
struct Rotates;

/// Rotates any entity around the x and y axis
fn rotate(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    for mut transform in &mut query {
        transform.rotate_x(0.55 * time.delta_seconds());
        transform.rotate_z(0.15 * time.delta_seconds());
    }
}
