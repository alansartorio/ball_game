use std::{assert_matches::assert_matches, f64::consts::PI};

use nalgebra::Vector2;

use crate::{Ball, Block, Event, EventType, SimulationState};

#[test]
fn single_block_from_all_sides() {
    let simulation_state = SimulationState {
        time: 0.0,
        space_width: 10.0,
        space_height: 10.0,
        blocks: vec![Block {
            min_x: 4.0,
            max_x: 6.0,
            min_y: 4.0,
            max_y: 6.0,
        }],
        balls: vec![],
    };

    let add_ball_with_angle = |mut simulation_state: SimulationState, angle: f64| {
        let center = Vector2::new(5.0, 5.0);
        let position = center + Vector2::new(angle.cos(), angle.sin()) * 3.0;
        let velocity = center - position;

        simulation_state.balls = vec![Ball {
            position,
            velocity,
            radius: 1.0,
        }];
        simulation_state
    };

    for angle in (0..360).map(|a| a as f64 * PI / 180.0) {
        //eprintln!("{angle}");
        assert_matches!(
            add_ball_with_angle(simulation_state.clone(), angle).next(&[]),
            Some((
                _,
                Event {
                    data: EventType::Collision(crate::CollisionData {
                        ball: 0,
                        against: crate::CollisionType::Block { index: 0, .. },
                    }),
                    ..
                }
            ))
        );
    }
}
