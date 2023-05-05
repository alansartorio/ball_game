use ball_interpolator::*;
use ball_simulation::*;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Resource)]
struct Simulation {
    state: SimulationState,
    next: Option<(SimulationState, ball_simulation::Event<EventType>)>,
}

#[derive(Resource)]
struct InterpolatedSimulation {
    state: SimulationState,
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Block;

fn add_simulation_state(mut commands: Commands) {
    let simulation_state = SimulationState {
        time: 0.0,
        space_width: 1.0,
        space_height: 1.0,
        balls: vec![],
        blocks: vec![ball_simulation::Block {
            min_y: 0.8,
            max_y: 0.9,
            min_x: 0.8,
            max_x: 0.9,
        }],
    };
    commands.insert_resource(Simulation {
        state: simulation_state.clone(),
        next: None,
    });
    commands.insert_resource(InterpolatedSimulation {
        state: simulation_state,
    });
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
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(1.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            visibility: Visibility::Hidden,
            ..default()
        },
        Ball,
    ));
}

fn update_simulation(
    time: Res<Time>,
    mut simulation: ResMut<Simulation>,
    mut interpolated_simulation: ResMut<InterpolatedSimulation>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    balls: Query<Entity, With<Ball>>,
) {
    if simulation.next.is_none() {
        simulation.next = simulation.state.clone().next();
    }
    while simulation
        .next
        .as_ref()
        .is_some_and(|(next_state, _)| time.elapsed_seconds() > next_state.time as f32)
    {
        let (next_state, next_event) = simulation.next.as_ref().unwrap().clone();
        simulation.state = next_state;
        simulation.next = None;
        interpolated_simulation.state = simulation.state.clone();
        //println!("Time: {} | Event: {:?}", simulation.state.time, event);

        if let EventType::Spawn = next_event.data {
            add_ball(&mut commands, &mut meshes, &mut materials);
        } else if let EventType::Collision(CollisionData {
            ball,
            against: CollisionType::Wall(WallType::YNegative),
        }) = next_event.data
        {
            commands.entity(balls.iter().next().unwrap()).despawn();
            simulation.state.balls.remove(ball);
        }
    }
}

fn interpolate_simulation(
    time: Res<Time>,
    simulation: ResMut<Simulation>,
    mut interpolated_simulation: ResMut<InterpolatedSimulation>,
) {
    let advance = time.elapsed_seconds() as f64 - simulation.state.time;
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
