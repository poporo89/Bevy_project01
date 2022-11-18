use bevy::prelude::*;
use bevy_rhai::*;
use rhai::plugin::*;

#[derive(Bundle)]
struct LevelBundle {
    level: Level,
    map: Map,
    position: Position,
    visible: Visible,
}

#[derive(Component, PartialEq, Clone)]
enum Level {
    TestMap,
}

impl Level {
    // bind levels to Python method names
    fn name(&self) -> &str {
        match *self {
            Level::TestMap => "test_map",
        }
    }
}

#[derive(Component, Default, Debug, Clone)]
struct Map {
    floors: Vec<Floor>,
    stairs: Vec<Stair>,
    walls: Vec<Wall>,
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

    fn new() -> Self {
        Map {
            floors: Vec::new(),
            stairs: Vec::new(),
            walls: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.floors = Vec::new();
        self.stairs = Vec::new();
        self.walls = Vec::new();
    }
}

#[derive(Component, Debug, Default, Clone)]
struct Floor {
    height: i32,
    data: Vec<Vec<i32>>,
}

impl Floor {
    fn new() -> Self {
        Self {
            height: 0,
            data: Vec::new(),
        }
    }
}

#[derive(Component, Debug, Clone)]
struct Stair {
    translation: Vec3,
    direction: Direction,
    scale: Vec3,
}

impl Stair {
    fn new() -> Self {
        Self {
            translation: Vec3::ZERO,
            direction: Direction::PZ,
            scale: Vec3::ZERO,
        }
    }
}

#[derive(Component, Debug, Clone)]
struct Wall {
    translation: Vec3,
    direction: Direction,
    size: Vec2,
}

impl Wall {
    fn new() -> Self {
        Self {
            translation: Vec3::ZERO,
            direction: Direction::PZ,
            size: Vec2::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
enum Direction {
    PX,
    MX,
    PZ,
    MZ,
}

#[derive(Component, Debug, Default, Clone)]
struct Position(Vec3);

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
    // test map
    commands
        .spawn(LevelBundle {
            level: Level::TestMap,
            map: Map::new(),
            position: Position(Vec3::ZERO),
            visible: Visible(false),
        })
        .insert_bundle(RhaiBundle {
            engine: StandardEngine::with_engine({
                let mut engine = Engine::new_raw();
                engine.set_strict_variables(true);
                engine.disable_symbol("eval");
                engine
            }),
            script_handle: handle,
            scope: StandardScope::default(),
        })
        .insert(Floor::default());
}

#[allow(clippy::complexity)]
fn manual_load_map(
    keyboard_input: Res<Input<KeyCode>>,
    scripts: Res<Assets<StandardScript>>,
    query: Query<(
        &mut Map,
        &mut Position,
        &StandardEngine,
        &Handle<StandardScript>,
        &mut StandardScope,
    )>,
) {
    if keyboard_input.just_pressed(KeyCode::P) {
        let level_to_load = Level::TestMap;
        load_map(&level_to_load, scripts, query);
    }
}

#[allow(clippy::complexity)]
fn load_map(
    level_to_load: &Level,
    scripts: Res<Assets<StandardScript>>,
    mut query: Query<(
        &mut Map,
        &mut Position,
        &StandardEngine,
        &Handle<StandardScript>,
        &mut StandardScope,
    )>,
) {
    for (mut map, mut position, engine, script, mut scope) in query.iter_mut() {
        if let Some(script) = scripts.get(script) {
            let result: rhai::Map = engine
                .call_fn(&mut scope, &script.ast, level_to_load.name(), ())
                .unwrap();
            for (map_key, map_value) in result.iter() {
                match map_key.as_str() {
                    "position" => {
                        let raw_position = map_value.clone_cast::<rhai::Array>();
                        let vec: Vec<f32> = raw_position
                            .into_iter()
                            .map(|item| item.try_cast::<f32>().unwrap())
                            .collect();
                        position.0 = Vec3::new(vec[0], vec[1], vec[2]);
                    }
                    "floors" => {
                        for raw_floor in map_value.clone_cast::<Vec<Dynamic>>().into_iter() {
                            let mut temp_floor = Floor::new();
                            let parsed_floor = raw_floor.try_cast::<rhai::Map>().unwrap();
                            for (floor_key, floor_value) in parsed_floor.iter() {
                                match floor_key.as_str() {
                                    "height" => {
                                        let height = floor_value.clone_cast::<i32>();
                                        temp_floor.height = height;
                                    }
                                    "data" => {
                                        let data = floor_value.clone_cast::<rhai::Array>();
                                        temp_floor.data = data
                                            .into_iter()
                                            .map(|item| item.into_typed_array::<i32>().unwrap())
                                            .collect();
                                    }
                                    _ => {}
                                }
                            }
                            map.floors.push(temp_floor);
                        }
                    }
                    "stairs" => {
                        for raw_stair in map_value.clone_cast::<Vec<Dynamic>>().into_iter() {
                            let mut temp_stair = Stair::new();
                            let parsed_stair = raw_stair.try_cast::<rhai::Map>().unwrap();
                            for (stair_key, stair_value) in parsed_stair.iter() {
                                match stair_key.as_str() {
                                    "translation" => {
                                        let raw_translation =
                                            stair_value.clone_cast::<rhai::Array>();
                                        let vec: Vec<f32> = raw_translation
                                            .into_iter()
                                            .map(|item| item.try_cast::<f32>().unwrap())
                                            .collect();
                                        temp_stair.translation = Vec3::new(vec[0], vec[1], vec[2]);
                                    }
                                    "direction" => {
                                        let raw_direction = stair_value.clone_cast::<String>();
                                        match raw_direction.as_str() {
                                            "PX" => {
                                                temp_stair.direction = Direction::PX;
                                            }
                                            "MX" => {
                                                temp_stair.direction = Direction::MX;
                                            }
                                            "MZ" => {
                                                temp_stair.direction = Direction::MZ;
                                            }
                                            _ => {
                                                temp_stair.direction = Direction::PZ;
                                            }
                                        }
                                    }
                                    "scale" => {
                                        let raw_scale = stair_value.clone_cast::<rhai::Array>();
                                        let vec: Vec<f32> = raw_scale
                                            .into_iter()
                                            .map(|item| item.try_cast::<f32>().unwrap())
                                            .collect();
                                        temp_stair.scale = Vec3::new(vec[0], vec[1], vec[2]);
                                    }
                                    _ => {}
                                }
                            }
                            map.stairs.push(temp_stair);
                        }
                    }
                    "walls" => {
                        for raw_wall in map_value.clone_cast::<Vec<Dynamic>>().into_iter() {
                            let mut temp_wall = Wall::new();
                            let parsed_wall = raw_wall.try_cast::<rhai::Map>().unwrap();
                            for (wall_key, wall_value) in parsed_wall.iter() {
                                match wall_key.as_str() {
                                    "translation" => {
                                        let raw_translation =
                                            wall_value.clone_cast::<rhai::Array>();
                                        let vec: Vec<f32> = raw_translation
                                            .into_iter()
                                            .map(|item| item.try_cast::<f32>().unwrap())
                                            .collect();
                                        temp_wall.translation = Vec3::new(vec[0], vec[1], vec[2]);
                                    }
                                    "direction" => {
                                        let raw_direction = wall_value.clone_cast::<String>();
                                        match raw_direction.as_str() {
                                            "PX" => {
                                                temp_wall.direction = Direction::PX;
                                            }
                                            "MX" => {
                                                temp_wall.direction = Direction::MX;
                                            }
                                            "MZ" => {
                                                temp_wall.direction = Direction::MZ;
                                            }
                                            _ => {
                                                temp_wall.direction = Direction::PZ;
                                            }
                                        }
                                    }
                                    "size" => {
                                        let raw_size = wall_value.clone_cast::<rhai::Array>();
                                        let vec: Vec<f32> = raw_size
                                            .into_iter()
                                            .map(|item| item.try_cast::<f32>().unwrap())
                                            .collect();
                                        temp_wall.size = Vec2::new(vec[0], vec[1]);
                                    }
                                    _ => {}
                                }
                            }
                            map.walls.push(temp_wall);
                        }
                    }
                    _ => {}
                }
            }
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

// delete loaded map data
fn unload_map(level_to_unload: &Level, mut query: Query<(&Level, &mut Map, &mut Position)>) {
    for (level, mut map, mut position) in query.iter_mut() {
        if level == level_to_unload {
            map.clear();
            position.0 = Vec3::ZERO;
            return;
        }
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
        // floors
        let floors = map.floors.iter();
        floors.for_each(|floor| {
            let floor_height = floor.height as f32;
            for i in 0..map.width() {
                for j in 0..map.depth() {
                    // -1 means no tile.
                    if floor.data[j][i] == -1 {
                        continue;
                    }
                    let tile_height = floor.data[j][i] as f32;
                    let x = (map.width() - 1 - i) as f32 + position.0.x + 0.5;
                    let z = (map.depth() - 1 - j) as f32 + position.0.z + 0.5;
                    let y = tile_height / 2.0 + floor_height + position.0.y;
                    let scale = Vec3::new(1.0, tile_height, 1.0);
                    if floor.data[j][i] != 0 {
                        commands
                            .spawn(PbrBundle {
                                mesh: meshes.add(mesh.clone()),
                                material: materials.add(material.clone()),
                                transform: Transform::from_translation(Vec3::new(x, y, z))
                                    .with_scale(scale),
                                ..default()
                            })
                            .insert(Tile)
                            .insert(level.clone());
                    }
                    commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(mesh.clone()),
                            material: materials.add(material.clone()),
                            transform: Transform::from_translation(Vec3::new(x, -0.25, z))
                                .with_scale(Vec3::new(1.0, 0.5, 1.0)),
                            ..default()
                        })
                        .insert(Tile)
                        .insert(level.clone());
                }
            }
        });
        // stairs
        let stairs = map.stairs.iter();
        stairs.for_each(|stair| {
            let translation = stair.translation;
            let scale = stair.scale;
            match stair.direction {
                Direction::PX => {
                    let num = 3 * scale.x as usize;
                    for i in 1..=num {
                        let po = Vec3::new(
                            (i - 1) as f32 / 3.0 + 1.0 / 6.0,
                            i as f32 / 6.0 * scale.y / scale.x,
                            scale.z / 2.0,
                        );
                        commands
                            .spawn(PbrBundle {
                                mesh: meshes.add(mesh.clone()),
                                material: materials.add(material.clone()),
                                transform: Transform::from_translation(translation + po)
                                    .with_scale(Vec3::new(
                                        1.0 / 3.0,
                                        i as f32 / 3.0 * scale.y / scale.x,
                                        scale.z,
                                    )),
                                ..default()
                            })
                            .insert(Tile)
                            .insert(level.clone());
                    }
                }
                Direction::MX => {
                    let num = 3 * scale.x as usize;
                    for i in 1..=num {
                        let po = Vec3::new(
                            (i - 1) as f32 / 3.0 + 1.0 / 6.0,
                            (num - i + 1) as f32 / 6.0 * scale.y / scale.x,
                            scale.z / 2.0,
                        );
                        commands
                            .spawn(PbrBundle {
                                mesh: meshes.add(mesh.clone()),
                                material: materials.add(material.clone()),
                                transform: Transform::from_translation(translation + po)
                                    .with_scale(Vec3::new(
                                        1.0 / 3.0,
                                        (num - i + 1) as f32 / 3.0 * scale.y / scale.x,
                                        scale.z,
                                    )),
                                ..default()
                            })
                            .insert(Tile)
                            .insert(level.clone());
                    }
                }
                Direction::PZ => {
                    let num = 3 * scale.z as usize;
                    for i in 1..=num {
                        let po = Vec3::new(
                            scale.x / 2.0,
                            i as f32 / 6.0 * scale.y / scale.z,
                            (i - 1) as f32 / 3.0 + 1.0 / 6.0,
                        );
                        commands
                            .spawn(PbrBundle {
                                mesh: meshes.add(mesh.clone()),
                                material: materials.add(material.clone()),
                                transform: Transform::from_translation(translation + po)
                                    .with_scale(Vec3::new(
                                        scale.x,
                                        i as f32 / 3.0 * scale.y / scale.z,
                                        1.0 / 3.0,
                                    )),
                                ..default()
                            })
                            .insert(Tile)
                            .insert(level.clone());
                    }
                }
                Direction::MZ => {
                    let num = 3 * scale.z as usize;
                    for i in 1..=num {
                        let po = Vec3::new(
                            scale.x / 2.0,
                            (num - i + 1) as f32 / 6.0 * scale.y / scale.z,
                            (i - 1) as f32 / 3.0 + 1.0 / 6.0,
                        );
                        commands
                            .spawn(PbrBundle {
                                mesh: meshes.add(mesh.clone()),
                                material: materials.add(material.clone()),
                                transform: Transform::from_translation(translation + po)
                                    .with_scale(Vec3::new(
                                        scale.x,
                                        (num - i + 1) as f32 / 3.0 * scale.y / scale.z,
                                        1.0 / 3.0,
                                    )),
                                ..default()
                            })
                            .insert(Tile)
                            .insert(level.clone());
                    }
                }
            }
        });
        // walls
        let walls = map.walls.iter();
        walls.for_each(|wall| {
            let translation = wall.translation;
            let size = wall.size;
            let material =
                StandardMaterial::from(Color::rgb(230. / 255., 230. / 255., 230. / 255.));
            match wall.direction {
                Direction::PX => {
                    let material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
                    let offset = Vec3::new(-0.01, size.y / 2.0 - 0.5, size.x / 2.0);
                    let mesh = Mesh::from(shape::Quad { size, flip: false });
                    commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(mesh),
                            material: materials.add(material),
                            transform: Transform::from_rotation(Quat::from_rotation_y(
                                -std::f32::consts::FRAC_PI_2,
                            ))
                            .with_translation(translation + offset),
                            ..default()
                        })
                        .insert(Tile)
                        .insert(level.clone());
                }
                Direction::MX => {
                    let offset = Vec3::new(0.0, size.y / 2.0 - 0.5, size.x / 2.0);
                    let mesh = Mesh::from(shape::Quad { size, flip: false });
                    commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(mesh),
                            material: materials.add(material),
                            transform: Transform::from_rotation(Quat::from_rotation_y(
                                -std::f32::consts::FRAC_PI_2,
                            ))
                            .with_translation(translation + offset),
                            ..default()
                        })
                        .insert(Tile)
                        .insert(level.clone());
                }
                Direction::PZ => {
                    let material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
                    let offset = Vec3::new(size.x / 2.0, size.y / 2.0 - 0.5, -0.01);
                    let mesh = Mesh::from(shape::Quad { size, flip: false });
                    commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(mesh),
                            material: materials.add(material),
                            transform: Transform::from_rotation(Quat::from_rotation_y(
                                std::f32::consts::PI,
                            ))
                            .with_translation(translation + offset),
                            ..default()
                        })
                        .insert(Tile)
                        .insert(level.clone());
                }
                Direction::MZ => {
                    let offset = Vec3::new(size.x / 2.0, size.y / 2.0 - 0.5, 0.0);
                    let mesh = Mesh::from(shape::Quad { size, flip: false });
                    commands
                        .spawn(PbrBundle {
                            mesh: meshes.add(mesh),
                            material: materials.add(material),
                            transform: Transform::from_rotation(Quat::from_rotation_y(
                                std::f32::consts::PI,
                            ))
                            .with_translation(translation + offset),
                            visibility: Visibility { is_visible: false },
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
