use crate::enums::{Ability, AnimationType};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Position component - world coordinates
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Velocity component - pixels per second
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Collider component - axis-aligned bounding box
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Collider {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

/// Grounded state - tracks ground contact
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct GroundedState {
    pub is_grounded: bool,
    pub ground_normal: Vec2,
}

impl Default for GroundedState {
    fn default() -> Self {
        Self {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        }
    }
}

/// Ability set - tracks unlocked abilities
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AbilitySet {
    pub abilities: HashSet<Ability>,
}

impl AbilitySet {
    pub fn new() -> Self {
        Self {
            abilities: HashSet::new(),
        }
    }

    pub fn has(&self, ability: Ability) -> bool {
        self.abilities.contains(&ability)
    }

    pub fn add(&mut self, ability: Ability) {
        self.abilities.insert(ability);
    }
}

impl Default for AbilitySet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Ability>> for AbilitySet {
    fn from(abilities: Vec<Ability>) -> Self {
        Self {
            abilities: abilities.into_iter().collect(),
        }
    }
}

/// Swing state - active swing data
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct SwingState {
    pub anchor_point: Vec2,
    pub rope_length: f32,
    pub angular_velocity: f32,
}

/// Wall climb state - active climb data
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct WallClimbState {
    pub is_clinging: bool,
    pub wall_normal: Vec2,
}

impl Default for WallClimbState {
    fn default() -> Self {
        Self {
            is_clinging: false,
            wall_normal: Vec2::ZERO,
        }
    }
}

/// Animation state - current animation
#[derive(Component, Clone, Debug, PartialEq)]
pub struct AnimationState {
    pub current: AnimationType,
    pub frame: usize,
    pub timer: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            current: AnimationType::Idle,
            frame: 0,
            timer: 0.0,
        }
    }
}

/// Player marker component
#[derive(Component)]
pub struct Player;

/// Player intent component - captures player input
#[derive(Component, Clone, Copy, Debug, PartialEq, Default)]
pub struct PlayerIntent {
    pub move_left: bool,
    pub move_right: bool,
    pub jump_pressed: bool,
    pub jump_just_released: bool,
}

/// Level geometry component - static collision data
#[derive(Component, Clone, Debug, PartialEq)]
pub struct LevelGeometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Power-up component - represents a collectible ability power-up
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct PowerUp {
    pub ability: Ability,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(100.0, 200.0);
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);
    }

    #[test]
    fn test_velocity_default() {
        let vel = Velocity::default();
        assert_eq!(vel.x, 0.0);
        assert_eq!(vel.y, 0.0);
    }

    #[test]
    fn test_ability_set_operations() {
        let mut abilities = AbilitySet::new();
        assert!(!abilities.has(Ability::HighJump));

        abilities.add(Ability::HighJump);
        assert!(abilities.has(Ability::HighJump));
        assert!(!abilities.has(Ability::WallClimb));
    }

    #[test]
    fn test_ability_set_from_vec() {
        let abilities = AbilitySet::from(vec![Ability::HighJump, Ability::Swing]);
        assert!(abilities.has(Ability::HighJump));
        assert!(abilities.has(Ability::Swing));
        assert!(!abilities.has(Ability::WallClimb));
    }

    #[test]
    fn test_grounded_state_default() {
        let grounded = GroundedState::default();
        assert!(!grounded.is_grounded);
        assert_eq!(grounded.ground_normal, Vec2::ZERO);
    }

    #[test]
    fn test_collider_creation() {
        let collider = Collider::new(32.0, 64.0);
        assert_eq!(collider.width, 32.0);
        assert_eq!(collider.height, 64.0);
        assert_eq!(collider.offset_x, 0.0);
        assert_eq!(collider.offset_y, 0.0);
    }
}
