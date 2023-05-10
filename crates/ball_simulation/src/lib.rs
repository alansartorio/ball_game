#![feature(assert_matches)]
mod collision_finder;
mod collision_primitives;
mod collision_times;
#[cfg(test)]
mod tests;
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

    pub fn next(
        mut self,
        custom_events: &[Event<EventType>],
    ) -> Option<(SimulationState, Event<EventType>)> {
        self.earliest_event(custom_events).map(|event| {
            let Event {
                time,
                data: event_data,
            } = event;

            self.forward(time);

            //println!("{:?}", event);

            match event_data {
                EventType::Collision(CollisionData { ball, against }) => match against {
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
                        let ball = &mut self.balls[ball];
                        let contact_normal = (contact_position - ball.position).normalize();
                        let normal_velocity = ball.velocity.dot(&contact_normal) * contact_normal;
                        ball.velocity -= 2.0 * normal_velocity;
                    }
                },
                EventType::Custom => {}
            }

            (self, event)
        })
    }
}

#[cfg(feature = "svg")]
mod svg_mod {
    use super::SimulationState;
    use svg::{
        node::element::{path::Data, Circle, Group, Path, Rectangle},
        Document,
    };
    impl SimulationState {
        pub fn save_img(&self, document: Document, page: usize) -> Document {
            let mut group = Group::new().set("inkscape:groupmode", "layer").set(
                "transform",
                format!("matrix(10, 0, 0, -10, {}, 10)", page * 11),
            );

            group = group.add(
                Rectangle::new()
                    .set("fill", "none")
                    .set("stroke", "black")
                    .set("stroke-width", 0.001)
                    .set("x", 0)
                    .set("y", 0)
                    .set("width", 1)
                    .set("height", 1),
            );

            for ball in &self.balls {
                group = group
                    .add(
                        Circle::new()
                            .set("fill", "#ff681d")
                            .set("stroke", "none")
                            .set("r", ball.radius)
                            .set("cx", ball.position.x)
                            .set("cy", ball.position.y),
                    )
                    .add(
                        Path::new()
                            .set("fill", "none")
                            .set("stroke", "black")
                            .set("stroke-width", 0.005)
                            .set(
                                "d",
                                Data::new()
                                    .move_to((ball.position.x, ball.position.y))
                                    .line_by((ball.velocity.x * 0.02, ball.velocity.y * 0.02)),
                            ),
                    );
            }

            for (i, block) in self.blocks.iter().enumerate() {
                group = group.add(
                    Rectangle::new()
                        .set("fill", "#1a5fb4")
                        .set("stroke", "none")
                        .set("x", block.min_x)
                        .set("y", block.min_y)
                        .set("width", block.max_x - block.min_x)
                        .set("height", block.max_y - block.min_y)
                        .set("id", i),
                );
            }

            document.add(group)
        }
    }
}
#[cfg(feature = "svg")]
pub use svg_mod::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event<T> {
    pub time: f64,
    pub data: T,
}

#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Collision(CollisionData),
    Custom,
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
