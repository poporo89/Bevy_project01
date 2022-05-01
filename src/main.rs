use bevy::{audio::AudioSink, prelude::*};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use pyo3::{prelude::*, types::PyList};
use std::env;

#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Tile;

#[derive(Component)]
struct Floor;

#[derive(Component)]
struct Size {
    width: u32,
    depth: u32,
}

#[derive(Bundle)]
struct FloorBundle {
    floor: Floor,
    size: Size,
    position: Position,
}

#[derive(Component, Inspectable)]
struct Speed(f32);

#[derive(Component)]
struct MovableCamera;

struct MusicController(Handle<AudioSink>);

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Speed>()
        .add_startup_system(setup)
        .add_startup_system(setup_audio)
        .add_startup_system_to_stage(StartupStage::Startup, spawn_floor)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_tiles)
        .add_system(pause)
        .add_system(move_camera)
        .add_system(call_python)
        .run();
}

// use Python
fn call_python(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::P) {
        match test_python() {
            Ok(()) => println!("Python works."),
            Err(e) => println!("error parsing header: {:?}", e),
        }
    }
}

// Python test
fn test_python() -> PyResult<()> {
    Python::with_gil(|py| {
        let sys = py.import("sys")?;
        // get a list of paths where Python modules may exist
        let syspath: &PyList = sys.getattr("path")?.extract()?;
        // create a path to add the list
        let mut path = env::current_dir()?;
        path.push("scripts");

        // add the path (unwrap because it returns Result)
        syspath.insert(0, format!("{}", path.display())).unwrap();

        let map = py.import("map")?;
        map.call_method0("test_map")?;

        Ok(())
    })
}

// move camera
fn move_camera(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &Speed), With<MovableCamera>>,
    timer: Res<Time>,
) {
    let (mut transform, speed) = query.single_mut();

    if keyboard_input.pressed(KeyCode::J) {
        transform.translation -=
            (Vec3::X + Vec3::Z) / 2.0_f32.sqrt() * speed.0 * timer.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::K) {
        transform.translation +=
            (Vec3::X + Vec3::Z) / 2.0_f32.sqrt() * speed.0 * timer.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::H) {
        transform.translation +=
            (Vec3::X - Vec3::Z) / 2.0_f32.sqrt() * speed.0 * timer.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::L) {
        transform.translation -=
            (Vec3::X - Vec3::Z) / 2.0_f32.sqrt() * speed.0 * timer.delta_seconds();
    }
}

// BGM
fn setup_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    let music = asset_server.load("sounds/Lady_Maria.ogg");
    // play audio and upgrade to a strong handle
    let handle = audio_sinks.get_handle(audio.play(music));
    commands.insert_resource(MusicController(handle));
}

fn pause(
    keyboard_input: Res<Input<KeyCode>>,
    audio_sinks: Res<Assets<AudioSink>>,
    music_controller: Res<MusicController>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Some(sink) = audio_sinks.get(&music_controller.0) {
            if sink.is_paused() {
                sink.play()
            } else {
                sink.pause()
            }
        }
    }
}

// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // set up the camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 10.0;
    camera.transform = Transform::from_xyz(-10.0, 10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y);

    // camera
    commands
        .spawn_bundle(camera)
        .insert(Speed(15.0))
        .insert(MovableCamera);

    // light
    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_xyz(-3.0, 8.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 6000.0,
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
}

fn spawn_floor(mut commands: Commands) {
    commands.spawn_bundle(FloorBundle {
        floor: Floor,
        size: Size { width: 5, depth: 4 },
        position: Position(Vec3::new(0.0, 0.0, 0.0)),
    });
}

fn spawn_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Size, &Position), With<Floor>>,
) {
    let mesh = Mesh::from(shape::Cube { size: 1.0 });
    let material = StandardMaterial::from(Color::rgb(230. / 255., 230. / 255., 230. / 255.));

    for (size, position) in query.iter() {
        let y = position.0.y;
        for i in 0..size.width {
            for j in 0..size.depth {
                let x = (i as f32) - 0.5 * (size.width as f32) + 0.5;
                let z = (j as f32) - 0.5 * (size.depth as f32) + 0.5;
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(mesh.clone()),
                        material: materials.add(material.clone()),
                        transform: Transform::from_translation(Vec3::new(x, y, z)),
                        ..default()
                    })
                    .insert(Tile);
            }
        }
    }
}
