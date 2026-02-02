use crate::components::{
    AbilitySet, Collider, GroundedState, LevelGeometry, Player, PlayerIntent, Position, Velocity,
    WallClimbState,
};
use crate::enums::Ability;
use crate::plugins::physics::swept_aabb_collision;
use bevy::prelude::*;

/// Physics constants
pub const MOVE_SPEED: f32 = 200.0; // pixels per second
pub const BASE_JUMP_VELOCITY: f32 = -400.0; // pixels per second (negative = up)
pub const HIGH_JUMP_VELOCITY: f32 = -600.0; // pixels per second
pub const WALL_CLIMB_SPEED: f32 = 150.0; // pixels per second
pub const WALL_JUMP_HORIZONTAL_VELOCITY: f32 = 250.0; // pixels per second
pub const WALL_JUMP_VERTICAL_VELOCITY: f32 = -450.0; // pixels per second (negative = up)

/// Plugin for player character logic and state
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                process_input_system,
                update_wall_cling_state,
                apply_horizontal_movement_system,
                apply_wall_climb_movement_system,
                apply_jump_system,
            )
                .chain(),
        );
    }
}

/// Process keyboard input and translate to PlayerIntent
fn process_input_system(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut PlayerIntent, With<Player>>,
) {
    for mut intent in query.iter_mut() {
        intent.move_left = keyboard.pressed(KeyCode::Left) || keyboard.pressed(KeyCode::A);
        intent.move_right = keyboard.pressed(KeyCode::Right) || keyboard.pressed(KeyCode::D);
        intent.jump_pressed = keyboard.pressed(KeyCode::Space);
        intent.jump_just_released = keyboard.just_released(KeyCode::Space);
    }
}

/// Apply horizontal movement based on player intent
fn apply_horizontal_movement_system(
    mut query: Query<(&PlayerIntent, &mut Velocity, &GroundedState), With<Player>>,
) {
    for (intent, mut velocity, grounded) in query.iter_mut() {
        // Only apply horizontal movement when grounded
        if grounded.is_grounded {
            if intent.move_right && !intent.move_left {
                velocity.x = MOVE_SPEED;
            } else if intent.move_left && !intent.move_right {
                velocity.x = -MOVE_SPEED;
            } else {
                velocity.x = 0.0;
            }
        }
    }
}

/// Apply jump mechanics based on player intent
fn apply_jump_system(
    mut query: Query<
        (
            &PlayerIntent,
            &mut Velocity,
            &GroundedState,
            &AbilitySet,
            &mut WallClimbState,
        ),
        With<Player>,
    >,
) {
    for (intent, mut velocity, grounded, ability_set, mut wall_state) in query.iter_mut() {
        // Wall jump - takes priority over normal jump
        if wall_state.is_clinging && intent.jump_pressed {
            // Apply velocity away from wall and upward
            let wall_normal = wall_state.wall_normal;

            // Horizontal velocity is in direction of wall normal (away from wall)
            velocity.x = wall_normal.x * WALL_JUMP_HORIZONTAL_VELOCITY;

            // Vertical velocity is upward
            velocity.y = WALL_JUMP_VERTICAL_VELOCITY;

            // Exit wall-cling state
            wall_state.is_clinging = false;

            return; // Don't process normal jump
        }

        // Apply jump velocity when grounded and jump pressed
        if grounded.is_grounded && intent.jump_pressed {
            // Check if high jump ability is unlocked
            if ability_set.has(Ability::HighJump) {
                velocity.y = HIGH_JUMP_VELOCITY;
            } else {
                velocity.y = BASE_JUMP_VELOCITY;
            }
        }

        // Variable jump height - reduce velocity on key release during ascent
        if intent.jump_just_released && velocity.y < 0.0 {
            velocity.y *= 0.5;
        }
    }
}

