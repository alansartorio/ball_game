#![feature(let_chains)]
#![feature(option_as_slice)]
use nalgebra::Vector2;
use std::f32::consts::PI;
use std::ops::{Add, Div, Mul};

use ball_interpolator::*;
use ball_simulation::*;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

#[derive(Resource)]
struct Simulation {
    state: SimulationState,
    next: Option<(SimulationState, ball_simulation::Event<EventType>)>,
}

#[derive(Resource)]
struct InterpolatedSimulation {
    state: SimulationState,
}

#[derive(Resource, Default)]
struct BlockEntities(Vec<Entity>);

#[derive(Resource, Default)]
struct BallEntities(Vec<Entity>);

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Block;

#[derive(Component)]
struct BallMesh;

fn add_simulation_state(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut blocks = vec![];

    for y in 6..=10 {
        let y = y as f64 / 11.0;
        for x in 1..=10 {
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
    commands.insert_resource(Simulation {
        state: simulation_state.clone(),
        next: None,
    });
    let mut block_ids = BlockEntities::default();
    add_blocks_from_state(
        &simulation_state,
        &mut block_ids,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    commands.insert_resource(block_ids);
    let ball_ids = BallEntities::default();
    commands.insert_resource(ball_ids);
    commands.insert_resource(InterpolatedSimulation {
        state: simulation_state,
    });
    commands.spawn((
        Mesh2dHandle::from(meshes.add(shape::Circle::new(1.).into())),
        BallMesh,
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

fn initialize_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle {
        transform: Transform::from_xyz(0.5, 0.5, 0.0),
        ..Default::default()
    };
    camera.projection.scale = 0.002;
    commands.spawn(camera);
}

fn add_ball(
    commands: &mut Commands,
    ball_ids: &mut ResMut<BallEntities>,
    mesh: Mesh2dHandle,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let id = commands
        .spawn((
            MaterialMesh2dBundle {
                mesh,
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                visibility: Visibility::Hidden,
                ..default()
            },
            Ball,
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
                    0.0,
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
        ))
        .id();
    block_ids.0.push(id);
}

fn update_simulation(
    time: Res<Time>,
    mut simulation: ResMut<Simulation>,
    mut ball_ids: ResMut<BallEntities>,
    mut block_ids: ResMut<BlockEntities>,
    mut interpolated_simulation: ResMut<InterpolatedSimulation>,
    mut commands: Commands,
    ball_mesh: Query<&Mesh2dHandle, With<BallMesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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
    } && time.elapsed_seconds() as f64 >= next_state.time {
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
    time: Res<Time>,
    simulation: ResMut<Simulation>,
    mut interpolated_simulation: ResMut<InterpolatedSimulation>,
) {
    let advance = time.elapsed_seconds() as f64 - interpolated_simulation.state.time;
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
    simulation: Res<InterpolatedSimulation>,
    mut balls: Query<(&mut Transform, &mut Visibility), With<Ball>>,
) {
    for ((mut ball_entity, mut visibility), ball) in balls.iter_mut().zip(&simulation.state.balls) {
        ball_entity.translation = Vec3::new(ball.position.x as f32, ball.position.y as f32, 0.0);
        ball_entity.scale = Vec2::new(ball.radius as f32, ball.radius as f32).extend(1.0);
        *visibility = Visibility::Visible;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(add_simulation_state)
        .add_startup_system(initialize_camera)
        .add_system(update_simulation)
        .add_system(interpolate_simulation.after(update_simulation))
        .add_system(update_balls.after(interpolate_simulation))
        .run();
}
