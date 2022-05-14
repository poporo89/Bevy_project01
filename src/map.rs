use bevy::prelude::*;
use bevy_rhai::*;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use rhai::plugin::*;
use std::env;

#[derive(Bundle)]
struct LevelBundle {
    level: Level,
    map: Map,
    position: Position,
    visible: Visible,
}

#[derive(Component, PartialEq, Eq, Clone)]
enum Level {
    TestMap,
}

impl Level {
    // bind lebels to Python method names
    fn name(&self) -> &str {
        match *self {
            Level::TestMap => "test_map",
        }
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct Map {
    pub floors: Vec<Floor>,
}

impl Map {
    fn width(&self) -> usize {
        let first_floor = self.floors.first().unwrap();
        first_floor.data[0].len()
    }

    fn depth(&self) -> usize {
        let first_floor = self.floors.first().unwrap();
        first_floor.data.len()
    }

    fn is_loaded(&self) -> bool {
        !self.floors.is_empty()
    }

    pub fn push(&mut self, floor: &Floor) {
        self.floors.push(floor.clone());
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct Floor {
    height: i32,
    data: Vec<Vec<i32>>,
}

impl Floor {
    pub fn new() -> Self {
        Self {
            height: 0,
            data: Vec::new(),
        }
    }

    pub fn set_height(&mut self, height: i32) {
        self.height = height;
    }

    pub fn set_data(&mut self, data: Dynamic) {
        let result = data.try_cast::<Vec<i32>>();
        if let Some(data) = result {
            self.data.push(data);
        } else {
            println!("no data");
        }
    }
}

#[derive(Component, Default, Clone)]
pub struct Position(Vec3);

impl Position {
    pub fn set_position(&mut self, position: Dynamic) {
        let array: [f32; 3] = position.try_cast::<[f32; 3]>().unwrap();
        self.0 = Vec3::from(array);
    }
}

#[derive(Component, Default)]
struct Visible(bool);

#[derive(Component)]
struct Tile;

#[derive(Bundle, Default)]
struct RhaiBundle {
    engine: StandardEngine,
    script_handle: Handle<StandardScript>,
    scope: StandardScope,
}

//#[export_module]
//mod map_editor_api {
//pub fn
//}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_levels)
            .add_system(manual_load_map)
            .add_system(manual_unload_map)
            .add_system(manual_spawn_map)
            .add_system(manual_despawn_map);
    }
}

// setup levels with empty maps
fn setup_levels(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<StandardScript> = asset_server.load("scripts/map_editor.rhai");
    asset_server.watch_for_changes().unwrap();
    // test map
    commands
        .spawn_bundle(LevelBundle {
            level: Level::TestMap,
            map: Map { floors: Vec::new() },
            position: Position(Vec3::ZERO),
            visible: Visible(false),
        })
        .insert_bundle(RhaiBundle {
            engine: StandardEngine::with_engine({
                let mut engine = Engine::new_raw();
                engine.set_strict_variables(true);
                engine.disable_symbol("eval")
                .register_type_with_name::<Map>("Map")
                .register_fn("push", Map::push)
                .register_type_with_name::<Floor>("Floor")
                .register_fn("floor_new", Floor::new)
                .register_set("height", Floor::set_height)
                .register_set("data", Floor::set_data)
                //.register_type_with_name::<Position>("Position")
                //.register_set("xyz", Position::set_position)
                //.register_global_module(exported_module!(map_editor_api).into())
                ;
                engine
            }),
            script_handle: handle,
            scope: StandardScope::default(),
        })
        .insert(Floor::default());
}

// load map at manual event
#[allow(dead_code)]
fn manual_load_map_py(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(&Level, &mut Map, &mut Position)>,
) {
    let level_to_load = Level::TestMap;
    // manual event to load map
    if keyboard_input.just_pressed(KeyCode::Q) {
        load_map_py(&level_to_load, query);
    }
}

#[allow(clippy::complexity)]
fn manual_load_map(
    keyboard_input: Res<Input<KeyCode>>,
    scripts: Res<Assets<StandardScript>>,
    query: Query<(
        &mut Map,
        &StandardEngine,
        &Handle<StandardScript>,
        &mut StandardScope,
    )>,
) {
    if keyboard_input.just_pressed(KeyCode::P) {
        let level_to_load = Level::TestMap;
        load_map(&level_to_load, scripts, query);
        println!("po");
    }
}

#[allow(clippy::complexity)]
fn load_map(
    _level_to_load: &Level,
    scripts: Res<Assets<StandardScript>>,
    mut query: Query<(
        &mut Map,
        &StandardEngine,
        &Handle<StandardScript>,
        &mut StandardScope,
    )>,
) {
    for (mut map, engine, script, mut scope) in query.iter_mut() {
        if let Some(script) = scripts.get(script) {
            println!("popopo");
            let mut floor = Floor::new();
            let result: Vec<Dynamic> = engine
                .call_fn(&mut scope, &script.ast, "array_test", ())
                .unwrap();
            floor.data = result
                .into_iter()
                .map(|item| item.into_typed_array::<i32>().unwrap())
                .collect();
            map.push(&floor);
            println!("{:?}", &map);
        }
    }
}

fn manual_unload_map(
    keyboard_input: Res<Input<KeyCode>>,
    query: Query<(&Level, &mut Map, &mut Position)>,
) {
    if keyboard_input.just_pressed(KeyCode::D) {
        let level_to_unload = Level::TestMap;
        unload_map(&level_to_unload, query);
    }
}

fn manual_spawn_map(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Level, &Map, &Position, &mut Visible)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // manual event to spawn map
    if keyboard_input.pressed(KeyCode::P) {
        spawn_map(commands, meshes, materials, query);
    }
}

fn manual_despawn_map(
    commands: Commands,
    query: Query<(Entity, &Level, Option<&mut Visible>, Option<&Tile>)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::D) {
        let level_to_despawn = Level::TestMap;
        despawn_map(commands, query, &level_to_despawn);
    }
}