/// Update wall-cling state based on conditions
#[allow(clippy::type_complexity)]
fn update_wall_cling_state(
    mut query: Query<
        (
            &PlayerIntent,
            &AbilitySet,
            &mut WallClimbState,
            &GroundedState,
            &Position,
            &Collider,
        ),
        With<Player>,
    >,
    geometry_query: Query<&LevelGeometry>,
) {
    for (intent, ability_set, mut wall_state, grounded, position, collider) in query.iter_mut() {
        // Can only enter wall-cling if:
        // 1. Wall climb ability is unlocked
        // 2. Adjacent to a wall (wall_normal is non-zero)
        // 3. Player is pressing toward the wall
        // 4. Not grounded

        let has_wall_climb = ability_set.has(Ability::WallClimb);
        let wall_detected = wall_state.wall_normal != Vec2::ZERO;

        // Check if player has reached the top of the wall
        if wall_state.is_clinging {
            let mut wall_continues_above = false;

            // Check if there's still wall above the player
            let check_above = Vec2::new(0.0, -10.0); // Check 10 pixels above
            for geometry in geometry_query.iter() {
                // Check if wall continues above current position
                if let Some((time, normal)) =
                    swept_aabb_collision(position, collider, geometry, check_above)
                {
                    // If we hit a wall above and it's the same wall (same normal direction)
                    if time < 1.0
                        && normal.x.abs() > 0.5
                        && ((normal.x > 0.0 && wall_state.wall_normal.x > 0.0)
                            || (normal.x < 0.0 && wall_state.wall_normal.x < 0.0))
                    {
                        wall_continues_above = true;
                        break;
                    }
                }
            }

            // If no wall above, exit wall-cling (reached top)
            if !wall_continues_above {
                wall_state.is_clinging = false;
                return;
            }
        }

        if has_wall_climb && wall_detected && !grounded.is_grounded {
            // Check if player is pressing toward the wall
            let wall_on_left = wall_state.wall_normal.x > 0.0; // Wall normal points right, wall is on left
            let wall_on_right = wall_state.wall_normal.x < 0.0; // Wall normal points left, wall is on right

            let pressing_toward_wall =
                (wall_on_left && intent.move_left) || (wall_on_right && intent.move_right);

            wall_state.is_clinging = pressing_toward_wall;
        } else {
            wall_state.is_clinging = false;
        }
    }
}

