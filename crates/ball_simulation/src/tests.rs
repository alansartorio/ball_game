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

#[test]
fn test_crash() {
    let simulation_state = SimulationState {
        time: 3.091881033704481,
        space_width: 1.0,
        space_height: 1.0,
        balls: vec![Ball {
            position: Vector2::new(0.15366112818873037, 0.4829665403837622),
            velocity: Vector2::new(-0.45658153289741865, -1.9471859962050475),
            radius: 0.02,
        }],
        blocks: vec![Block {
            max_y: 0.4945454545454545,
            min_x: 0.05090909090909091,
            max_x: 0.13090909090909092,
            min_y: 0.41454545454545455,
        }],
    };

    simulation_state.next(&[]).unwrap().0.next(&[]).unwrap();
}

#[test]
fn test_crash2() {
    let mut simulation_state = SimulationState {
        time: 0.4958926321538045,
        space_width: 1.0,
        space_height: 1.0,
        balls: vec![Ball {
            position: Vector2::new(0.98, 0.45073156690401917),
            velocity: Vector2::new(-1.4579651981269763, 1.3690644546735433),
            radius: 0.02,
        }],
        blocks: vec![Block {
            max_y: 0.9490909090909091,
            min_x: 0.5054545454545454,
            max_x: 0.5854545454545454,
            min_y: 0.869090909090909,
        }],
    };

    #[cfg(feature = "svg")]
    use svg::Document;

    #[cfg(feature = "svg")]
    let mut document = Document::new().set("viewBox", (0, 0, 100, 10));

    simulation_state = simulation_state.next(&[]).unwrap().0;

    #[cfg(feature = "svg")]
    {
        document = simulation_state.save_img(document, 0);
    }

    simulation_state = simulation_state.next(&[]).unwrap().0;

    #[cfg(feature = "svg")]
    {
        document = simulation_state.save_img(document, 1);
    }

    assert!(simulation_state.balls[0].velocity.y < 0.0);

    #[cfg(feature = "svg")]
    svg::save("dbg.svg", &document).unwrap();
}

#[test]
#[cfg(feature = "svg")]
fn test_close_bounce() {
    let mut simulation_state = SimulationState {
        time: 0.6504021566055405,
        space_width: 1.0,
        space_height: 1.0,
        balls: vec![Ball {
            position: Vector2::new(0.98, 0.8400000576819083),
            velocity: Vector2::new(-0.9922778253475968, 1.736486313600958),
            radius: 0.02,
        }],
        blocks: vec![Block {
            max_y: 0.9490909090909091,
            min_x: 0.869090909090909,
            max_x: 0.9490909090909091,
            min_y: 0.869090909090909,
        }],
    };

    use svg::Document;

    let mut document = Document::new().set("viewBox", (0, 0, 51 * 11, 10));

    document = simulation_state.save_img(document, 0);

    for i in 1..=50 {
        simulation_state = simulation_state.next(&[]).unwrap().0;

        document = simulation_state.save_img(document, i);
    }

    svg::save("dbg.svg", &document).unwrap();
}
