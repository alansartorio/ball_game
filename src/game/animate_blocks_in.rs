use bevy::prelude::*;
use ndarray::{ArrayView1, Axis};
use rand::distributions::uniform::SampleRange;
use simple_easing::cubic_out;

use crate::despawn_screen;

use super::{
    utils::{add_blocks_from_state, get_block, get_block_separations},
    BoardState, InnerGameState,
};

#[derive(Component)]
struct OnAnimateBlocksIn;

pub struct AnimateBlocksInPlugin;

impl Plugin for AnimateBlocksInPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InnerGameState::AnimateBlocksIn),
            (
                generate_new_blocks,
                generate_graphic_blocks.after(generate_new_blocks),
            ),
        )
        .add_systems(
            Update,
            animate.run_if(in_state(InnerGameState::AnimateBlocksIn)),
        )
        .add_systems(
            OnExit(InnerGameState::AnimateBlocksIn),
            despawn_screen::<OnAnimateBlocksIn>,
        );
    }
}

#[derive(Component)]
struct AnimationTimer(Timer);

#[derive(Component, Default)]
struct BlockEntities(Vec<Entity>);

#[derive(Component, Default)]
struct BallEntities(Vec<Entity>);

#[derive(Component, Default)]
struct BlocksParent;

fn generate_new_blocks(mut board_state: Query<&mut BoardState>) {
    let mut board_state = board_state.single_mut();
    let mut rng = rand::thread_rng();

    let [_h, w]: [usize; 2] = board_state.blocks.shape().try_into().unwrap();
    let mut blocks = vec![0; w];
    for block in blocks.iter_mut() {
        if (0.0..=1.0).sample_single(&mut rng) > 0.5 {
            *block = 4;
        }
    }

    board_state
        .blocks
        .push_row(ArrayView1::from(&blocks))
        .unwrap();

    board_state.blocks.remove_index(Axis(0), 0);
}

fn generate_graphic_blocks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board_state: Query<&mut BoardState>,
) {
    let mut blocks = vec![];
    let [h, w]: [usize; 2] = board_state.single().blocks.shape().try_into().unwrap();
    for ((y, x), &lives) in board_state.single().blocks.indexed_iter() {
        if lives > 0 {
            blocks.push((get_block(w, h, x, y), lives));
        }
    }

    let mut block_ids = BlockEntities::default();
    let blocks_parent = commands
        .spawn((
            SpatialBundle::INHERITED_IDENTITY,
            BlocksParent,
            OnAnimateBlocksIn,
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

    commands.spawn((block_ids, OnAnimateBlocksIn));

    commands.spawn((
        AnimationTimer(Timer::from_seconds(0.3, TimerMode::Once)),
        OnAnimateBlocksIn,
    ));
}

fn animate(
    time: Res<Time>,
    mut timer: Query<&mut AnimationTimer>,
    mut inner_game_state: ResMut<NextState<InnerGameState>>,
    mut blocks_parent: Query<&mut Transform, With<BlocksParent>>,
    board_state: Query<&mut BoardState>,
) {
    timer.single_mut().0.tick(time.delta());
    let timer = &timer.single().0;

    let easing = cubic_out(timer.percent());

    let movement = {
        let [rows, columns]: [usize; 2] = board_state.single().blocks.shape().try_into().unwrap();
        get_block_separations(columns, rows).y
    };

    blocks_parent.single_mut().translation.y = (1.0 - easing) * movement as f32;

    if timer.finished() {
        inner_game_state.set(InnerGameState::AcceptUserInput);
    }
}
