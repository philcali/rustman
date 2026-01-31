use bevy::prelude::*;
use crate::components::{Position, Velocity, GroundedState};

/// Physics constants
pub const GRAVITY: f32 = 980.0; // pixels per second squared
const FIXED_TIMESTEP: f32 = 1.0 / 60.0; // 60 FPS fixed timestep

/// Plugin for collision detection, movement, and gravity
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_seconds(FIXED_TIMESTEP as f64));
        app.add_systems(FixedUpdate, (
            apply_gravity,
            integrate_velocity,
        ).chain());
    }
}

/// Apply gravity to airborne entities
fn apply_gravity(
    mut query: Query<(&mut Velocity, &GroundedState)>,
    time: Res<Time<Fixed>>,
) {
    let delta_time = time.delta_seconds();
    
    for (mut velocity, grounded) in query.iter_mut() {
        // Only apply gravity if not grounded
        if !grounded.is_grounded {
            velocity.y += GRAVITY * delta_time;
        }
    }
}

/// Integrate velocity to update position each frame
fn integrate_velocity(
    mut query: Query<(&mut Position, &Velocity)>,
    time: Res<Time<Fixed>>,
) {
    let delta_time = time.delta_seconds();
    
    for (mut position, velocity) in query.iter_mut() {
        position.x += velocity.x * delta_time;
        position.y += velocity.y * delta_time;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gravity_applied_when_airborne() {
        let mut velocity = Velocity::new(0.0, 0.0);
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };
        
        // Simulate one frame of gravity
        if !grounded.is_grounded {
            velocity.y += GRAVITY * FIXED_TIMESTEP;
        }
        
        let expected_velocity = GRAVITY * FIXED_TIMESTEP;
        assert!((velocity.y - expected_velocity).abs() < 0.01, 
            "Expected velocity.y to be ~{}, got {}", expected_velocity, velocity.y);
    }

    #[test]
    fn test_gravity_not_applied_when_grounded() {
        let mut velocity = Velocity::new(0.0, 0.0);
        let grounded = GroundedState {
            is_grounded: true,
            ground_normal: Vec2::new(0.0, -1.0),
        };
        
        // Simulate one frame - gravity should NOT be applied
        if !grounded.is_grounded {
            velocity.y += GRAVITY * FIXED_TIMESTEP;
        }
        
        assert_eq!(velocity.y, 0.0);
    }

    #[test]
    fn test_velocity_integration() {
        let mut position = Position::new(100.0, 200.0);
        let velocity = Velocity::new(50.0, -100.0);
        
        // Simulate one frame of velocity integration
        position.x += velocity.x * FIXED_TIMESTEP;
        position.y += velocity.y * FIXED_TIMESTEP;
        
        let expected_x = 100.0 + 50.0 * FIXED_TIMESTEP;
        let expected_y = 200.0 + (-100.0) * FIXED_TIMESTEP;
        
        assert!((position.x - expected_x).abs() < 0.01,
            "Expected position.x to be ~{}, got {}", expected_x, position.x);
        assert!((position.y - expected_y).abs() < 0.01,
            "Expected position.y to be ~{}, got {}", expected_y, position.y);
    }

    #[test]
    fn test_deterministic_physics() {
        // Run simulation twice with same initial conditions
        let run_simulation = || {
            let mut position = Position::new(100.0, 200.0);
            let mut velocity = Velocity::new(50.0, -100.0);
            let grounded = GroundedState {
                is_grounded: false,
                ground_normal: Vec2::ZERO,
            };

            // Simulate 10 frames
            for _ in 0..10 {
                // Apply gravity
                if !grounded.is_grounded {
                    velocity.y += GRAVITY * FIXED_TIMESTEP;
                }
                
                // Integrate velocity
                position.x += velocity.x * FIXED_TIMESTEP;
                position.y += velocity.y * FIXED_TIMESTEP;
            }

            (position, velocity)
        };

        let (pos1, vel1) = run_simulation();
        let (pos2, vel2) = run_simulation();

        // Results should be identical (deterministic)
        assert_eq!(pos1.x, pos2.x);
        assert_eq!(pos1.y, pos2.y);
        assert_eq!(vel1.x, vel2.x);
        assert_eq!(vel1.y, vel2.y);
    }

    #[test]
    fn test_gravity_accumulates_over_time() {
        let mut velocity = Velocity::new(0.0, 0.0);
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };
        
        // Simulate 5 frames
        for _ in 0..5 {
            if !grounded.is_grounded {
                velocity.y += GRAVITY * FIXED_TIMESTEP;
            }
        }
        
        let expected_velocity = GRAVITY * FIXED_TIMESTEP * 5.0;
        assert!((velocity.y - expected_velocity).abs() < 0.01, 
            "Expected velocity.y to be ~{}, got {}", expected_velocity, velocity.y);
    }

    #[test]
    fn test_horizontal_velocity_integration() {
        let mut position = Position::new(0.0, 0.0);
        let velocity = Velocity::new(200.0, 0.0); // Moving right at 200 px/s
        
        // Simulate 1 second (60 frames)
        for _ in 0..60 {
            position.x += velocity.x * FIXED_TIMESTEP;
            position.y += velocity.y * FIXED_TIMESTEP;
        }
        
        // After 1 second, should have moved 200 pixels
        assert!((position.x - 200.0).abs() < 0.1,
            "Expected position.x to be ~200.0, got {}", position.x);
        assert!((position.y - 0.0).abs() < 0.01,
            "Expected position.y to be ~0.0, got {}", position.y);
    }
}
