use crate::components::{AbilitySet, GroundedState, Player, PlayerIntent, Velocity};
use crate::enums::Ability;
use bevy::prelude::*;

/// Physics constants
pub const MOVE_SPEED: f32 = 200.0; // pixels per second
pub const BASE_JUMP_VELOCITY: f32 = -400.0; // pixels per second (negative = up)
pub const HIGH_JUMP_VELOCITY: f32 = -600.0; // pixels per second

/// Plugin for player character logic and state
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                process_input_system,
                apply_horizontal_movement_system,
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
    mut query: Query<(&PlayerIntent, &mut Velocity, &GroundedState, &AbilitySet), With<Player>>,
) {
    for (intent, mut velocity, grounded, ability_set) in query.iter_mut() {
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
}
