/// Modules under lib.rs
pub use audio::*;
pub use camera::*;
pub use map::*;

pub mod audio;
pub mod camera;
pub mod map;

/// Crates for lib.rs
use bevy::{app::*, prelude::*};

pub struct IndividualPlugins;

impl PluginGroup for IndividualPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AudioPlugin)
            .add(CameraPlugin)
            .add(MapPlugin)
    }
}

// set up
pub fn setup(mut commands: Commands) {
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
