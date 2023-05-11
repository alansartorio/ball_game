use super::utils::{add_ball, add_blocks_from_state, get_block, Ball, Block, Lives};
use super::{BoardState, InnerGameState};
use crate::{despawn_screen, GameState};
use ball_simulation::SimulationState;
use ball_simulation::*;
use bevy::time::Stopwatch;
use bevy::{prelude::*, sprite::Mesh2dHandle};
use nalgebra::Vector2;
use std::ops::{Add, Div, Mul};

pub struct PlaySimulationPlugin;

#[derive(Component)]
struct OnPlaySimulation;

impl Plugin for PlaySimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(InnerGameState::PlaySimulation),
            add_simulation_state,
        )
        .add_systems(
            Update,
            ((
                touch_to_speed_up.before(update_watch),
                update_watch.before(update_simulation),
                update_simulation,
                handle_stop.after(update_simulation),
                interpolate_simulation.after(update_simulation),
                update_balls.after(interpolate_simulation),
            )
                .run_if(in_state(InnerGameState::PlaySimulation)),)
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(
            OnExit(InnerGameState::PlaySimulation),
            (
                save_state,
                despawn_screen::<OnPlaySimulation>.after(save_state),
            ),
        );
    }
}

#[derive(Component)]
struct Simulation {
    balls_left: usize,
    balls_increment: usize,
    launcher_position: f64,
    spawn_direction: Vector2<f64>,
    state: SimulationState,
    next: Option<(SimulationState, ball_simulation::Event<EventType>)>,
    speed: f64,
}

#[derive(Component)]
struct InterpolatedSimulation {
    state: SimulationState,
}

#[derive(Component, Default)]
struct BlockEntities(Vec<Entity>);

#[derive(Component, Default)]
struct GridPosition(Vector2<usize>);

#[derive(Component, Default)]
struct BallEntities(Vec<Entity>);

#[derive(Component)]
struct BlocksParent;

#[derive(Component)]
struct BallMesh;

#[derive(Component)]
struct SimulationWatch(Stopwatch);

fn add_simulation_state(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board_state: Query<&BoardState>,
    assets: Res<AssetServer>,
) {
    commands.spawn((SimulationWatch(Stopwatch::new()), OnPlaySimulation));

    let mut block_positions = vec![];
    let mut blocks = vec![];

    let [h, w]: [usize; 2] = board_state.single().blocks.shape().try_into().unwrap();
    for ((y, x), &lives) in board_state.single().blocks.indexed_iter() {
        if lives > 0 {
            blocks.push((get_block(w, h, x, y), lives));
            block_positions.push(Vector2::new(x, y));
        }
    }

    let simulation_state = SimulationState {
        time: 0.0,
        space_width: 1.0,
        space_height: 1.0,
        balls: vec![],
        blocks: blocks.iter().map(|&(block, _)| block).collect(),
    };
    commands.spawn((
        Simulation {
            state: simulation_state.clone(),
            next: None,
            balls_left: board_state.single().ball_count,
            balls_increment: 0,
            spawn_direction: board_state.single().direction,
            speed: 1.0,
            launcher_position: board_state.single().launcher_position,
        },
        OnPlaySimulation,
    ));

    let mut block_ids = BlockEntities::default();
    let blocks_parent = commands
        .spawn((
            SpatialBundle::INHERITED_IDENTITY,
            OnPlaySimulation,
            BlocksParent,
        ))
        .id();
    let block_entities = add_blocks_from_state(
        &blocks,
        &mut block_ids.0,
        &mut commands,
        &mut meshes,
        &mut materials,
        blocks_parent,
        assets,
    );
    for (block_entity, position) in block_entities.into_iter().zip(block_positions) {
        commands.entity(block_entity).insert(GridPosition(position));
    }
    commands.spawn((block_ids, OnPlaySimulation));
    let ball_ids = BallEntities::default();
    commands.spawn((ball_ids, OnPlaySimulation));
    commands.spawn((
        InterpolatedSimulation {
            state: simulation_state,
        },
        OnPlaySimulation,
    ));
    commands.spawn((
        Mesh2dHandle::from(meshes.add(shape::Circle::new(1.).into())),
        BallMesh,
        OnPlaySimulation,
    ));
}

fn touch_to_speed_up(
    mut simulation: Query<&mut Simulation>,
    buttons: Res<Input<MouseButton>>,
    touches: Res<Touches>,
) {
    if buttons.just_released(MouseButton::Left) || touches.any_just_released() {
        simulation.single_mut().speed *= 1.2;
    }
}

fn update_watch(
    simulation: Query<&Simulation>,
    time: Res<Time>,
    mut watch: Query<&mut SimulationWatch>,
) {
    watch
        .single_mut()
        .0
        .tick(time.delta().mul_f64(simulation.single().speed));
}

