use super::utils::{add_ball, add_blocks_from_state, get_block, Ball};
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
                update_watch.before(update_simulation),
                update_simulation,
                interpolate_simulation.after(update_simulation),
                update_balls.after(interpolate_simulation),
            )
                .run_if(in_state(InnerGameState::PlaySimulation)),)
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(
            OnExit(InnerGameState::PlaySimulation),
            despawn_screen::<OnPlaySimulation>,
        );
    }
}

#[derive(Component)]
struct Simulation {
    state: SimulationState,
    next: Option<(SimulationState, ball_simulation::Event<EventType>)>,
}

#[derive(Component)]
struct InterpolatedSimulation {
    state: SimulationState,
}

#[derive(Component, Default)]
struct BlockEntities(Vec<Entity>);

#[derive(Component, Default)]
struct BallEntities(Vec<Entity>);

#[derive(Component)]
struct BallMesh;

#[derive(Component)]
struct SimulationWatch(Stopwatch);

fn add_simulation_state(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board_state: Query<&BoardState>,
) {
    commands.spawn((SimulationWatch(Stopwatch::new()), OnPlaySimulation));

    let mut blocks = vec![];

    let [h, w]: [usize; 2] = board_state.single().blocks.shape().try_into().unwrap();
    for ((y, x), &has_block) in board_state.single().blocks.indexed_iter() {
        if has_block {
            blocks.push(get_block(w, h, x, y));
        }
    }

    let simulation_state = SimulationState {
        time: 0.0,
        space_width: 1.0,
        space_height: 1.0,
        balls: vec![],
        blocks,
    };
    commands.spawn((
        Simulation {
            state: simulation_state.clone(),
            next: None,
        },
        OnPlaySimulation,
    ));
    let mut block_ids = BlockEntities::default();
    let blocks_parent = commands
        .spawn((SpatialBundle::INHERITED_IDENTITY, OnPlaySimulation))
        .id();
    add_blocks_from_state(
        &simulation_state.blocks,
        &mut block_ids.0,
        &mut commands,
        &mut meshes,
        &mut materials,
        blocks_parent,
    );
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

fn update_watch(time: Res<Time>, mut watch: Query<&mut SimulationWatch>) {
    watch.single_mut().0.tick(time.delta());
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
) {
    let time = time.single();
    let mut ball_ids = ball_ids.single_mut();
    let mut block_ids = block_ids.single_mut();
    let mut simulation = simulation.single_mut();
    let mut interpolated_simulation = interpolated_simulation.single_mut();

    while let Some((next_state, next_event)) = {
        let time = simulation.state.time;
        if simulation.next.is_none() {
            let spawn_event = (time < 10.0).then(||ball_simulation::Event {
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
        interpolated_simulation.state = simulation.state.clone();
        //println!("Time: {} | Event: {:?}", simulation.state.time, event);

        if let EventType::Custom = next_event.data {
            simulation.state.balls.push(ball_simulation::Ball {
                position: Vector2::new(0.2, 0.0),
                velocity: Vector2::new(0.2, 0.02) * 10.0,
                radius: 0.01,
            });
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
                let entity_index = block_ids.0.remove(index);
                commands.entity(entity_index).despawn();
                simulation.state.blocks.remove(index);
            }
        }
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
