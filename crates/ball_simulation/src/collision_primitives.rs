use approx::AbsDiffEq;
use nalgebra::Vector2;

use crate::{Ball, Event};

pub(crate) fn ball_ball(b1: &Ball, b2: &Ball) -> Option<f64> {
    let delta_v = b2.velocity - b1.velocity;
    let delta_r = b2.position - b1.position;
    let sigma = b1.radius + b2.radius;
    let d = (delta_v.dot(&delta_r).powi(2))
        - delta_v.dot(&delta_v) * (delta_r.dot(&delta_r) - sigma.powi(2));

    (delta_v.dot(&delta_r) < 0.0 && d >= 0.0)
        .then(|| -(delta_v.dot(&delta_r) + d.sqrt()) / (delta_v.dot(&delta_v)))
}

fn ball_point(ball: &Ball, point: Vector2<f64>) -> Option<f64> {
    ball_ball(
        ball,
        &Ball {
            position: point,
            velocity: Vector2::zeros(),
            radius: 0.0,
        },
    )
}

fn segment_ball_alpha(segment_a: Vector2<f64>, segment_b: Vector2<f64>, ball: &Ball) -> f64 {
    let ab = segment_b - segment_a;

    (ball.velocity.perp(&(ball.position - segment_a))
        + ball.radius * ab.normalize().dot(&ball.velocity))
        / ball.velocity.perp(&ab)
}

fn segment_ball_time(
    segment_a: Vector2<f64>,
    segment_b: Vector2<f64>,
    ball: &Ball,
    alpha: f64,
) -> f64 {
    let ab = segment_b - segment_a;

    if ball.velocity.x.abs() > ball.velocity.y.abs() {
        (segment_a.x * (1.0 - alpha) + segment_b.x * alpha - ball.position.x
            + ball.radius / ab.magnitude() * (segment_b.y - segment_a.y))
            / ball.velocity.x
    } else {
        (segment_a.y * (1.0 - alpha) + segment_b.y * alpha - ball.position.y
            + ball.radius / ab.magnitude() * (segment_a.x - segment_b.x))
            / ball.velocity.y
    }
}

const CLEARANCE: f64 = 0.0001;

// Calculates collision point and time between a moving ball and a segment from segment_a to segment_b
pub(crate) fn segment_ball(
    segment_a: Vector2<f64>,
    segment_b: Vector2<f64>,
    ball: &Ball,
) -> Option<Event<Vector2<f64>>> {
    let ab = segment_b - segment_a;
    let ab_mag = ab.magnitude();
    let signed_distance = (ball.position - segment_a).perp(&ab) / ab_mag;
    let normal_velocity = ball.velocity.perp(&ab) / ab_mag;
    if signed_distance < -CLEARANCE {
        None
    } else if signed_distance < ball.radius {
        let ab_proj = (ball.position - segment_a).dot(&ab) / ab_mag.powi(2);
        if ab_proj < 0.0 {
            ball_point(ball, segment_a).map(|time| Event {
                time,
                data: segment_a,
            })
        } else if ab_proj > 1.0 {
            ball_point(ball, segment_b).map(|time| Event {
                time,
                data: segment_b,
            })
        } else if signed_distance > ball.radius - CLEARANCE {
            if normal_velocity >= 0.0 {
                None
            } else {
                Some(Event {
                    time: 0.0,
                    data: segment_a + ab_proj * ab,
                })
            }
        } else {
            panic!("already in collision");
        }
    } else {
        let alpha = segment_ball_alpha(segment_a, segment_b, ball);
        if alpha < 0.0 {
            ball_point(ball, segment_a).map(|time| Event {
                time,
                data: segment_a,
            })
        } else if alpha > 1.0 {
            ball_point(ball, segment_b).map(|time| Event {
                time,
                data: segment_b,
            })
        } else if normal_velocity >= 0.0 {
            None
        } else {
            let time = segment_ball_time(segment_a, segment_b, ball, alpha);
            Some(Event {
                time,
                data: segment_a + alpha * (segment_b - segment_a),
            })
        }
    }
    .filter(|event| event.time >= 0.0)
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use nalgebra::Vector2;

    use crate::{
        collision_primitives::{segment_ball, segment_ball_time},
        Ball, Event,
    };

    use super::segment_ball_alpha;

    #[test]
    fn test_segment_ball_inside() {
        let segment_a = Vector2::new(2.0, -1.0);
        let segment_b = Vector2::new(6.0, -1.0);
        let ball = Ball {
            position: Vector2::new(8.0, -4.0),
            velocity: Vector2::new(-1.0, 1.0),
            radius: 0.5,
        };

        assert_eq!(segment_ball_alpha(segment_a, segment_b, &ball), 0.875);

        assert_eq!(segment_ball_time(segment_a, segment_b, &ball, 0.875), 2.5);

        let collision = segment_ball(segment_a, segment_b, &ball);

        assert!(collision.is_some());
        assert_eq!(
            collision.unwrap(),
            Event {
                time: 2.5,
                data: Vector2::new(5.5, -1.0),
            }
        );
    }

    #[test]
    fn test_segment_ball_outside() {
        let segment_a = Vector2::new(1.0, 3.0);
        let segment_b = Vector2::new(3.0, 5.0);
        let ball1 = Ball {
            position: Vector2::new(1.0, 1.0),
            velocity: Vector2::new(0.0, 1.0),
            radius: 0.5,
        };
        let ball2 = Ball {
            position: Vector2::new(6.0, 3.0),
            velocity: Vector2::new(-1.0, 1.0),
            radius: 1.0,
        };

        let collision1 = segment_ball(segment_a, segment_b, &ball1);

        assert!(collision1.is_some());
        assert_eq!(
            collision1.unwrap(),
            Event {
                time: 1.5,
                data: segment_a,
            }
        );

        let collision2 = segment_ball(segment_a, segment_b, &ball2);

        assert!(collision2.is_some());
        assert_eq!(collision2.unwrap().data, segment_b);
    }

    #[test]
    fn test_parallel_close() {
        let segment_a = Vector2::new(0.0, 0.0);
        let segment_b = Vector2::new(2.0, 0.0);
        let ball = Ball {
            position: Vector2::new(-2.0, -0.5),
            velocity: Vector2::new(1.0, 0.0),
            radius: 1.0,
        };

        let collision = segment_ball(segment_a, segment_b, &ball);

        assert!(collision.is_some());

        assert_relative_eq!(collision.unwrap().data, segment_a);
    }
}
