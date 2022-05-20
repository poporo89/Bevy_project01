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
            .insert_resource(ClearColor(Color::rgb(10. / 255., 10. / 255., 10. / 255.)))
            .register_inspectable::<Speed>()
            .add_startup_system(setup_camera)
            .add_system(move_camera);
    }
}

// set up a camera
fn setup_camera(mut commands: Commands) {
    let position = Vec3::new(7.0, 20.0, 7.0);
    let height = position.y;
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 10.0;
    camera.transform = Transform::from_xyz(-height, height, -height)
        .looking_at(Vec3::ZERO, Vec3::Y)
        .with_translation(position);

    commands
        .spawn_bundle(camera)
        .insert(Speed(15.0))
        .insert(MovableCamera);
}

// move camera by hjkl
fn move_camera(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &Speed), With<MovableCamera>>,
    timer: Res<Time>,
) {
    let (mut transform, speed) = query.single_mut();
    let sqrt = std::f32::consts::SQRT_2;

    if keyboard_input.pressed(KeyCode::J) {
        transform.translation -= (Vec3::X + Vec3::Z) / sqrt * speed.0 * timer.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::K) {
        transform.translation += (Vec3::X + Vec3::Z) / sqrt * speed.0 * timer.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::H) {
        transform.translation += (Vec3::X - Vec3::Z) / sqrt * speed.0 * timer.delta_seconds();
    }

    if keyboard_input.pressed(KeyCode::L) {
        transform.translation -= (Vec3::X - Vec3::Z) / sqrt * speed.0 * timer.delta_seconds();
    }
}
