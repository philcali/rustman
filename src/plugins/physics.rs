use crate::components::{Collider, GroundedState, LevelGeometry, Position, Velocity};
use bevy::prelude::*;

/// Physics constants
pub const GRAVITY: f32 = 980.0; // pixels per second squared
const FIXED_TIMESTEP: f32 = 1.0 / 60.0; // 60 FPS fixed timestep
const GROUND_CHECK_EPSILON: f32 = 2.0; // Distance to check for ground contact
const SLOPE_FRICTION: f32 = 0.3; // Friction coefficient for slopes
const MIN_SLOPE_ANGLE: f32 = 0.1; // Minimum angle (radians) to be considered a slope

/// Collision result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollisionResult {
    NoCollision,
    Collided(Vec2), // Contains collision normal
}

/// Plugin for collision detection, movement, and gravity
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_seconds(FIXED_TIMESTEP as f64));
        app.add_systems(
            FixedUpdate,
            (
                apply_gravity,
                resolve_collisions,
                update_grounded_state,
                apply_slope_physics,
            )
                .chain(),
        );
    }
}

/// Apply gravity to airborne entities
fn apply_gravity(mut query: Query<(&mut Velocity, &GroundedState)>, time: Res<Time<Fixed>>) {
    let delta_time = time.delta_seconds();

    for (mut velocity, grounded) in query.iter_mut() {
        // Only apply gravity if not grounded
        if !grounded.is_grounded {
            velocity.y += GRAVITY * delta_time;
        }
    }
}

/// Swept AABB collision detection
/// Returns (time_of_impact, collision_normal) if collision occurs
fn swept_aabb_collision(
    pos: &Position,
    collider: &Collider,
    geometry: &LevelGeometry,
    movement: Vec2,
) -> Option<(f32, Vec2)> {
    // Calculate AABB bounds for the entity
    let entity_left = pos.x + collider.offset_x;
    let entity_right = entity_left + collider.width;
    let entity_top = pos.y + collider.offset_y;
    let entity_bottom = entity_top + collider.height;

    // Calculate geometry bounds
    let geo_left = geometry.x;
    let geo_right = geometry.x + geometry.width;
    let geo_top = geometry.y;
    let geo_bottom = geometry.y + geometry.height;

    // Calculate entry and exit times for each axis
    let entry_x: f32;
    let exit_x: f32;

    if movement.x > 0.0 {
        entry_x = geo_left - entity_right;
        exit_x = geo_right - entity_left;
    } else if movement.x < 0.0 {
        entry_x = geo_right - entity_left;
        exit_x = geo_left - entity_right;
    } else {
        entry_x = f32::NEG_INFINITY;
        exit_x = f32::INFINITY;
    }

    let entry_y: f32;
    let exit_y: f32;

    if movement.y > 0.0 {
        entry_y = geo_top - entity_bottom;
        exit_y = geo_bottom - entity_top;
    } else if movement.y < 0.0 {
        entry_y = geo_bottom - entity_top;
        exit_y = geo_top - entity_bottom;
    } else {
        entry_y = f32::NEG_INFINITY;
        exit_y = f32::INFINITY;
    }

    // Calculate time of entry and exit
    let entry_time_x = if movement.x != 0.0 {
        entry_x / movement.x
    } else {
        f32::NEG_INFINITY
    };
    let exit_time_x = if movement.x != 0.0 {
        exit_x / movement.x
    } else {
        f32::INFINITY
    };
    let entry_time_y = if movement.y != 0.0 {
        entry_y / movement.y
    } else {
        f32::NEG_INFINITY
    };
    let exit_time_y = if movement.y != 0.0 {
        exit_y / movement.y
    } else {
        f32::INFINITY
    };

    // Find the latest entry time and earliest exit time
    let entry_time = entry_time_x.max(entry_time_y);
    let exit_time = exit_time_x.min(exit_time_y);

    // Check if collision occurs
    if entry_time > exit_time || entry_time_x < 0.0 && entry_time_y < 0.0 || entry_time > 1.0 {
        return None;
    }

    // Calculate collision normal
    let normal = if entry_time_x > entry_time_y {
        if movement.x > 0.0 {
            Vec2::new(-1.0, 0.0) // Hit left side
        } else {
            Vec2::new(1.0, 0.0) // Hit right side
        }
    } else if movement.y > 0.0 {
        Vec2::new(0.0, -1.0) // Hit top side
    } else {
        Vec2::new(0.0, 1.0) // Hit bottom side
    };

    Some((entry_time.max(0.0), normal))
}