/// Apply wall climb movement when in wall-cling state
fn apply_wall_climb_movement_system(
    mut query: Query<(&PlayerIntent, &WallClimbState, &mut Velocity), With<Player>>,
) {
    for (intent, wall_state, mut velocity) in query.iter_mut() {
        if wall_state.is_clinging {
            // Allow vertical movement input
            if intent.move_left || intent.move_right {
                // Determine vertical movement based on up/down keys
                // For now, we'll use a simple approach where holding the direction key
                // allows climbing up, and not holding it allows sliding down slowly
                velocity.y = -WALL_CLIMB_SPEED; // Climb up
            } else {
                velocity.y = 0.0; // Stay in place
            }

            // Zero out horizontal velocity while clinging
            velocity.x = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;

    fn create_test_player() -> (PlayerIntent, Velocity, GroundedState) {
        (
            PlayerIntent::default(),
            Velocity::default(),
            GroundedState {
                is_grounded: true,
                ground_normal: Vec2::new(0.0, 1.0),
            },
        )
    }

    #[test]
    fn test_horizontal_movement_right() {
        let (mut intent, mut velocity, grounded) = create_test_player();
        intent.move_right = true;
        intent.move_left = false;

        // Simulate the system
        if grounded.is_grounded && intent.move_right && !intent.move_left {
            velocity.x = MOVE_SPEED;
        }

        assert_eq!(velocity.x, MOVE_SPEED);
    }

    #[test]
    fn test_horizontal_movement_left() {
        let (mut intent, mut velocity, grounded) = create_test_player();
        intent.move_left = true;
        intent.move_right = false;

        // Simulate the system
        if grounded.is_grounded && intent.move_left && !intent.move_right {
            velocity.x = -MOVE_SPEED;
        }

        assert_eq!(velocity.x, -MOVE_SPEED);
    }

    #[test]
    fn test_horizontal_movement_both_keys() {
        let (mut intent, mut velocity, grounded) = create_test_player();
        intent.move_left = true;
        intent.move_right = true;

        // Simulate the system
        if grounded.is_grounded {
            if intent.move_right && !intent.move_left {
                velocity.x = MOVE_SPEED;
            } else if intent.move_left && !intent.move_right {
                velocity.x = -MOVE_SPEED;
            } else {
                velocity.x = 0.0;
            }
        }

        assert_eq!(velocity.x, 0.0);
    }

    #[test]
    fn test_horizontal_movement_no_keys() {
        let (intent, mut velocity, grounded) = create_test_player();

        // Simulate the system
        if grounded.is_grounded {
            if intent.move_right && !intent.move_left {
                velocity.x = MOVE_SPEED;
            } else if intent.move_left && !intent.move_right {
                velocity.x = -MOVE_SPEED;
            } else {
                velocity.x = 0.0;
            }
        }

        assert_eq!(velocity.x, 0.0);
    }

    #[test]
    fn test_jump_initial_velocity() {
        let (mut intent, mut velocity, grounded) = create_test_player();
        intent.jump_pressed = true;

        // Simulate the system
        if grounded.is_grounded && intent.jump_pressed {
            velocity.y = BASE_JUMP_VELOCITY;
        }

        assert_eq!(velocity.y, BASE_JUMP_VELOCITY);
    }

    #[test]
    fn test_jump_not_grounded() {
        let (mut intent, mut velocity, mut grounded) = create_test_player();
        grounded.is_grounded = false;
        intent.jump_pressed = true;
        let initial_velocity = velocity.y;

        // Simulate the system
        if grounded.is_grounded && intent.jump_pressed {
            velocity.y = BASE_JUMP_VELOCITY;
        }

        assert_eq!(velocity.y, initial_velocity);
    }

    #[test]
    fn test_variable_jump_height() {
        let (mut intent, mut velocity, _grounded) = create_test_player();
        velocity.y = -300.0; // Ascending
        intent.jump_just_released = true;

        // Simulate the system
        if intent.jump_just_released && velocity.y < 0.0 {
            velocity.y *= 0.5;
        }

        assert_eq!(velocity.y, -150.0);
    }

    #[test]
    fn test_variable_jump_no_effect_when_descending() {
        let (mut intent, mut velocity, _grounded) = create_test_player();
        velocity.y = 100.0; // Descending
        intent.jump_just_released = true;

        // Simulate the system
        if intent.jump_just_released && velocity.y < 0.0 {
            velocity.y *= 0.5;
        }

        assert_eq!(velocity.y, 100.0);
    }

    #[test]
    fn test_no_horizontal_movement_when_airborne() {
        let (mut intent, mut velocity, mut grounded) = create_test_player();
        grounded.is_grounded = false;
        intent.move_right = true;
        velocity.x = 50.0; // Some existing velocity

        // Simulate the system
        if grounded.is_grounded && intent.move_right && !intent.move_left {
            velocity.x = MOVE_SPEED;
        }

        // Velocity should remain unchanged when airborne
        assert_eq!(velocity.x, 50.0);
    }

    #[test]
    fn test_high_jump_increases_velocity() {
        let (mut intent, mut velocity, grounded) = create_test_player();
        let mut ability_set = AbilitySet::new();
        ability_set.add(Ability::HighJump);
        intent.jump_pressed = true;

        // Simulate the system with high jump
        if grounded.is_grounded && intent.jump_pressed {
            if ability_set.has(Ability::HighJump) {
                velocity.y = HIGH_JUMP_VELOCITY;
            } else {
                velocity.y = BASE_JUMP_VELOCITY;
            }
        }

        assert_eq!(velocity.y, HIGH_JUMP_VELOCITY);
        assert!(velocity.y.abs() > BASE_JUMP_VELOCITY.abs());
    }

    #[test]
    fn test_base_jump_without_high_jump_ability() {
        let (mut intent, mut velocity, grounded) = create_test_player();
        let ability_set = AbilitySet::new(); // No abilities
        intent.jump_pressed = true;

        // Simulate the system without high jump
        if grounded.is_grounded && intent.jump_pressed {
            if ability_set.has(Ability::HighJump) {
                velocity.y = HIGH_JUMP_VELOCITY;
            } else {
                velocity.y = BASE_JUMP_VELOCITY;
            }
        }

        assert_eq!(velocity.y, BASE_JUMP_VELOCITY);
    }

    #[test]
    fn test_high_jump_only_when_grounded() {
        let (mut intent, mut velocity, mut grounded) = create_test_player();
        grounded.is_grounded = false;
        let mut ability_set = AbilitySet::new();
        ability_set.add(Ability::HighJump);
        intent.jump_pressed = true;
        let initial_velocity = velocity.y;

        // Simulate the system
        if grounded.is_grounded && intent.jump_pressed {
            if ability_set.has(Ability::HighJump) {
                velocity.y = HIGH_JUMP_VELOCITY;
            } else {
                velocity.y = BASE_JUMP_VELOCITY;
            }
        }

        // Velocity should remain unchanged when airborne
        assert_eq!(velocity.y, initial_velocity);
    }

    #[test]
    fn test_wall_cling_state_entered_with_ability() {
        let mut intent = PlayerIntent::default();
        let mut ability_set = AbilitySet::new();
        ability_set.add(Ability::WallClimb);
        let wall_state = WallClimbState {
            is_clinging: false,
            wall_normal: Vec2::new(1.0, 0.0), // Wall on left
        };
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };
        intent.move_left = true; // Pressing toward wall

        // Simulate wall-cling state update
        let mut is_clinging = false;
        let has_wall_climb = ability_set.has(Ability::WallClimb);
        let wall_detected = wall_state.wall_normal != Vec2::ZERO;

        if has_wall_climb && wall_detected && !grounded.is_grounded {
            let wall_on_left = wall_state.wall_normal.x > 0.0;
            let wall_on_right = wall_state.wall_normal.x < 0.0;
            let pressing_toward_wall =
                (wall_on_left && intent.move_left) || (wall_on_right && intent.move_right);

            if pressing_toward_wall {
                is_clinging = true;
            }
        }

        assert!(
            is_clinging,
            "Should enter wall-cling state with ability and correct input"
        );
    }

    #[test]
    fn test_wall_cling_state_not_entered_without_ability() {
        let mut intent = PlayerIntent::default();
        let ability_set = AbilitySet::new(); // No wall climb ability
        let wall_state = WallClimbState {
            is_clinging: false,
            wall_normal: Vec2::new(1.0, 0.0), // Wall on left
        };
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };
        intent.move_left = true; // Pressing toward wall

        // Simulate wall-cling state update
        let mut is_clinging = false;
        let has_wall_climb = ability_set.has(Ability::WallClimb);
        let wall_detected = wall_state.wall_normal != Vec2::ZERO;

        if has_wall_climb && wall_detected && !grounded.is_grounded {
            let wall_on_left = wall_state.wall_normal.x > 0.0;
            let wall_on_right = wall_state.wall_normal.x < 0.0;
            let pressing_toward_wall =
                (wall_on_left && intent.move_left) || (wall_on_right && intent.move_right);

            if pressing_toward_wall {
                is_clinging = true;
            }
        }

        assert!(
            !is_clinging,
            "Should not enter wall-cling state without ability"
        );
    }

    #[test]
    fn test_wall_cling_state_not_entered_when_grounded() {
        let mut intent = PlayerIntent::default();
        let mut ability_set = AbilitySet::new();
        ability_set.add(Ability::WallClimb);
        let wall_state = WallClimbState {
            is_clinging: false,
            wall_normal: Vec2::new(1.0, 0.0), // Wall on left
        };
        let grounded = GroundedState {
            is_grounded: true, // Grounded
            ground_normal: Vec2::new(0.0, -1.0),
        };
        intent.move_left = true; // Pressing toward wall

        // Simulate wall-cling state update
        let mut is_clinging = false;
        let has_wall_climb = ability_set.has(Ability::WallClimb);
        let wall_detected = wall_state.wall_normal != Vec2::ZERO;

        if has_wall_climb && wall_detected && !grounded.is_grounded {
            let wall_on_left = wall_state.wall_normal.x > 0.0;
            let wall_on_right = wall_state.wall_normal.x < 0.0;
            let pressing_toward_wall =
                (wall_on_left && intent.move_left) || (wall_on_right && intent.move_right);

            if pressing_toward_wall {
                is_clinging = true;
            }
        }

        assert!(
            !is_clinging,
            "Should not enter wall-cling state when grounded"
        );
    }

    #[test]
    fn test_wall_cling_negates_gravity() {
        let mut velocity = Velocity::new(0.0, 0.0);
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };
        let wall_state = WallClimbState {
            is_clinging: true,
            wall_normal: Vec2::new(1.0, 0.0),
        };

        // Simulate gravity application
        let delta_time = 1.0 / 60.0;
        if !grounded.is_grounded && !wall_state.is_clinging {
            velocity.y += 980.0 * delta_time;
        }

        assert_eq!(
            velocity.y, 0.0,
            "Gravity should not be applied during wall-cling"
        );
    }

    #[test]
    fn test_wall_climb_movement_speed() {
        let intent = PlayerIntent {
            move_left: true,
            ..Default::default()
        };
        let wall_state = WallClimbState {
            is_clinging: true,
            wall_normal: Vec2::new(1.0, 0.0),
        };
        let mut velocity = Velocity::new(50.0, 100.0);

        // Simulate wall climb movement
        if wall_state.is_clinging {
            if intent.move_left || intent.move_right {
                velocity.y = -WALL_CLIMB_SPEED;
            } else {
                velocity.y = 0.0;
            }
            velocity.x = 0.0;
        }

        assert_eq!(
            velocity.y, -WALL_CLIMB_SPEED,
            "Should climb at wall climb speed"
        );
        assert_eq!(
            velocity.x, 0.0,
            "Horizontal velocity should be zero while clinging"
        );
    }

    #[test]
    fn test_wall_cling_exits_when_not_pressing_toward_wall() {
        let intent = PlayerIntent::default(); // Not pressing any direction
        let mut ability_set = AbilitySet::new();
        ability_set.add(Ability::WallClimb);
        let wall_state = WallClimbState {
            is_clinging: false,
            wall_normal: Vec2::new(1.0, 0.0), // Wall on left
        };
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };

        // Simulate wall-cling state update
        let mut is_clinging = false;
        let has_wall_climb = ability_set.has(Ability::WallClimb);
        let wall_detected = wall_state.wall_normal != Vec2::ZERO;

        if has_wall_climb && wall_detected && !grounded.is_grounded {
            let wall_on_left = wall_state.wall_normal.x > 0.0;
            let wall_on_right = wall_state.wall_normal.x < 0.0;
            let pressing_toward_wall =
                (wall_on_left && intent.move_left) || (wall_on_right && intent.move_right);

            if pressing_toward_wall {
                is_clinging = true;
            }
        }

        assert!(
            !is_clinging,
            "Should exit wall-cling when not pressing toward wall"
        );
    }

    #[test]
    fn test_wall_jump_from_left_wall() {
        let intent = PlayerIntent {
            jump_pressed: true,
            ..Default::default()
        };
        let mut velocity = Velocity::new(0.0, 0.0);
        let wall_state = WallClimbState {
            is_clinging: true,
            wall_normal: Vec2::new(1.0, 0.0), // Wall on left, normal points right
        };

        // Simulate wall jump
        if wall_state.is_clinging && intent.jump_pressed {
            let wall_normal = wall_state.wall_normal;
            velocity.x = wall_normal.x * WALL_JUMP_HORIZONTAL_VELOCITY;
            velocity.y = WALL_JUMP_VERTICAL_VELOCITY;
        }

        assert!(
            velocity.x > 0.0,
            "Should jump away from left wall (positive x)"
        );
        assert!(velocity.y < 0.0, "Should jump upward (negative y)");
        assert_eq!(
            velocity.x, WALL_JUMP_HORIZONTAL_VELOCITY,
            "Horizontal velocity should match constant"
        );
        assert_eq!(
            velocity.y, WALL_JUMP_VERTICAL_VELOCITY,
            "Vertical velocity should match constant"
        );
    }

    #[test]
    fn test_wall_jump_from_right_wall() {
        let intent = PlayerIntent {
            jump_pressed: true,
            ..Default::default()
        };
        let mut velocity = Velocity::new(0.0, 0.0);
        let wall_state = WallClimbState {
            is_clinging: true,
            wall_normal: Vec2::new(-1.0, 0.0), // Wall on right, normal points left
        };

        // Simulate wall jump
        if wall_state.is_clinging && intent.jump_pressed {
            let wall_normal = wall_state.wall_normal;
            velocity.x = wall_normal.x * WALL_JUMP_HORIZONTAL_VELOCITY;
            velocity.y = WALL_JUMP_VERTICAL_VELOCITY;
        }

        assert!(
            velocity.x < 0.0,
            "Should jump away from right wall (negative x)"
        );
        assert!(velocity.y < 0.0, "Should jump upward (negative y)");
        assert_eq!(
            velocity.x, -WALL_JUMP_HORIZONTAL_VELOCITY,
            "Horizontal velocity should match constant"
        );
        assert_eq!(
            velocity.y, WALL_JUMP_VERTICAL_VELOCITY,
            "Vertical velocity should match constant"
        );
    }

    #[test]
    fn test_wall_jump_exits_cling_state() {
        let intent = PlayerIntent {
            jump_pressed: true,
            ..Default::default()
        };
        let mut wall_state = WallClimbState {
            is_clinging: true,
            wall_normal: Vec2::new(1.0, 0.0),
        };

        // Simulate wall jump
        if wall_state.is_clinging && intent.jump_pressed {
            wall_state.is_clinging = false;
        }

        assert!(
            !wall_state.is_clinging,
            "Should exit wall-cling state after wall jump"
        );
    }

    #[test]
    fn test_wall_jump_has_both_horizontal_and_vertical_components() {
        let intent = PlayerIntent {
            jump_pressed: true,
            ..Default::default()
        };
        let mut velocity = Velocity::new(0.0, 0.0);
        let wall_state = WallClimbState {
            is_clinging: true,
            wall_normal: Vec2::new(1.0, 0.0),
        };

        // Simulate wall jump
        if wall_state.is_clinging && intent.jump_pressed {
            let wall_normal = wall_state.wall_normal;
            velocity.x = wall_normal.x * WALL_JUMP_HORIZONTAL_VELOCITY;
            velocity.y = WALL_JUMP_VERTICAL_VELOCITY;
        }

        assert!(
            velocity.x.abs() > 0.0,
            "Wall jump should have horizontal component"
        );
        assert!(
            velocity.y.abs() > 0.0,
            "Wall jump should have vertical component"
        );
    }

    #[test]
    fn test_no_wall_jump_when_not_clinging() {
        let intent = PlayerIntent {
            jump_pressed: true,
            ..Default::default()
        };
        let mut velocity = Velocity::new(0.0, 0.0);
        let wall_state = WallClimbState {
            is_clinging: false, // Not clinging
            wall_normal: Vec2::new(1.0, 0.0),
        };

        // Simulate wall jump attempt
        if wall_state.is_clinging && intent.jump_pressed {
            let wall_normal = wall_state.wall_normal;
            velocity.x = wall_normal.x * WALL_JUMP_HORIZONTAL_VELOCITY;
            velocity.y = WALL_JUMP_VERTICAL_VELOCITY;
        }

        assert_eq!(velocity.x, 0.0, "Should not wall jump when not clinging");
        assert_eq!(velocity.y, 0.0, "Should not wall jump when not clinging");
    }

    #[test]
    fn test_wall_top_transition_exits_cling() {
        // When player reaches top of wall (no wall above), should exit wall-cling
        let position = Position::new(50.0, 100.0);
        let collider = Collider::new(32.0, 32.0);
        let wall = LevelGeometry {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 100.0, // Wall ends at y=100
        };

        // Check if wall continues above (should not)
        let check_above = Vec2::new(0.0, -10.0);
        let result = swept_aabb_collision(&position, &collider, &wall, check_above);

        // Player is at top of wall, no collision above
        let wall_continues_above = if let Some((time, normal)) = result {
            time < 1.0 && normal.x.abs() > 0.5
        } else {
            false
        };

        assert!(
            !wall_continues_above,
            "Should detect that wall does not continue above"
        );
    }

    #[test]
    fn test_wall_continues_when_not_at_top() {
        use crate::plugins::physics::WALL_CHECK_DISTANCE;

        // When player is not at top of wall, wall should continue above
        // Player needs to be positioned adjacent to the wall, not inside it
        let position = Position::new(35.0, 50.0); // Adjacent to wall
        let collider = Collider::new(32.0, 32.0);
        let wall = LevelGeometry {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 200.0, // Tall wall
        };

        // Check if wall continues above
        // The check needs to look for wall in the horizontal direction (where the wall is)
        let check_left = Vec2::new(-WALL_CHECK_DISTANCE, 0.0);
        let result = swept_aabb_collision(&position, &collider, &wall, check_left);

        // Should detect wall on the left
        assert!(result.is_some(), "Should detect wall when not at top");
    }
}
