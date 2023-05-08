use std::f32::consts::PI;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use nalgebra::Vector2;

use crate::colors;

pub(crate) fn get_block_separations(columns: usize, rows: usize) -> Vector2<f64> {
    let y = 1.0 / (rows as f64 + 1.0);
    let x = 1.0 / (columns as f64 + 1.0);
    Vector2::new(x, y)
}

pub(crate) fn get_block(columns: usize, rows: usize, x: usize, y: usize) -> ball_simulation::Block {
    let y = (1.0 + y as f64) / (rows as f64 + 1.0);
    let x = (1.0 + x as f64) / (columns as f64 + 1.0);
    ball_simulation::Block {
        min_y: y - 0.04,
        max_y: y + 0.04,
        min_x: x - 0.04,
        max_x: x + 0.04,
    }
}

#[derive(Component)]
pub(crate) struct Lives(pub usize);

#[derive(Component)]
pub(crate) struct Ball;

#[derive(Component)]
pub(crate) struct Block;

fn add_block(
    commands: &mut Commands,
    block_ids: &mut Vec<Entity>,
    mesh: Mesh2dHandle,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    block: ball_simulation::Block,
    lives: usize,
    parent: Entity,
) -> Entity {
    let id = commands
        .spawn((
            MaterialMesh2dBundle {
                mesh,
                material: materials.add(ColorMaterial::from(*colors::BLOCKS)),
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
            Lives(lives),
            Block,
        ))
        .id();
    commands.entity(parent).push_children(&[id]);
    block_ids.push(id);
    id
}

pub(crate) fn add_blocks_from_state(
    blocks: &[(ball_simulation::Block, usize)],
    block_ids: &mut Vec<Entity>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    parent: Entity,
) -> Vec<Entity> {
    let block_mesh: Mesh2dHandle = meshes
        .add(shape::RegularPolygon::new(2f32.sqrt() / 2.0, 4).into())
        .into();

    blocks
        .iter()
        .map(|&(block, lives)| {
            add_block(
                commands,
                block_ids,
                block_mesh.clone(),
                materials,
                block,
                lives,
                parent,
            )
        })
        .collect()
}

pub(crate) fn add_ball<State: Component>(
    commands: &mut Commands,
    ball_ids: &mut Vec<Entity>,
    mesh: Mesh2dHandle,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    state_value: State,
) {
    let id = commands
        .spawn((
            MaterialMesh2dBundle {
                mesh,
                material: materials.add(ColorMaterial::from(*colors::BALLS)),
                visibility: Visibility::Hidden,
                transform: Transform::from_xyz(0.0, 0.0, -0.5),
                ..default()
            },
            Ball,
            state_value,
        ))
        .id();
    ball_ids.push(id);
}
