use bevy::{audio::AudioSink, prelude::*};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use std::env;

#[derive(Component)]
struct Tile;

#[derive(Bundle)]
struct LevelBundle {
    level: Level,
    map: Map,
    visible: Visible,
}

#[derive(Component, PartialEq, Eq)]
enum Level {
    TestMap,
}

impl Level {
    // bind lebels to Python method names
    fn label(&self) -> &str {
        match *self {
            Level::TestMap => "test_map",
        }
    }
}

#[derive(Component, Default, Debug)]
struct Map {
    floors: Vec<Floor>,
    position: Vec3,
}

impl Map {
    fn width(&self) -> u32 {
        let first_floor = self.floors.first().unwrap();
        first_floor.data[0].len().try_into().unwrap()
    }

    fn depth(&self) -> u32 {
        let first_floor = self.floors.first().unwrap();
        first_floor.data.len().try_into().unwrap()
    }
}

#[derive(Debug)]
struct Floor {
    height: i32,
    data: Vec<Vec<i32>>,
}

#[derive(Component, Default)]
struct Visible(bool);

#[derive(Bundle)]
struct FloorBundle {
    floor: FloorLegacy,
    size: Size,
    position: Position,
}

#[derive(Component)]
struct FloorLegacy;

#[derive(Component)]
struct Size {
    width: u32,
    depth: u32,
}

#[derive(Component, Default)]
struct Position(Vec3);

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
        .add_startup_system(setup_levels)
        .add_startup_system_to_stage(StartupStage::Startup, spawn_floor)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_tiles)
        .add_system(pause)
        .add_system(move_camera)
        .add_system(manual_load_map)
        .run();
}

// setup levels with empty maps
fn setup_levels(mut commands: Commands) {
    // test map
    commands.spawn_bundle(LevelBundle {
        level: Level::TestMap,
        map: Map {
            floors: Vec::new(),
            position: Vec3::ZERO,
        },
        visible: Visible(false),
    });
}

// load map at manual event
fn manual_load_map(keyboard_input: Res<Input<KeyCode>>, query: Query<(&Level, &mut Map)>) {
    // manual event to load map
    if keyboard_input.just_pressed(KeyCode::P) {
        let level_to_load = Level::TestMap;
        load_map(&level_to_load, query);
    }
}

fn load_map(level_to_load: &Level, mut query: Query<(&Level, &mut Map)>) {
    // if the map is already loaded, quit the system
    for (level, map) in query.iter() {
        if level == level_to_load && !map.floors.is_empty() {
            return;
        }
    }

    for (level, mut map) in query.iter_mut() {
        if level == level_to_load {
            // get data through arguments
            let mut floor_data = Vec::new();
            let mut position_data = Vec3::ZERO;
            match parse_map_from_python(&mut floor_data, &mut position_data, level) {
                Ok(_) => println!("Python works."),
                Err(e) => println!("error parsing header: {:?}", e),
            };

            // store map data to the map component
            map.floors = floor_data;
            map.position = position_data;
            println!("{}", format!("{:?}", map));
        }
    }
}

// TODO:: restructure arguments & refactoring
fn parse_map_from_python(
    floors: &mut Vec<Floor>,
    position: &mut Vec3,
    level: &Level,
) -> PyResult<()> {
    // load map data from a Python module
    Python::with_gil(|py| {
        let sys_module = py.import("sys")?;
        // get a list of paths where Python modules may exist
        let path_list: &PyList = sys_module.getattr("path")?.extract()?;
        // create a path to add the list
        let mut map_path = env::current_dir()?;
        map_path.push("scripts");

        // add the path (unwrap because it returns Result)
        path_list
            .insert(0, format!("{}", map_path.display()))
            .unwrap();

        let map_module = py.import("map")?;
        let py_map: &PyDict = map_module.call_method0(level.label())?.extract()?;

        // parse floors
        let py_floors = py_map.get_item("floors").unwrap();
        for py_floor in py_floors.iter().unwrap() {
            let mut data = Vec::new();
            let py_data = py_floor.as_ref().unwrap().get_item("data").unwrap();
            // push each rows as Vec<i32>
            for py_row in py_data.iter().unwrap() {
                let row: Vec<i32> = py_row.unwrap().extract::<Vec<i32>>().unwrap();
                data.push(row);
            }
            let height = py_floor
                .as_ref()
                .unwrap()
                .get_item("height")
                .unwrap()
                .extract::<i32>()
                .unwrap();
            floors.push(Floor { data, height });
        }

        // parse position
        let py_position = py_map.get_item("position").unwrap();
        let x = py_position.get_item(0).unwrap().extract::<f32>().unwrap();
        let y = py_position.get_item(1).unwrap().extract::<f32>().unwrap();
        let z = py_position.get_item(2).unwrap().extract::<f32>().unwrap();
        *position = Vec3::new(x, y, z);

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
        floor: FloorLegacy,
        size: Size { width: 5, depth: 4 },
        position: Position(Vec3::new(0.0, 0.0, 0.0)),
    });
}

fn spawn_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Size, &Position), With<FloorLegacy>>,
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
