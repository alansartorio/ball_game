#![feature(let_chains)]
#![feature(option_as_slice)]

use bevy::prelude::*;
mod game;
use game::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
}