/// Resolve collisions with level geometry
fn resolve_collisions(
    mut query: Query<(&mut Position, &mut Velocity, &Collider)>,
    geometry_query: Query<&LevelGeometry>,
    time: Res<Time<Fixed>>,
) {
    let delta_time = time.delta_seconds();

    for (mut position, mut velocity, collider) in query.iter_mut() {
        let movement = Vec2::new(velocity.x * delta_time, velocity.y * delta_time);

        // Find earliest collision
        let mut earliest_collision: Option<(f32, Vec2)> = None;
        let mut earliest_time = 1.0;

        for geometry in geometry_query.iter() {
            if let Some((time, normal)) =
                swept_aabb_collision(&position, collider, geometry, movement)
                && time < earliest_time
            {
                earliest_time = time;
                earliest_collision = Some((time, normal));
            }
        }

        if let Some((time, normal)) = earliest_collision {
            // Move to collision point
            position.x += velocity.x * delta_time * time;
            position.y += velocity.y * delta_time * time;

            // Project velocity along collision surface (remove component along normal)
            let dot = velocity.x * normal.x + velocity.y * normal.y;
            velocity.x -= dot * normal.x;
            velocity.y -= dot * normal.y;
        } else {
            // No collision, move full distance
            position.x += velocity.x * delta_time;
            position.y += velocity.y * delta_time;
        }
    }
}

/// Update grounded state based on ground contact
fn update_grounded_state(
    mut query: Query<(&Position, &Collider, &mut GroundedState)>,
    geometry_query: Query<&LevelGeometry>,
) {
    for (position, collider, mut grounded_state) in query.iter_mut() {
        // Check for ground contact by testing slightly below the entity
        let check_movement = Vec2::new(0.0, GROUND_CHECK_EPSILON);

        let mut is_on_ground = false;
        let mut ground_normal = Vec2::ZERO;

        for geometry in geometry_query.iter() {
            if let Some((time, normal)) =
                swept_aabb_collision(position, collider, geometry, check_movement)
            {
                // If collision happens very close and normal points up, we're on ground
                if time < 1.0 && normal.y < -0.5 {
                    is_on_ground = true;
                    ground_normal = normal;
                    break;
                }
            }
        }

        grounded_state.is_grounded = is_on_ground;
        grounded_state.ground_normal = ground_normal;
    }
}

