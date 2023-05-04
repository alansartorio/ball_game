mod collision_times;
use collision_times::{earliest_collision_ball_block, earliest_collision_ball_walls};
use itertools::Itertools;
use nalgebra::Vector2;

type ID = usize;

pub struct Ball {
    position: Vector2<f64>,
    velocity: Vector2<f64>,
    radius: f64,
}

pub struct Block {
    top: f64,
    left: f64,
    right: f64,
    bottom: f64,
}

impl Block {
    pub fn new(top: f64, bottom: f64, left: f64, right: f64) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }
}

pub struct SimulationInput<'a> {
    space_width: f64,
    space_height: f64,
    balls: &'a [Ball],
    blocks: &'a [Block],
}

pub struct Collision<T> {
    time: f64,
    data: T,
}

pub struct CollisionData<'a> {
    ball: &'a Ball,
    against: CollisionType<'a>,
}

pub enum WallType {
    Horizontal,
    Vertical,
}

pub enum CollisionType<'a> {
    Wall(WallType),
    Block(&'a Block),
}

impl<'a> Iterator for SimulationInput<'a> {
    type Item = Collision<CollisionData<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.balls
            .iter()
            .filter_map(|ball| {
                earliest_collision_ball_walls(ball, self.space_width, self.space_height).map(
                    |Collision { time, data }| Collision {
                        time,
                        data: CollisionData {
                            ball,
                            against: CollisionType::Wall(data),
                        },
                    },
                )
            })
            .chain(
                self.balls
                    .iter()
                    .cartesian_product(self.blocks.iter())
                    .filter_map(|(ball, block)| {
                        earliest_collision_ball_block(ball, block).map(
                            |Collision { time, .. }| Collision {
                                time,
                                data: CollisionData {
                                    ball,
                                    against: CollisionType::Block(block),
                                },
                            },
                        )
                    }),
            )
            .min_by(|a, b| a.time.total_cmp(&b.time))
    }
}
