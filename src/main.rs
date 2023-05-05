#![feature(let_chains)]
#![feature(option_as_slice)]

use bevy::prelude::*;
mod game;
use game::GamePlugin;
mod menu;
use menu::MenuPlugin;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, States)]
pub enum GameState {
    #[default]
    Menu,
    Game,
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .add_plugin(MenuPlugin)
        .run();
}


fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
