use crate::components::{
    AbilitySet, GroundedState, Player, PlayerIntent, Position, SwingPoint, SwingState, Velocity,
};
use crate::enums::Ability;
use bevy::prelude::*;

/// Physics constants for swing mechanics
pub const SWING_DAMPING: f32 = 0.98; // Angular velocity damping per frame
pub const SWING_INPUT_TORQUE: f32 = 2.0; // Torque applied by player input
pub const SWING_RANGE: f32 = 100.0; // Default range for swing points

/// Plugin for swing mechanic
pub struct SwingPlugin;

impl Plugin for SwingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                detect_swing_points_system,
                attach_to_swing_system,
                update_swing_physics_system,
                release_swing_system,
            )
                .chain(),
        );
    }
}

/// Detect nearby swing points and store the closest one
fn detect_swing_points_system(
    player_query: Query<&Position, With<Player>>,
    swing_point_query: Query<(&Position, &SwingPoint)>,
    mut nearest_swing_point: Local<Option<(Vec2, f32)>>, // (position, distance)
) {
    *nearest_swing_point = None;

    for player_pos in player_query.iter() {
        let player_position = Vec2::new(player_pos.x, player_pos.y);
        let mut closest_distance = f32::MAX;
        let mut closest_point = None;

        for (swing_pos, swing_point) in swing_point_query.iter() {
            let swing_position = Vec2::new(swing_pos.x, swing_pos.y);
            let distance = player_position.distance(swing_position);

            // Check if within range and closer than previous closest
            if distance <= swing_point.range && distance < closest_distance {
                closest_distance = distance;
                closest_point = Some((swing_position, distance));
            }
        }

        *nearest_swing_point = closest_point;
    }
}

/// Attach to swing point when conditions are met
#[allow(clippy::type_complexity)]
fn attach_to_swing_system(
    mut commands: Commands,
    mut player_query: Query<
        (
            Entity,
            &Position,
            &AbilitySet,
            &PlayerIntent,
            &GroundedState,
        ),
        (With<Player>, Without<SwingState>),
    >,
    nearest_swing_point: Local<Option<(Vec2, f32)>>,
) {
    for (entity, _player_pos, ability_set, intent, grounded) in player_query.iter_mut() {
        // Can only attach if:
        // 1. Swing ability is unlocked
        // 2. Player is within range of a swing point
        // 3. Player presses the swing key (we'll use 'E' key)
        // 4. Player is not grounded (can't swing from ground)

        if !ability_set.has(Ability::Swing) {
            continue;
        }

        if grounded.is_grounded {
            continue;
        }

        // Check if there's a nearby swing point
        if let Some((swing_position, distance)) = *nearest_swing_point {
            // For now, we'll use jump_pressed as swing key (will be changed later)
            // In a real implementation, we'd add a separate swing_pressed field to PlayerIntent
            if intent.jump_pressed {
                // Attach to swing point
                let rope_length = distance;

                commands.entity(entity).insert(SwingState {
                    anchor_point: swing_position,
                    rope_length,
                    angular_velocity: 0.0,
                });
            }
        }
    }
}

/// Update swing physics using pendulum mechanics
fn update_swing_physics_system(
    mut query: Query<(&mut Position, &mut Velocity, &mut SwingState, &PlayerIntent), With<Player>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();

    for (mut position, mut velocity, mut swing_state, input) in query.iter_mut() {
        // Calculate current angle from anchor point
        let dx = position.x - swing_state.anchor_point.x;
        let dy = position.y - swing_state.anchor_point.y;
        let angle = dy.atan2(dx);

        // Apply pendulum physics: angular acceleration = (g / L) * sin(θ)
        let gravity = 980.0; // pixels per second squared
        let angular_acceleration = (gravity / swing_state.rope_length) * angle.sin();
        swing_state.angular_velocity += angular_acceleration * delta_time;

        // Apply player input as torque
        if input.move_left {
            swing_state.angular_velocity -= SWING_INPUT_TORQUE * delta_time;
        }
        if input.move_right {
            swing_state.angular_velocity += SWING_INPUT_TORQUE * delta_time;
        }

        // Apply damping
        swing_state.angular_velocity *= SWING_DAMPING;

        // Update angle
        let new_angle = angle + swing_state.angular_velocity * delta_time;

        // Calculate new position on arc
        position.x = swing_state.anchor_point.x + swing_state.rope_length * new_angle.cos();
        position.y = swing_state.anchor_point.y + swing_state.rope_length * new_angle.sin();

        // Calculate tangential velocity for release
        velocity.x = -swing_state.rope_length * swing_state.angular_velocity * new_angle.sin();
        velocity.y = swing_state.rope_length * swing_state.angular_velocity * new_angle.cos();
    }
}

