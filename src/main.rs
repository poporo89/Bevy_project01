use bevy::{prelude::*, DefaultPlugins};
use bevy_project01::*;
use bevy_rhai::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..default()
        }))
        .add_plugin(StandardScriptPlugin)
        .add_plugins(IndividualPlugins)
        .add_startup_system(setup)
        .run();
}