fn update_simulation(
    time: Query<&SimulationWatch>,
    mut simulation: Query<&mut Simulation>,
    mut ball_ids: Query<&mut BallEntities>,
    mut block_ids: Query<&mut BlockEntities>,
    mut interpolated_simulation: Query<&mut InterpolatedSimulation>,
    mut commands: Commands,
    ball_mesh: Query<&Mesh2dHandle, With<BallMesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    blocks_parent: Query<Entity, With<BlocksParent>>,
    mut blocks: Query<(&mut Lives, &Children), With<Block>>,
    mut blocks_texts: Query<&mut Text>,
) {
    let time = time.single();
    let mut ball_ids = ball_ids.single_mut();
    let mut block_ids = block_ids.single_mut();
    let mut simulation = simulation.single_mut();
    let mut interpolated_simulation = interpolated_simulation.single_mut();

    while let Some((next_state, next_event)) = {
        let time = simulation.state.time;
        if simulation.next.is_none() {
            let spawn_event = (simulation.balls_left > 0).then(||ball_simulation::Event {
                time: time.mul(6.0).floor().add(1.0).div(6.0) - time,
                data: EventType::Custom,
            });
            simulation.next = simulation.state.clone().next(spawn_event.as_slice());
        }
        simulation.next.as_ref()
    } && time.0.elapsed_secs_f64() >= next_state.time {
        let next_state = next_state.clone();
        let next_event = *next_event;

        simulation.state = next_state.clone();
        simulation.next = None;
        //println!("Time: {} | Event: {:?}", simulation.state.time, event);

        if let EventType::Custom = next_event.data {
            let spawn_direction = simulation.spawn_direction;
            let launcher_position = simulation.launcher_position;
            simulation.state.balls.push(ball_simulation::Ball {
                position: Vector2::new(launcher_position, 0.0),
                velocity: spawn_direction * 2.0,
                radius: 0.02,
            });
            simulation.balls_left -= 1;
            add_ball(
                &mut commands,
                &mut ball_ids.0,
                ball_mesh.get_single().unwrap().clone(),
                &mut materials,
                OnPlaySimulation
            );
        } else if let EventType::Collision(CollisionData { ball, against }) = next_event.data {
            if let CollisionType::Wall(WallType::YNegative) = against {
                let entity = ball_ids.0.remove(ball);
                commands.entity(entity).despawn();
                simulation.state.balls.remove(ball);
            } else if let CollisionType::Block { index, .. } = against {
                let hit_entity = block_ids.0[index];
                let (mut lives, children) = blocks.get_mut(hit_entity).unwrap();
                lives.0 -= 1;
                let mut text = blocks_texts.get_mut(*children.into_iter().find(|&&child| blocks_texts.contains(child)).unwrap()).unwrap();
                text.sections[0].value = lives.0.to_string();

                if lives.0 == 0 {
                    commands.entity(blocks_parent.single()).remove_children(&[hit_entity]);
                    commands.entity(hit_entity).despawn_recursive();
                    simulation.state.blocks.remove(index);
                    simulation.balls_increment += 1;
                    block_ids.0.remove(index);
                }
            }
        }

        interpolated_simulation.state = simulation.state.clone();
    }
}

fn interpolate_simulation(
    time: Query<&SimulationWatch>,
    simulation: Query<&Simulation>,
    mut interpolated_simulation: Query<&mut InterpolatedSimulation>,
) {
    let mut interpolated_simulation = interpolated_simulation.single_mut();
    let simulation = simulation.single();
    let time = time.single();
    let advance = time.0.elapsed_secs_f64() - interpolated_simulation.state.time;
    for (mut ball, initial_ball) in interpolated_simulation
        .state
        .balls
        .iter_mut()
        .zip(&simulation.state.balls)
    {
        ball.position = initial_ball.position + initial_ball.velocity * advance;
    }
}

fn handle_stop(
    keys: Res<Input<KeyCode>>,
    simulation: Query<&Simulation>,
    mut inner_game_state: ResMut<NextState<InnerGameState>>,
) {
    if simulation.single().next.is_none() || keys.just_pressed(KeyCode::Space) {
        inner_game_state.set(InnerGameState::AnimateBlocksIn);
    }
}

fn update_balls(
    simulation: Query<&InterpolatedSimulation>,
    mut balls: Query<(&mut Transform, &mut Visibility), With<Ball>>,
) {
    let simulation = simulation.single();
    for ((mut ball_entity, mut visibility), ball) in balls.iter_mut().zip(&simulation.state.balls) {
        ball_entity.translation = Vec3::new(ball.position.x as f32, ball.position.y as f32, -0.5);
        ball_entity.scale = Vec2::new(ball.radius as f32, ball.radius as f32).extend(1.0);
        *visibility = Visibility::Visible;
    }
}

fn save_state(
    blocks: Query<(&GridPosition, &Lives), With<Block>>,
    mut board_state: Query<&mut BoardState>,
    simulation: Query<&Simulation>,
) {
    let simulation = simulation.single();
    let board_state = &mut board_state.single_mut();

    for lives in board_state.blocks.iter_mut() {
        *lives = 0;
    }

    board_state.ball_count += simulation.balls_increment;
    for (block_position, lives) in blocks.iter() {
        board_state.blocks[(block_position.0.y, block_position.0.x)] = lives.0;
    }
}
