use crate::collision_times::{earliest_collision_ball_block, earliest_collision_ball_walls};
use itertools::Itertools;

use crate::{Collision, CollisionData, CollisionType, SimulationState};

impl SimulationState {
    pub(crate) fn earliest_collision<'a>(&mut self) -> Option<Collision<CollisionData>> {
        self.balls
            .iter()
            .enumerate()
            .filter_map(|(ball_index, ball)| {
                earliest_collision_ball_walls(ball, self.space_width, self.space_height).map(
                    |Collision { time, data }| Collision {
                        time,
                        data: CollisionData {
                            ball: ball_index,
                            against: CollisionType::Wall(data),
                        },
                    },
                )
            })
            .chain(
                self.balls
                    .iter()
                    .enumerate()
                    .cartesian_product(self.blocks.iter().enumerate())
                    .filter_map(|((ball_index, ball), (block_index, block))| {
                        earliest_collision_ball_block(ball, block).map(
                            |Collision { time, .. }| Collision {
                                time,
                                data: CollisionData {
                                    ball: ball_index,
                                    against: CollisionType::Block(block_index),
                                },
                            },
                        )
                    }),
            )
            .min_by(|a, b| a.time.total_cmp(&b.time))
    }
}
