#![feature(let_chains)]
#![feature(option_as_slice)]

use bevy::prelude::*;
mod game;
use game::GamePlugin;
mod menu;
use menu::MenuPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(GamePlugin)
        .add_plugin(MenuPlugin)
        .run();
}
