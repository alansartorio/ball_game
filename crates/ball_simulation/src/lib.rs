mod collision_finder;
mod collision_primitives;
mod collision_times;
use nalgebra::Vector2;

#[derive(Clone, Copy)]
pub struct Ball {
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
    pub radius: f64,
}

pub struct Block {
    pub top: f64,
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
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

pub struct SimulationState {
    space_width: f64,
    space_height: f64,
    balls: Vec<Ball>,
    blocks: Vec<Block>,
}

impl SimulationState {
    fn forward(&mut self, time: f64) {
        for ball in self.balls.iter_mut() {
            ball.position += ball.velocity * time;
        }
    }

    fn next(mut self) -> Option<(SimulationState, Collision<CollisionData>)> {
        self.earliest_collision().map(|collision| {
            let Collision {
                time,
                data: CollisionData { ball, against },
            } = collision;

            self.forward(time);

            match against {
                CollisionType::Wall(wall_type) => match wall_type {
                    WallType::Horizontal => self.balls[ball].velocity.y *= -1.0,
                    WallType::Vertical => self.balls[ball].velocity.x *= -1.0,
                },
                CollisionType::Block(block) => {
                    //let block = self.blocks[block];
                    //let
                }
            }

            (self, collision)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Collision<T> {
    time: f64,
    data: T,
}

#[derive(Debug, Clone, Copy)]
pub struct CollisionData {
    pub ball: usize,
    pub against: CollisionType,
}

#[derive(Debug, Clone, Copy)]
pub enum WallType {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub enum CollisionType {
    Wall(WallType),
    Block(usize),
}
