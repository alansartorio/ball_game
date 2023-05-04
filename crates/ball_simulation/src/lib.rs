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
    pub time: f64,
    pub space_width: f64,
    pub space_height: f64,
    pub balls: Vec<Ball>,
    pub blocks: Vec<Block>,
}

impl SimulationState {
    fn forward(&mut self, time: f64) {
        for ball in self.balls.iter_mut() {
            ball.position += ball.velocity * time;
        }
    }

    pub fn next(mut self) -> Option<(SimulationState, Event<EventType>)> {
        self.earliest_event().map(|event| {
            let Event {
                time,
                data: event_data,
            } = event;

            self.forward(time);

            match event_data {
                EventType::Collision(CollisionData { ball, against }) => {
                    match against {
                        CollisionType::Wall(wall_type) => match wall_type {
                            WallType::Horizontal => self.balls[ball].velocity.y *= -1.0,
                            WallType::Vertical => self.balls[ball].velocity.x *= -1.0,
                        },
                        CollisionType::Block {
                            contact_position, ..
                        } => {
                            //let block = &self.blocks[index];
                            let ball = &mut self.balls[ball];
                            let contact_normal = (contact_position - ball.position).normalize();
                            let normal_velocity =
                                ball.velocity.dot(&contact_normal) * contact_normal;
                            ball.velocity -= 2.0 * normal_velocity;
                        }
                    }
                }
                EventType::Spawn => self.balls.push(Ball {
                    position: Vector2::new(0.0, 0.0),
                    velocity: Vector2::new(1.0, 0.0),
                    radius: 1.0,
                }),
            }

            (self, event)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event<T> {
    time: f64,
    data: T,
}

#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Collision(CollisionData),
    Spawn,
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
    Block {
        index: usize,
        contact_position: Vector2<f64>,
    },
}
