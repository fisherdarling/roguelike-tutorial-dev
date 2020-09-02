mod game;
mod tcod_plugin;

use std::time::Duration;

use bevy::{
    app::AppExit, app::ScheduleRunnerPlugin, core::CorePlugin, input::InputPlugin, prelude::*,
    type_registry::TypeRegistryPlugin,
};
use tcod::{
    colors,
    console::*,
    input::{
        Key,
        KeyCode::{self, *},
    },
};
use tcod_plugin::{Tcod, TcodPlugin};

fn main() {
    // let wait_duration = 1.0 / tcod_plugin::LIMIT_FPS;

    AppBuilder::default()
        .add_plugin(TypeRegistryPlugin::default())
        .add_plugin(CorePlugin::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(game::GamePlugin::default())
        .run();
}
