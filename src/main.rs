use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
//use bevy_kira_audio::{Audio, AudioPlugin};

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

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        //.add_plugin(AudioPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        //.add_startup_system(start_background_audio)
        .add_startup_system(setup)
        .add_startup_system_to_stage(StartupStage::Startup, spawn_floor)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_tiles)
        .run();
}

// BGM
//fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
//audio.play_looped(asset_server.load("Lady_Maria.mp3"));
//}

// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // set up the camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 10.0;
    camera.transform = Transform::from_xyz(-10.0, 10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y);

    // camera
    commands.spawn_bundle(camera);

    // light
    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_xyz(-3.0, 8.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 3000.0,
            color: Color::WHITE,
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
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
    let material = StandardMaterial::from(Color::rgb(0.8, 0.7, 0.6));

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
                        ..Default::default()
                    })
                    .insert(Tile);
            }
        }
    }
}
