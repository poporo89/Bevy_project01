use bevy::{prelude::*, DefaultPlugins};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_project01::*;
use bevy_rhai::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(StandardScriptPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugins(IndividualPlugins)
        .add_startup_system(setup)
        .run();
}
