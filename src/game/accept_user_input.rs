use bevy::prelude::*;

use crate::despawn_screen;

use super::{
    utils::{add_blocks_from_state, get_block},
    BoardState, InnerGameState,
};

#[derive(Component)]
struct OnAcceptUserInput;

pub struct AcceptUserInputPlugin;

impl Plugin for AcceptUserInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InnerGameState::AcceptUserInput),
            generate_graphic_blocks,
        )
        .add_systems(
            Update,
            update.run_if(in_state(InnerGameState::AcceptUserInput)),
        )
        .add_systems(
            OnExit(InnerGameState::AcceptUserInput),
            despawn_screen::<OnAcceptUserInput>,
        );
    }
}

#[derive(Component, Default)]
struct BlockEntities(Vec<Entity>);

#[derive(Component, Default)]
struct BallEntities(Vec<Entity>);

#[derive(Component, Default)]
struct BlocksParent;

fn generate_graphic_blocks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board_state: Query<&mut BoardState>,
) {
    let mut blocks = vec![];
    let [h, w]: [usize; 2] = board_state.single().blocks.shape().try_into().unwrap();
    for ((y, x), &has_block) in board_state.single().blocks.indexed_iter() {
        if has_block {
            blocks.push(get_block(w, h, x, y));
        }
    }

    let mut block_ids = BlockEntities::default();
    let blocks_parent = commands
        .spawn((
            SpatialBundle::INHERITED_IDENTITY,
            BlocksParent,
            OnAcceptUserInput,
        ))
        .id();
    add_blocks_from_state(
        &blocks,
        &mut block_ids.0,
        &mut commands,
        &mut meshes,
        &mut materials,
        blocks_parent,
    );

    commands.spawn((block_ids, OnAcceptUserInput));
}

fn update(
    buttons: Res<Input<MouseButton>>,
    mut inner_game_state: ResMut<NextState<InnerGameState>>,
) {
    if buttons.just_released(MouseButton::Left) {
        inner_game_state.set(InnerGameState::PlaySimulation);
    }
}
