mod collision_finder;
mod collision_primitives;
mod collision_times;
use nalgebra::Vector2;

#[derive(Debug, Clone, Copy)]
pub struct Ball {
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
    pub radius: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub max_y: f64,
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
}

impl Block {
    pub fn new(top: f64, bottom: f64, left: f64, right: f64) -> Self {
        Self {
            max_y: top,
            min_y: bottom,
            min_x: left,
            max_x: right,
        }
    }
}

#[derive(Debug, Clone)]
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
        self.time += time;
    }

    pub fn next(mut self) -> Option<(SimulationState, Event<EventType>)> {
        self.earliest_event().map(|event| {
            let Event {
                time,
                data: event_data,
            } = event;

            self.forward(time);

            //println!("{:?}", event);

            match event_data {
                EventType::Collision(CollisionData { ball, against }) => {
                    match against {
                        CollisionType::Wall(wall_type) => match wall_type {
                            WallType::YPositive | WallType::YNegative => {
                                self.balls[ball].velocity.y *= -1.0
                            }
                            WallType::XNegative | WallType::XPositive => {
                                self.balls[ball].velocity.x *= -1.0
                            }
                        },
                        CollisionType::Block {
                            contact_position, ..
                        } => {
                            //let block = &self.blocks[index];
                            let ball = &mut self.balls[ball];
                            let contact_normal = (contact_position - ball.position).normalize();
                            let normal_velocity =
                                ball.velocity.dot(&contact_normal) * contact_normal;
                            let previous = ball.velocity;
                            ball.velocity -= 2.0 * normal_velocity;
                            //println!("before: {:?} | after: {:?}", previous, ball.velocity);
                        }
                    }
                }
                EventType::Spawn => self.balls.push(Ball {
                    position: Vector2::new(0.2, 0.2),
                    velocity: Vector2::new(0.2, 0.00313) * 10.0,
                    radius: 0.01,
                }),
            }

            (self, event)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event<T> {
    pub time: f64,
    pub data: T,
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
    YPositive,
    YNegative,
    XNegative,
    XPositive,
}

#[derive(Debug, Clone, Copy)]
pub enum CollisionType {
    Wall(WallType),
    Block {
        index: usize,
        contact_position: Vector2<f64>,
    },
}
