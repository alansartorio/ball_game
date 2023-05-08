use bevy::prelude::*;
use ndarray::{ArrayView1, Axis};
use rand::distributions::uniform::SampleRange;

use crate::despawn_screen;

use super::{
    utils::{add_blocks_from_state, get_block},
    BoardState, InnerGameState,
};

#[derive(Component)]
struct OnAnimateBlocksIn;

pub struct AnimateBlocksInPlugin;

impl Plugin for AnimateBlocksInPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InnerGameState::AnimateBlocksIn),
            (generate_new_blocks, generate_graphic_blocks),
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

fn generate_new_blocks(mut board_state: Query<&mut BoardState>) {
    let mut board_state = board_state.single_mut();
    let mut rng = rand::thread_rng();

    let [_h, w]: [usize; 2] = board_state.blocks.shape().try_into().unwrap();
    let mut blocks = vec![false; w];
    for block in blocks.iter_mut() {
        *block = (0.0..=1.0).sample_single(&mut rng) > 0.5;
    }
    println!("{blocks:?}");

    board_state
        .blocks
        .push_row(ArrayView1::from(&blocks))
        .unwrap();

    board_state.blocks.remove_index(Axis(0), 0);
    
    println!("{:?}", board_state.blocks);
}

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
        .spawn((SpatialBundle::INHERITED_IDENTITY, OnAnimateBlocksIn))
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

    commands.spawn(AnimationTimer(Timer::from_seconds(1.0, TimerMode::Once)));
}

fn animate(
    time: Res<Time>,
    mut timer: Query<&mut AnimationTimer>,
    mut inner_game_state: ResMut<NextState<InnerGameState>>,
) {
    timer.single_mut().0.tick(time.delta());
    let timer = &timer.single().0;

    let remaining = timer.remaining_secs();

    if timer.finished() {
        inner_game_state.set(InnerGameState::PlaySimulation);
    }
}