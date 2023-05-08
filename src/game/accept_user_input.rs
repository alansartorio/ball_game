use bevy::{math::vec2, prelude::*, sprite::MaterialMesh2dBundle};
use nalgebra::Vector2;

use crate::{colors, despawn_screen};

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
            (update, update_indicator.after(update))
                .run_if(in_state(InnerGameState::AcceptUserInput)),
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
struct BlocksParent;

#[derive(Component, Default)]
struct AimIndicator {
    direction: Option<Vector2<f64>>,
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

    commands
        .spawn((
            SpatialBundle {
                visibility: Visibility::Hidden,
                transform: Transform::from_xyz(0.5, 0.0, -0.5),
                ..default()
            },
            AimIndicator::default(),
            OnAcceptUserInput,
        ))
        .with_children(|parent| {
            parent.spawn((MaterialMesh2dBundle {
                mesh: meshes.add(shape::Quad::new(vec2(0.25, 0.02)).into()).into(),
                material: materials.add(ColorMaterial::from(*colors::BLOCKS)),
                transform: Transform::from_xyz(0.125, 0.0, 0.0),
                ..default()
            },));
        });
}

fn update(
    buttons: Res<Input<MouseButton>>,
    touches: Res<Touches>,
    window: Query<&Window>,
    mut inner_game_state: ResMut<NextState<InnerGameState>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut board_state: Query<&mut BoardState>,
    mut aim_indicator: Query<&mut AimIndicator>,
) {
    let touch = touches.iter().map(|e| e.position()).last();
    let mouse = || {
        buttons
            .pressed(MouseButton::Left)
            .then(|| window.single().cursor_position())
            .flatten()
    };
    if let Some(position) = touch.or_else(mouse) {
        let (camera, camera_transform) = camera_q.single();
        let world_position = camera
            .viewport_to_world(camera_transform, position)
            .unwrap()
            .origin
            .truncate();

        if world_position.y > 0.0 {
            let start = Vector2::new(board_state.single().launcher_position, 0.0);
            let end = Vector2::new(world_position.x, world_position.y).cast();

            let delta = (end - start).normalize();

            aim_indicator.single_mut().direction = Some(delta);
        }
    }

    if touches.any_just_released() || buttons.just_released(MouseButton::Left) {
        if let Some(direction) = aim_indicator.single().direction {
            board_state.single_mut().direction = direction;
            inner_game_state.set(InnerGameState::PlaySimulation);
        }
    }
}

fn update_indicator(mut aim_indicator: Query<(&AimIndicator, &mut Transform, &mut Visibility)>) {
    if let Some(direction) = aim_indicator.single().0.direction {
        let angle = f64::atan2(direction.y, direction.x) as f32;
        aim_indicator.single_mut().1.rotation = Quat::from_rotation_z(angle);
        *aim_indicator.single_mut().2 = Visibility::Visible;
    } else {
        *aim_indicator.single_mut().2 = Visibility::Hidden;
    }
}
