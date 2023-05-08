mod accept_user_input;
mod animate_blocks_in;
mod play_simulation;
mod utils;

use std::f32::consts::PI;

use crate::{despawn_screen, GameState};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use ndarray::Array2;

use self::accept_user_input::AcceptUserInputPlugin;
use self::animate_blocks_in::AnimateBlocksInPlugin;
use self::play_simulation::PlaySimulationPlugin;

#[derive(Component)]
pub struct OnGame;

pub struct GamePlugin;

#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, States)]
enum InnerGameState {
    #[default]
    Inactive,
    AnimateBlocksIn,
    AcceptUserInput,
    PlaySimulation,
}

#[derive(Component)]
struct BoardState {
    blocks: Array2<bool>,
    ball_count: usize,
    launcher_position: f64,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<InnerGameState>()
            .add_systems(
                OnEnter(GameState::Game),
                (
                    game_setup,
                    add_game_rectangle,
                    initialize_camera,
                    initialize_game_state,
                ),
            )
            .add_plugin(AnimateBlocksInPlugin)
            .add_plugin(AcceptUserInputPlugin)
            .add_plugin(PlaySimulationPlugin)
            .add_systems(
                OnExit(GameState::Game),
                (despawn_screen::<OnGame>, set_inactive_game_state),
            );
    }
}

fn game_setup(mut inner_game_state: ResMut<NextState<InnerGameState>>) {
    inner_game_state.set(InnerGameState::AnimateBlocksIn);
}

fn add_game_rectangle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::RegularPolygon::new(2f32.sqrt() / 2.0, 4).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::BLACK)),
            transform: Transform::from_xyz(0.5, 0.5, -1.0)
                .with_rotation(Quat::from_rotation_z(PI / 4.0)),
            ..default()
        },
        OnGame,
    ));
}

fn initialize_game_state(mut commands: Commands) {
    commands.spawn((
        BoardState {
            blocks: Array2::default((10, 10)),
            ball_count: 1,
            launcher_position: 0.5,
        },
        OnGame,
    ));
}

fn initialize_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle {
        transform: Transform::from_xyz(0.5, 0.5, 0.0),
        ..Default::default()
    };
    camera.projection.scale = 0.002;
    commands.spawn((camera, OnGame));
}

fn set_inactive_game_state(mut inner_game_state: ResMut<NextState<InnerGameState>>) {
    inner_game_state.set(InnerGameState::Inactive);
}
