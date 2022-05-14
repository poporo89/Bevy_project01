use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

#[derive(Component)]
pub struct MovableCamera;

#[derive(Component, Inspectable)]
struct Speed(f32);

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa { samples: 4 })
            .register_inspectable::<Speed>()
            .add_startup_system(setup_camera)
            .add_system(move_camera);
    }
}

// set up a simple 3D scene
fn setup_camera(mut commands: Commands) {
    // set up a camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 10.0;
    camera.transform = Transform::from_xyz(-10.0, 10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y);

    // camera
    commands
        .spawn_bundle(camera)
        .insert(Speed(15.0))
        .insert(MovableCamera);
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
