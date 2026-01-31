use serde::{Deserialize, Serialize};

/// Ability enum - different power-ups the player can unlock
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ability {
    HighJump,
    WallClimb,
    Swing,
}

/// Player movement state - tracks current movement mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerMovementState {
    Grounded,
    Airborne,
    WallCling,
    Swinging,
}

/// Animation type - different sprite animations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationType {
    Idle,
    Running,
    Jumping,
    Falling,
    WallCling,
    Swinging,
}
