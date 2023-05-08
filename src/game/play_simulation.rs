use super::{BoardState, InnerGameState};
use crate::{despawn_screen, GameState};
use ball_simulation::SimulationState;
use ball_simulation::*;
use bevy::time::Stopwatch;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use nalgebra::Vector2;
use std::f32::consts::PI;
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
struct Ball;

#[derive(Component)]
struct Block;

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
    let mut blocks = vec![];

    commands.spawn((SimulationWatch(Stopwatch::new()), OnPlaySimulation));

    for ((y, x), &has_block) in board_state.single().blocks.indexed_iter() {
        if has_block {
            let y = y as f64 / 11.0;
            let x = x as f64 / 11.0;
            blocks.push(ball_simulation::Block {
                min_y: y - 0.04,
                max_y: y + 0.04,
                min_x: x - 0.04,
                max_x: x + 0.04,
            });
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
    add_blocks_from_state(
        &simulation_state,
        &mut block_ids,
        &mut commands,
        &mut meshes,
        &mut materials,
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

fn add_blocks_from_state(
    simulation: &SimulationState,
    block_ids: &mut BlockEntities,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let block_mesh: Mesh2dHandle = meshes
        .add(shape::RegularPolygon::new(2f32.sqrt() / 2.0, 4).into())
        .into();
    for block in &simulation.blocks {
        add_block(commands, block_ids, block_mesh.clone(), materials, *block);
    }
}

fn add_ball(
    commands: &mut Commands,
    ball_ids: &mut BallEntities,
    mesh: Mesh2dHandle,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let id = commands
        .spawn((
            MaterialMesh2dBundle {
                mesh,
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                visibility: Visibility::Hidden,
                transform: Transform::from_xyz(0.0, 0.0, -0.5),
                ..default()
            },
            Ball,
            OnPlaySimulation,
        ))
        .id();
    ball_ids.0.push(id);
}

fn add_block(
    commands: &mut Commands,
    block_ids: &mut BlockEntities,
    mesh: Mesh2dHandle,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    block: ball_simulation::Block,
) {
    let id = commands
        .spawn((
            MaterialMesh2dBundle {
                mesh,
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                transform: Transform::from_xyz(
                    (block.min_x + block.max_x) as f32 / 2.0,
                    (block.min_y + block.max_y) as f32 / 2.0,
                    -0.5,
                )
                .with_rotation(Quat::from_rotation_z(PI / 4.0))
                .with_scale(Vec3::new(
                    (block.max_x - block.min_x) as f32,
                    (block.max_y - block.min_y) as f32,
                    1.0,
                )),
                ..default()
            },
            Block,
            OnPlaySimulation,
        ))
        .id();
    block_ids.0.push(id);
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
                &mut ball_ids,
                ball_mesh.get_single().unwrap().clone(),
                &mut materials,
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