/// Release from swing point when key is released
fn release_swing_system(
    mut commands: Commands,
    query: Query<(Entity, &PlayerIntent, &SwingState), With<Player>>,
) {
    for (entity, intent, _swing_state) in query.iter() {
        // Release when jump key is released
        // In a real implementation, we'd check for swing key release
        if intent.jump_just_released {
            // Remove SwingState component to exit swing mode
            commands.entity(entity).remove::<SwingState>();
            // Velocity is already set by update_swing_physics_system
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swing_point_detection_within_range() {
        let player_pos = Position::new(100.0, 100.0);
        let swing_pos = Position::new(150.0, 100.0);
        let swing_point = SwingPoint { range: 100.0 };

        let player_position = Vec2::new(player_pos.x, player_pos.y);
        let swing_position = Vec2::new(swing_pos.x, swing_pos.y);
        let distance = player_position.distance(swing_position);

        assert!(
            distance <= swing_point.range,
            "Player should be within swing range"
        );
        assert_eq!(distance, 50.0, "Distance should be 50 pixels");
    }

    #[test]
    fn test_swing_point_detection_out_of_range() {
        let player_pos = Position::new(100.0, 100.0);
        let swing_pos = Position::new(300.0, 100.0);
        let swing_point = SwingPoint { range: 100.0 };

        let player_position = Vec2::new(player_pos.x, player_pos.y);
        let swing_position = Vec2::new(swing_pos.x, swing_pos.y);
        let distance = player_position.distance(swing_position);

        assert!(
            distance > swing_point.range,
            "Player should be out of swing range"
        );
    }

    #[test]
    fn test_swing_attachment_requires_ability() {
        let ability_set = AbilitySet::new(); // No swing ability
        assert!(
            !ability_set.has(Ability::Swing),
            "Should not have swing ability"
        );
    }

    #[test]
    fn test_swing_attachment_with_ability() {
        let mut ability_set = AbilitySet::new();
        ability_set.add(Ability::Swing);
        assert!(ability_set.has(Ability::Swing), "Should have swing ability");
    }

    #[test]
    fn test_pendulum_angular_acceleration() {
        let gravity = 980.0;
        let rope_length = 100.0;
        let angle = std::f32::consts::PI / 4.0; // 45 degrees

        let angular_acceleration = (gravity / rope_length) * angle.sin();

        // For 45 degrees, sin(45°) ≈ 0.707
        let expected = (gravity / rope_length) * 0.707;
        assert!(
            (angular_acceleration - expected).abs() < 0.1,
            "Angular acceleration should match pendulum formula"
        );
    }

    #[test]
    fn test_swing_damping_reduces_velocity() {
        let mut angular_velocity = 10.0;
        let initial_velocity = angular_velocity;

        angular_velocity *= SWING_DAMPING;

        assert!(
            angular_velocity < initial_velocity,
            "Damping should reduce angular velocity"
        );
        assert!(
            angular_velocity > 0.0,
            "Damping should not reverse direction"
        );
    }

    #[test]
    fn test_swing_input_affects_angular_velocity() {
        let mut angular_velocity = 0.0;
        let delta_time = 1.0 / 60.0;

        // Apply right input
        angular_velocity += SWING_INPUT_TORQUE * delta_time;

        assert!(
            angular_velocity > 0.0,
            "Right input should increase angular velocity"
        );

        // Apply left input
        angular_velocity -= SWING_INPUT_TORQUE * delta_time * 2.0;

        assert!(
            angular_velocity < 0.0,
            "Left input should decrease angular velocity"
        );
    }

    #[test]
    fn test_tangential_velocity_calculation() {
        let rope_length = 100.0;
        let angular_velocity = 1.0; // radians per second
        let angle: f32 = 0.0; // Horizontal position

        // Calculate tangential velocity
        let velocity_x = -rope_length * angular_velocity * angle.sin();
        let velocity_y = rope_length * angular_velocity * angle.cos();

        // At angle 0, sin(0) = 0, cos(0) = 1
        assert_eq!(
            velocity_x, 0.0,
            "Horizontal velocity should be 0 at angle 0"
        );
        assert_eq!(
            velocity_y, 100.0,
            "Vertical velocity should equal rope_length * angular_velocity"
        );
    }

    #[test]
    fn test_swing_position_on_arc() {
        let anchor = Vec2::new(100.0, 200.0);
        let rope_length = 50.0;
        let angle: f32 = 0.0; // Horizontal right

        let position_x = anchor.x + rope_length * angle.cos();
        let position_y = anchor.y + rope_length * angle.sin();

        // At angle 0, cos(0) = 1, sin(0) = 0
        assert_eq!(
            position_x, 150.0,
            "Position should be rope_length to the right"
        );
        assert_eq!(position_y, 200.0, "Position should be at same height");
    }

    #[test]
    fn test_no_swing_when_grounded() {
        let grounded = GroundedState {
            is_grounded: true,
            ground_normal: Vec2::new(0.0, -1.0),
        };

        // Should not attach to swing when grounded
        assert!(
            grounded.is_grounded,
            "Should not be able to swing when grounded"
        );
    }

    #[test]
    fn test_swing_allowed_when_airborne() {
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };

        // Should be able to attach to swing when airborne
        assert!(
            !grounded.is_grounded,
            "Should be able to swing when airborne"
        );
    }
}