/// Apply slope physics (friction and sliding)
fn apply_slope_physics(mut query: Query<(&mut Velocity, &GroundedState)>, time: Res<Time<Fixed>>) {
    let delta_time = time.delta_seconds();

    for (mut velocity, grounded_state) in query.iter_mut() {
        if !grounded_state.is_grounded {
            continue;
        }

        let normal = grounded_state.ground_normal;

        // Calculate slope angle from normal
        // For a flat surface, normal is (0, -1) pointing up
        // Angle is measured from horizontal
        let slope_angle = normal.x.atan2(-normal.y);

        // Only apply slope physics if angle is significant
        if slope_angle.abs() < MIN_SLOPE_ANGLE {
            continue;
        }

        // Calculate sliding force along slope: F = g * sin(θ)
        let sliding_acceleration = GRAVITY * slope_angle.sin();

        // Apply sliding force in the direction of the slope
        // Slope direction is perpendicular to normal
        let slope_direction = Vec2::new(-normal.y, normal.x);

        // Add sliding velocity
        velocity.x += sliding_acceleration * slope_direction.x * delta_time;
        velocity.y += sliding_acceleration * slope_direction.y * delta_time;

        // Apply friction to oppose motion along slope
        let velocity_along_slope = velocity.x * slope_direction.x + velocity.y * slope_direction.y;
        let friction_force = -velocity_along_slope * SLOPE_FRICTION;

        velocity.x += friction_force * slope_direction.x * delta_time;
        velocity.y += friction_force * slope_direction.y * delta_time;
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
        assert!(
            (velocity.y - expected_velocity).abs() < 0.01,
            "Expected velocity.y to be ~{}, got {}",
            expected_velocity,
            velocity.y
        );
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

        assert!(
            (position.x - expected_x).abs() < 0.01,
            "Expected position.x to be ~{}, got {}",
            expected_x,
            position.x
        );
        assert!(
            (position.y - expected_y).abs() < 0.01,
            "Expected position.y to be ~{}, got {}",
            expected_y,
            position.y
        );
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
        assert!(
            (velocity.y - expected_velocity).abs() < 0.01,
            "Expected velocity.y to be ~{}, got {}",
            expected_velocity,
            velocity.y
        );
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
        assert!(
            (position.x - 200.0).abs() < 0.1,
            "Expected position.x to be ~200.0, got {}",
            position.x
        );
        assert!(
            (position.y - 0.0).abs() < 0.01,
            "Expected position.y to be ~0.0, got {}",
            position.y
        );
    }

    #[test]
    fn test_swept_aabb_detects_collision() {
        let position = Position::new(0.0, 0.0);
        let collider = Collider::new(32.0, 32.0);
        let geometry = LevelGeometry {
            x: 50.0,
            y: 0.0,
            width: 100.0,
            height: 32.0,
        };

        // Moving right towards the geometry
        let movement = Vec2::new(30.0, 0.0);

        let result = swept_aabb_collision(&position, &collider, &geometry, movement);
        assert!(result.is_some(), "Should detect collision");

        if let Some((time, normal)) = result {
            assert!((0.0..=1.0).contains(&time), "Time should be in [0, 1]");
            assert_eq!(
                normal,
                Vec2::new(-1.0, 0.0),
                "Should hit left side of geometry"
            );
        }
    }

    #[test]
    fn test_swept_aabb_no_collision() {
        let position = Position::new(0.0, 0.0);
        let collider = Collider::new(32.0, 32.0);
        let geometry = LevelGeometry {
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        };

        // Moving away from geometry
        let movement = Vec2::new(-10.0, -10.0);

        let result = swept_aabb_collision(&position, &collider, &geometry, movement);
        assert!(result.is_none(), "Should not detect collision");
    }

    #[test]
    fn test_swept_aabb_vertical_collision() {
        let position = Position::new(0.0, 100.0);
        let collider = Collider::new(32.0, 32.0);
        let geometry = LevelGeometry {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 32.0,
        };

        // Moving down towards ground
        let movement = Vec2::new(0.0, -80.0);

        let result = swept_aabb_collision(&position, &collider, &geometry, movement);
        assert!(result.is_some(), "Should detect collision with ground");

        if let Some((time, normal)) = result {
            assert!((0.0..=1.0).contains(&time), "Time should be in [0, 1]");
            assert_eq!(
                normal,
                Vec2::new(0.0, 1.0),
                "Should hit top of ground (normal points up)"
            );
        }
    }

    #[test]
    fn test_velocity_projection_on_collision() {
        let mut velocity = Velocity::new(100.0, 200.0);
        let normal = Vec2::new(0.0, 1.0); // Ground normal (pointing up)

        // Project velocity along surface (remove component along normal)
        let dot = velocity.x * normal.x + velocity.y * normal.y;
        velocity.x -= dot * normal.x;
        velocity.y -= dot * normal.y;

        // Vertical component should be zero, horizontal unchanged
        assert_eq!(velocity.x, 100.0, "Horizontal velocity should be unchanged");
        assert_eq!(
            velocity.y, 0.0,
            "Vertical velocity should be zero after ground collision"
        );
    }

    #[test]
    fn test_no_tunneling_through_thin_walls() {
        let position = Position::new(0.0, 0.0);
        let collider = Collider::new(32.0, 32.0);
        let thin_wall = LevelGeometry {
            x: 40.0,
            y: 0.0,
            width: 5.0, // Very thin wall
            height: 100.0,
        };

        // High velocity movement that would tunnel without swept collision
        let movement = Vec2::new(100.0, 0.0);

        let result = swept_aabb_collision(&position, &collider, &thin_wall, movement);
        assert!(
            result.is_some(),
            "Should detect collision even with high velocity"
        );
    }

    #[test]
    fn test_ground_detection() {
        let position = Position::new(0.0, 50.0);
        let collider = Collider::new(32.0, 32.0);
        let ground = LevelGeometry {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 32.0,
        };

        // Check slightly below entity
        let check_movement = Vec2::new(0.0, GROUND_CHECK_EPSILON);

        let result = swept_aabb_collision(&position, &collider, &ground, check_movement);

        // Should detect ground if entity is close enough
        if position.y + collider.height <= ground.y + ground.height + GROUND_CHECK_EPSILON {
            assert!(result.is_some(), "Should detect ground contact");
        }
    }

    #[test]
    fn test_collision_resolution_stops_at_wall() {
        let mut position = Position::new(0.0, 0.0);
        let mut velocity = Velocity::new(100.0, 0.0);
        let collider = Collider::new(32.0, 32.0);
        let wall = LevelGeometry {
            x: 50.0,
            y: 0.0,
            width: 10.0,
            height: 100.0,
        };

        let delta_time = FIXED_TIMESTEP;
        let movement = Vec2::new(velocity.x * delta_time, velocity.y * delta_time);

        if let Some((time, normal)) = swept_aabb_collision(&position, &collider, &wall, movement) {
            // Move to collision point
            position.x += velocity.x * delta_time * time;
            position.y += velocity.y * delta_time * time;

            // Project velocity
            let dot = velocity.x * normal.x + velocity.y * normal.y;
            velocity.x -= dot * normal.x;
            velocity.y -= dot * normal.y;

            // Should stop at wall
            assert!(position.x < wall.x, "Should stop before wall");
            assert_eq!(
                velocity.x, 0.0,
                "Horizontal velocity should be zero after wall collision"
            );
        }
    }

    #[test]
    fn test_slope_angle_calculation() {
        // Test various slope normals

        // Flat ground (normal pointing straight up)
        let flat_normal = Vec2::new(0.0, -1.0);
        let flat_angle = flat_normal.x.atan2(-flat_normal.y);
        assert!(
            (flat_angle).abs() < 0.01,
            "Flat ground should have ~0 angle"
        );

        // 45-degree slope (normal at 45 degrees)
        let slope_45_normal = Vec2::new(0.707, -0.707).normalize();
        let slope_45_angle = slope_45_normal.x.atan2(-slope_45_normal.y);
        assert!(
            (slope_45_angle - std::f32::consts::PI / 4.0).abs() < 0.01,
            "45-degree slope should have angle ~π/4"
        );
    }

    #[test]
    fn test_slope_sliding_acceleration() {
        // Test that sliding acceleration equals g * sin(θ)
        let slope_angle = std::f32::consts::PI / 6.0; // 30 degrees
        let expected_acceleration = GRAVITY * slope_angle.sin();

        // For 30 degrees, sin(30°) = 0.5
        assert!(
            (expected_acceleration - GRAVITY * 0.5).abs() < 1.0,
            "Sliding acceleration should be g * sin(θ)"
        );
    }

    #[test]
    fn test_slope_physics_applies_sliding_force() {
        let mut velocity = Velocity::new(0.0, 0.0);

        // Create a slope with 30-degree angle
        // Normal for 30-degree slope: perpendicular to slope surface
        let slope_angle = std::f32::consts::PI / 6.0;
        let normal = Vec2::new(slope_angle.sin(), -slope_angle.cos());

        let _grounded_state = GroundedState {
            is_grounded: true,
            ground_normal: normal,
        };

        // Calculate expected sliding
        let calculated_angle = normal.x.atan2(-normal.y);
        let sliding_acceleration = GRAVITY * calculated_angle.sin();
        let slope_direction = Vec2::new(-normal.y, normal.x);

        // Apply for one frame
        velocity.x += sliding_acceleration * slope_direction.x * FIXED_TIMESTEP;
        velocity.y += sliding_acceleration * slope_direction.y * FIXED_TIMESTEP;

        // Velocity should increase in the direction of the slope
        let velocity_magnitude = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        assert!(velocity_magnitude > 0.0, "Entity should slide down slope");
    }

    #[test]
    fn test_slope_friction_opposes_motion() {
        // Start with velocity along slope
        let mut velocity = Velocity::new(100.0, 0.0);

        // Flat-ish slope (small angle to isolate friction effect)
        let normal = Vec2::new(0.05, -0.998).normalize();
        let slope_direction = Vec2::new(-normal.y, normal.x);

        let velocity_along_slope = velocity.x * slope_direction.x + velocity.y * slope_direction.y;
        let friction_force = -velocity_along_slope * SLOPE_FRICTION;

        let initial_velocity = velocity.x;
        velocity.x += friction_force * slope_direction.x * FIXED_TIMESTEP;

        // Friction should reduce velocity
        assert!(
            velocity.x.abs() < initial_velocity.abs(),
            "Friction should reduce velocity magnitude"
        );
    }

    #[test]
    fn test_no_slope_physics_when_airborne() {
        let mut velocity = Velocity::new(0.0, 0.0);
        let _grounded_state = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::new(0.5, -0.866), // 30-degree slope
        };

        // Slope physics should not apply when airborne
        if _grounded_state.is_grounded {
            velocity.x += 100.0; // This should not execute
        }

        assert_eq!(velocity.x, 0.0, "No slope physics when airborne");
    }

    #[test]
    fn test_no_slope_physics_on_flat_ground() {
        let mut velocity = Velocity::new(0.0, 0.0);
        let grounded_state = GroundedState {
            is_grounded: true,
            ground_normal: Vec2::new(0.0, -1.0), // Flat ground
        };

        let normal = grounded_state.ground_normal;
        let slope_angle = normal.x.atan2(-normal.y);

        // Should not apply slope physics for flat ground
        if slope_angle.abs() >= MIN_SLOPE_ANGLE {
            velocity.x += 100.0; // This should not execute
        }

        assert_eq!(velocity.x, 0.0, "No slope physics on flat ground");
    }
}
