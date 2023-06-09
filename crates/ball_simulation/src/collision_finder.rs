use crate::{
    collision_times::{earliest_collision_ball_block, earliest_collision_ball_walls},
    EventType,
};
use itertools::Itertools;

use crate::{CollisionData, CollisionType, Event, SimulationState};

impl SimulationState {
    pub(crate) fn earliest_event(
        &mut self,
        custom_events: &[Event<EventType>],
    ) -> Option<Event<EventType>> {
        custom_events
            .iter()
            .copied()
            .chain(
                self.balls
                    .iter()
                    .enumerate()
                    .filter_map(|(ball_index, ball)| {
                        earliest_collision_ball_walls(ball, self.space_width, self.space_height)
                            .map(|Event { time, data }| Event {
                                time,
                                data: CollisionData {
                                    ball: ball_index,
                                    against: CollisionType::Wall(data),
                                },
                            })
                    })
                    .chain(
                        self.balls
                            .iter()
                            .enumerate()
                            .cartesian_product(self.blocks.iter().enumerate())
                            .filter_map(|((ball_index, ball), (block_index, block))| {
                                earliest_collision_ball_block(ball, block).map(
                                    |Event {
                                         time,
                                         data: contact_position,
                                     }| Event {
                                        time,
                                        data: CollisionData {
                                            ball: ball_index,
                                            against: CollisionType::Block {
                                                index: block_index,
                                                contact_position,
                                            },
                                        },
                                    },
                                )
                            }),
                    )
                    .map(|Event { time, data }| Event {
                        time,
                        data: EventType::Collision(data),
                    }),
            )
            .min_by(|a, b| a.time.total_cmp(&b.time))
    }
}