#[allow(dead_code)]
fn load_map_py(level_to_load: &Level, mut query: Query<(&Level, &mut Map, &mut Position)>) {
    for (level, mut map, mut position) in query.iter_mut() {
        if level == level_to_load {
            // if the map is already loaded, quit the system
            if map.is_loaded() {
                return;
            }
            // get data through arguments
            let mut floor_data = Vec::new();
            let mut position_data = Vec3::ZERO;
            match parse_map_from_python(&mut floor_data, &mut position_data, level) {
                Ok(_) => println!("Python works."),
                Err(e) => println!("error parsing header: {:?}", e),
            };

            // store map data to the map component
            map.floors = floor_data;
            position.0 = position_data;
            //println!("{}", format!("{:?}", map));
        }
    }
}

fn parse_map_from_python(
    floors: &mut Vec<Floor>,
    position: &mut Vec3,
    level: &Level,
) -> PyResult<()> {
    // all communications with Python are placed in the closure
    Python::with_gil(|py| {
        let sys_module = py.import("sys")?;
        // get a list of paths where Python modules may exist
        let path_list: &PyList = sys_module.getattr("path")?.extract()?;
        // create a path to add the list
        let mut map_path = env::current_dir()?;
        map_path.push("scripts");

        // add the path
        path_list
            .insert(0, format!("{}", map_path.display()))
            .unwrap();

        // use scripts/map.py
        let map_module = py.import("map")?;
        let py_map: &PyDict = map_module.call_method0(level.name())?.extract()?;

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

// delete loaded map data
fn unload_map(level_to_unload: &Level, mut query: Query<(&Level, &mut Map, &mut Position)>) {
    for (level, mut map, mut position) in query.iter_mut() {
        if level == level_to_unload {
            map.floors = Vec::new();
            position.0 = Vec3::ZERO;
            return;
        }
    }
}

fn spawn_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Level, &Map, &Position, &mut Visible)>,
) {
    let mesh = Mesh::from(shape::Cube { size: 1.0 });
    let material = StandardMaterial::from(Color::rgb(230. / 255., 230. / 255., 230. / 255.));

    // spawn maps that are invisible and loaded
    for (level, map, position, mut visible) in query.iter_mut() {
        if visible.0 || !map.is_loaded() {
            continue;
        }
        let floors = map.floors.iter();
        floors.for_each(|floor| {
            let height = floor.height as f32;
            for i in 0..map.width() {
                for j in 0..map.depth() {
                    // -1 means no tile.
                    if floor.data[j][i] == -1 {
                        continue;
                    }
                    let x = (map.width() - 1 - i) as f32 + position.0.x;
                    let z = (map.depth() - 1 - j) as f32 + position.0.z;
                    let y = floor.data[j][i] as f32 + height + position.0.y;
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(mesh.clone()),
                            material: materials.add(material.clone()),
                            transform: Transform::from_translation(Vec3::new(x, y, z)),
                            ..default()
                        })
                        .insert(Tile)
                        .insert(level.clone());
                }
            }
        });
        visible.0 = true;
    }
}

fn despawn_map(
    mut commands: Commands,
    mut query: Query<(Entity, &Level, Option<&mut Visible>, Option<&Tile>)>,
    level_to_despawn: &Level,
) {
    for (entity, level, visible, tile) in query.iter_mut() {
        if level == level_to_despawn {
            // despawn tiles
            if tile.is_some() {
                commands.entity(entity).despawn_recursive();
            }
            // set the map to be not visible
            if let Some(mut visible) = visible {
                visible.0 = false;
            }
        }
    }
}
