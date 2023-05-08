#![feature(let_chains)]
#![feature(option_as_slice)]

mod game;
mod menu;
mod colors;

use bevy::prelude::*;

use game::GamePlugin;
use menu::MenuPlugin;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug, States)]
pub enum GameState {
    #[default]
    Menu,
    Game,
}

#[bevy_main]
pub fn main() {
    let mut app = App::new();

    app.insert_resource(ClearColor(*colors::GLOBAL_BACKGROUND))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_state::<GameState>()
        .add_plugin(GamePlugin)
        .add_plugin(MenuPlugin);

    app.run();
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
