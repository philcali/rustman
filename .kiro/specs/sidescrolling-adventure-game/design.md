# Design Document: Sidescrolling Adventure Game

## Overview

This design describes a retro-styled sidescrolling adventure game built with Rust and the Bevy game engine. The game implements a metroidvania-style progression system where players unlock abilities that grant access to new areas. The architecture leverages Bevy's Entity Component System (ECS) for clean separation of concerns and high performance.

The core gameplay loop involves:
1. Exploring interconnected 2D levels
2. Collecting power-ups that unlock new movement abilities
3. Using new abilities to access previously unreachable areas
4. Saving progress at checkpoints

Key technical challenges include:
- Implementing deterministic physics for platforming precision
- Managing complex state transitions for multiple movement modes
- Designing a flexible ability system that can be extended
- Creating smooth camera behavior that enhances gameplay

## Architecture

### High-Level Architecture

The game follows Bevy's plugin-based architecture, organizing functionality into modular plugins:

```
GamePlugin (root)
├── InputPlugin (keyboard/gamepad input handling)
├── PhysicsPlugin (collision detection, movement, gravity)
├── PlayerPlugin (player character logic and state)
├── AbilityPlugin (ability unlocking and usage)
├── LevelPlugin (level loading, transitions, geometry)
├── CameraPlugin (camera following and constraints)
├── CheckpointPlugin (save/load game state)
├── AnimationPlugin (sprite animation state machine)
└── RenderPlugin (sprite rendering, visual effects)
```

### Bevy ECS Organization

**Components** (data):
- `Player` - marker component for player entity
- `Position` - world position (x, y)
- `Velocity` - current velocity vector
- `Collider` - collision shape and properties
- `GroundedState` - whether entity is on ground
- `AbilitySet` - set of unlocked abilities
- `AnimationState` - current animation state
- `SwingState` - swing attachment point and rope length
- `WallClimbState` - wall-cling status and wall normal
- `Checkpoint` - checkpoint marker and ID
- `LevelGeometry` - static level collision data

**Systems** (behavior):
- Input processing systems (run in PreUpdate stage)
- Physics systems (run in Update stage with fixed timestep)
- Animation systems (run in Update stage)
- Camera systems (run in PostUpdate stage)
- Rendering systems (run in Render stage)

**Resources** (global state):
- `GameState` - current game state (playing, paused, etc.)
- `SaveData` - persistent save data
- `LevelData` - current level information
- `InputMap` - key bindings configuration

### Data Flow

```
Input Events → Input Systems → Player Intent
                                    ↓
                            Ability Systems (check unlocks)
                                    ↓
                            Physics Systems (apply forces, detect collisions)
                                    ↓
                            Position/Velocity Updates
                                    ↓
                            Animation Systems (update sprites)
                                    ↓
                            Camera Systems (follow player)
                                    ↓
                            Render Systems (draw frame)
```

## Components and Interfaces

### Core Components

```rust
// Position component - world coordinates
struct Position {
    x: f32,
    y: f32,
}

// Velocity component - pixels per second
struct Velocity {
    x: f32,
    y: f32,
}

// Collider component - axis-aligned bounding box
struct Collider {
    width: f32,
    height: f32,
    offset_x: f32,
    offset_y: f32,
}

// Grounded state - tracks ground contact
struct GroundedState {
    is_grounded: bool,
    ground_normal: Vec2,
}

// Ability set - tracks unlocked abilities
struct AbilitySet {
    abilities: HashSet<Ability>,
}

enum Ability {
    HighJump,
    WallClimb,
    Swing,
}

// Swing state - active swing data
struct SwingState {
    anchor_point: Vec2,
    rope_length: f32,
    angular_velocity: f32,
}

// Wall climb state - active climb data
struct WallClimbState {
    is_clinging: bool,
    wall_normal: Vec2,
}

// Animation state - current animation
struct AnimationState {
    current: AnimationType,
    frame: usize,
    timer: f32,
}

enum AnimationType {
    Idle,
    Running,
    Jumping,
    Falling,
    WallCling,
    Swinging,
}
```

### System Interfaces

```rust
// Input system - processes keyboard/gamepad input
fn process_input_system(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut PlayerIntent, &AbilitySet)>,
) {
    // Read input and set player intent based on unlocked abilities
}

// Physics system - applies forces and resolves collisions
fn physics_system(
    time: Res<Time>,
    level: Res<LevelData>,
    mut query: Query<(&mut Position, &mut Velocity, &Collider, &mut GroundedState)>,
) {
    // Apply gravity, integrate velocity, detect and resolve collisions
}

// Ability system - handles ability-specific logic
fn ability_system(
    mut query: Query<(&PlayerIntent, &AbilitySet, &mut Velocity, &Position, &GroundedState)>,
) {
    // Execute ability logic based on player intent and unlocked abilities
}

// Camera system - follows player with smoothing
fn camera_follow_system(
    player_query: Query<&Position, With<Player>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    level: Res<LevelData>,
) {
    // Smoothly move camera to follow player within level bounds
}
```

## Data Models

### Level Data Format

Levels are stored as JSON files with the following structure:

```json
{
  "id": "level_01",
  "width": 1920,
  "height": 1080,
  "spawn_point": {"x": 100, "y": 500},
  "geometry": [
    {
      "type": "platform",
      "x": 0,
      "y": 0,
      "width": 1920,
      "height": 64
    }
  ],
  "swing_points": [
    {"x": 500, "y": 800}
  ],
  "checkpoints": [
    {"id": "cp_01", "x": 300, "y": 100}
  ],
  "power_ups": [
    {"type": "HighJump", "x": 800, "y": 200}
  ],
  "transitions": [
    {
      "to_level": "level_02",
      "trigger_area": {"x": 1800, "y": 100, "width": 64, "height": 200},
      "spawn_point": {"x": 100, "y": 500}
    }
  ]
}
```

### Save Data Format

Save data is serialized to disk using serde:

```rust
struct SaveData {
    checkpoint_id: String,
    checkpoint_level: String,
    checkpoint_position: Position,
    unlocked_abilities: HashSet<Ability>,
    timestamp: u64,
}
```

### Physics Constants

```rust
const GRAVITY: f32 = 980.0; // pixels per second squared
const BASE_JUMP_VELOCITY: f32 = -400.0; // pixels per second (negative = up)
const HIGH_JUMP_VELOCITY: f32 = -600.0; // pixels per second
const MOVE_SPEED: f32 = 200.0; // pixels per second
const WALL_CLIMB_SPEED: f32 = 150.0; // pixels per second
const SWING_DAMPING: f32 = 0.98; // angular velocity damping per frame
const CAMERA_FOLLOW_SPEED: f32 = 3.0; // interpolation factor
```

## Collision Detection Algorithm

The collision detection system uses axis-aligned bounding box (AABB) collision with swept collision detection to prevent tunneling:

1. **Broad Phase**: Check if entity's movement bounding box intersects with any level geometry
2. **Narrow Phase**: For each potential collision:
   - Calculate time of impact using swept AABB
   - Find earliest collision
   - Resolve by moving entity to collision point
   - Adjust velocity based on collision normal
3. **Ground Detection**: After collision resolution, check if entity is on ground by testing for collision below with small epsilon

```rust
fn resolve_collision(
    position: &mut Position,
    velocity: &mut Velocity,
    collider: &Collider,
    level_geometry: &[LevelGeometry],
    delta_time: f32,
) -> CollisionResult {
    // Swept AABB collision detection
    let movement = Vec2::new(velocity.x * delta_time, velocity.y * delta_time);
    let mut earliest_collision = None;
    let mut earliest_time = 1.0;
    
    for geometry in level_geometry {
        if let Some((time, normal)) = swept_aabb_collision(position, collider, geometry, movement) {
            if time < earliest_time {
                earliest_time = time;
                earliest_collision = Some((geometry, normal));
            }
        }
    }
    
    if let Some((geometry, normal)) = earliest_collision {
        // Move to collision point
        position.x += velocity.x * delta_time * earliest_time;
        position.y += velocity.y * delta_time * earliest_time;
        
        // Project velocity along collision surface
        let dot = velocity.x * normal.x + velocity.y * normal.y;
        velocity.x -= dot * normal.x;
        velocity.y -= dot * normal.y;
        
        CollisionResult::Collided(normal)
    } else {
        // No collision, move full distance
        position.x += velocity.x * delta_time;
        position.y += velocity.y * delta_time;
        CollisionResult::NoCollision
    }
}
```

## Swing Physics Implementation

The swing mechanic uses pendulum physics with player input influence:

```rust
fn update_swing_physics(
    swing_state: &mut SwingState,
    position: &mut Position,
    velocity: &mut Velocity,
    player_input: &PlayerIntent,
    delta_time: f32,
) {
    // Calculate current angle from anchor point
    let dx = position.x - swing_state.anchor_point.x;
    let dy = position.y - swing_state.anchor_point.y;
    let angle = dy.atan2(dx);
    
    // Apply pendulum physics: angular acceleration = (g / L) * sin(θ)
    let angular_acceleration = (GRAVITY / swing_state.rope_length) * angle.sin();
    swing_state.angular_velocity += angular_acceleration * delta_time;
    
    // Apply player input as torque
    if player_input.move_left {
        swing_state.angular_velocity -= 2.0 * delta_time;
    }
    if player_input.move_right {
        swing_state.angular_velocity += 2.0 * delta_time;
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
```

## State Machine for Player Movement

The player character uses a state machine to manage different movement modes:

```
States:
- Grounded (can walk, jump)
- Airborne (can air control, variable jump)
- WallCling (can climb, wall jump)
- Swinging (pendulum physics, can release)

Transitions:
- Grounded → Airborne: jump pressed OR no ground contact
- Airborne → Grounded: ground collision detected
- Airborne → WallCling: wall contact AND wall climb unlocked AND input toward wall
- WallCling → Airborne: jump pressed OR input away from wall
- Airborne → Swinging: swing input AND swing point in range AND swing unlocked
- Swinging → Airborne: swing release pressed
- Swinging → Grounded: ground collision while swinging
```

Implementation:

```rust
enum PlayerMovementState {
    Grounded,
    Airborne,
    WallCling,
    Swinging,
}

fn update_player_state(
    current_state: &mut PlayerMovementState,
    grounded: &GroundedState,
    abilities: &AbilitySet,
    input: &PlayerIntent,
    nearby_walls: &[Vec2],
    nearby_swing_points: &[Vec2],
) {
    match current_state {
        PlayerMovementState::Grounded => {
            if input.jump_pressed {
                *current_state = PlayerMovementState::Airborne;
            } else if !grounded.is_grounded {
                *current_state = PlayerMovementState::Airborne;
            }
        }
        PlayerMovementState::Airborne => {
            if grounded.is_grounded {
                *current_state = PlayerMovementState::Grounded;
            } else if abilities.has(Ability::WallClimb) && !nearby_walls.is_empty() && input.move_toward_wall {
                *current_state = PlayerMovementState::WallCling;
            } else if abilities.has(Ability::Swing) && !nearby_swing_points.is_empty() && input.swing_pressed {
                *current_state = PlayerMovementState::Swinging;
            }
        }
        PlayerMovementState::WallCling => {
            if input.jump_pressed || input.move_away_from_wall {
                *current_state = PlayerMovementState::Airborne;
            }
        }
        PlayerMovementState::Swinging => {
            if input.swing_released {
                *current_state = PlayerMovementState::Airborne;
            } else if grounded.is_grounded {
                *current_state = PlayerMovementState::Grounded;
            }
        }
    }
}
```



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property Reflection

After analyzing all acceptance criteria, I identified several areas of redundancy:

1. **Save/Load Properties**: Requirements 5.3 and 5.4 both test round-trip serialization. These can be combined into a single comprehensive property.
2. **Swing Physics Properties**: Requirements 6.2 and 6.5 both verify physics calculations. Energy conservation subsumes arc length calculations.
3. **Animation State Properties**: Requirements 8.1 and 8.5 both test animation matching movement state. One comprehensive property covers both.
4. **Camera Visibility Properties**: Requirements 9.1 and 9.5 both ensure player visibility. These can be combined into one property about maintaining visibility under all conditions.

The following properties represent the minimal set needed for comprehensive correctness validation:

### Movement Properties

**Property 1: Consistent horizontal movement speed**
*For any* player input sequence containing left/right movement keys, the Player_Character horizontal velocity should equal the base movement speed (or zero when no keys pressed) when in grounded state.
**Validates: Requirements 1.1**

**Property 2: Fixed jump initial velocity**
*For any* game state where the Player_Character is grounded, pressing jump should set vertical velocity to the fixed jump velocity constant.
**Validates: Requirements 1.2**

**Property 3: Variable jump height**
*For any* jump in progress (negative vertical velocity), releasing the jump key should reduce the upward velocity, resulting in a shorter jump than holding the key.
**Validates: Requirements 1.3**

**Property 4: Consistent gravity application**
*For any* airborne Player_Character state, the vertical velocity should decrease by GRAVITY * delta_time each physics frame.
**Validates: Requirements 1.4**

**Property 5: No terrain tunneling**
*For any* Player_Character position and velocity, after physics update, the Player_Character collider should not intersect with any solid terrain geometry.
**Validates: Requirements 1.5, 4.2**

### Ability System Properties

**Property 6: Ability collection adds to set**
*For any* ability power-up, collecting it should result in that ability being present in the Player_Character's AbilitySet.
**Validates: Requirements 2.1**

**Property 7: Ability persistence round-trip**
*For any* AbilitySet state, saving to disk then loading from disk should produce an equivalent AbilitySet.
**Validates: Requirements 2.2, 5.3**

**Property 8: High jump increases velocity**
*For any* Player_Character with high jump unlocked, jump velocity magnitude should be greater than base jump velocity.
**Validates: Requirements 2.3**

**Property 9: Wall climb gated by ability**
*For any* Player_Character adjacent to a wall, wall-cling state should only be enterable when wall climb ability is unlocked.
**Validates: Requirements 2.4, 7.1**

**Property 10: Swing gated by ability**
*For any* Player_Character near a swing point, swing state should only be enterable when swing ability is unlocked.
**Validates: Requirements 2.5, 6.1**

**Property 11: Locked abilities cannot be used**
*For any* ability not in the Player_Character's AbilitySet, attempting to use that ability should have no effect on game state.
**Validates: Requirements 2.6**

### Level and Progression Properties

**Property 12: Level data round-trip**
*For any* valid level data structure, serializing to JSON then deserializing should produce an equivalent level data structure.
**Validates: Requirements 3.1**

**Property 13: State preservation across level transitions**
*For any* level transition, the Player_Character's AbilitySet and checkpoint data should remain unchanged after the transition.
**Validates: Requirements 3.2**

**Property 14: Ability-gated area access**
*For any* area requiring a specific ability, the Player_Character should be able to enter if and only if that ability is unlocked.
**Validates: Requirements 3.3**

**Property 15: Correct spawn point after transition**
*For any* level transition, the Player_Character position after loading the new level should match the transition's designated spawn point.
**Validates: Requirements 3.5**

### Physics Properties

**Property 16: Velocity projection on collision**
*For any* collision with surface normal N, the Player_Character velocity component along N should be zero after collision resolution.
**Validates: Requirements 4.3**

**Property 17: Slope physics**
*For any* Player_Character on a slope with angle θ, the sliding acceleration should equal GRAVITY * sin(θ) along the slope direction.
**Validates: Requirements 4.4**

**Property 18: Deterministic physics**
*For any* input sequence, running the physics simulation twice with identical initial conditions should produce identical final states.
**Validates: Requirements 4.5**

### Checkpoint and Save Properties

**Property 19: Complete checkpoint save**
*For any* checkpoint activation, the saved GameState should contain the Player_Character position, all unlocked abilities, and the checkpoint ID.
**Validates: Requirements 5.1**

**Property 20: Checkpoint restore correctness**
*For any* saved checkpoint state, restoring from that checkpoint should set the Player_Character position and AbilitySet to match the saved values.
**Validates: Requirements 5.2**

**Property 21: Save data round-trip**
*For any* GameState, saving to disk then loading from disk should produce an equivalent GameState (position, abilities, checkpoint).
**Validates: Requirements 5.3, 5.4**

### Swing Mechanic Properties

**Property 22: Swing energy conservation**
*For any* swing state, the total energy (kinetic + potential) should remain approximately constant over time, accounting for damping factor.
**Validates: Requirements 6.2, 6.5**

**Property 23: Swing momentum preservation on release**
*For any* swing release, the Player_Character velocity magnitude immediately after release should approximately equal the tangential velocity during swing.
**Validates: Requirements 6.3**

**Property 24: Swing input affects angular velocity**
*For any* swing state, applying directional input should change the angular velocity in the corresponding direction.
**Validates: Requirements 6.4**

### Wall Climb Properties

**Property 25: Wall-cling negates gravity**
*For any* Player_Character in wall-cling state, vertical velocity should be determined by player input, not gravity (gravity force should be zero).
**Validates: Requirements 7.2**

**Property 26: Wall jump velocity direction**
*For any* wall jump, the resulting velocity should have a horizontal component away from the wall and an upward vertical component.
**Validates: Requirements 7.3**

**Property 27: Climb speed less than ground speed**
*For any* Player_Character in wall-cling state, the vertical movement speed should be less than the horizontal ground movement speed.
**Validates: Requirements 7.4**

**Property 28: Wall top transition**
*For any* Player_Character in wall-cling state, reaching the top of the wall (no more wall above) should transition to grounded state.
**Validates: Requirements 7.5**

### Animation and Visual Properties

**Property 29: Animation matches movement state**
*For any* Player_Character movement state, the current AnimationState should correspond to that movement state (grounded→running/idle, airborne→jumping/falling, wall-cling→wall-cling animation, swinging→swinging animation).
**Validates: Requirements 8.1, 8.5**

**Property 30: Ability usage spawns visual effect**
*For any* ability activation, a visual effect entity should be spawned in the game world.
**Validates: Requirements 8.2**

**Property 31: Power-up collection triggers feedback**
*For any* power-up collection, both a collection animation and visual effect should be triggered.
**Validates: Requirements 8.3**

**Property 32: Sprite position matches entity position**
*For any* rendered frame, the Player_Character sprite position should match the Player_Character entity Position component, and sprite facing direction should match velocity direction.
**Validates: Requirements 8.4**

### Camera Properties

**Property 33: Player always visible**
*For any* Player_Character position and velocity, the camera should be positioned such that the Player_Character remains within the viewport bounds.
**Validates: Requirements 9.1, 9.5**

**Property 34: Smooth camera following**
*For any* Player_Character movement, the camera position should change gradually using interpolation, not instantly snap to the target position.
**Validates: Requirements 9.2**

**Property 35: Camera bounds constraint**
*For any* camera position, the viewport should not show any area outside the level boundaries.
**Validates: Requirements 9.3**

**Property 36: Consistent viewport scale**
*For any* screen resolution, the ratio of game world units to screen pixels should remain constant.
**Validates: Requirements 9.4**



## Error Handling

### Level Loading Errors

**Invalid JSON Format**:
- Detection: JSON parsing fails during level load
- Response: Log error with file path and parse error details
- Recovery: Fall back to a default "error level" that allows player to return to main menu
- User Feedback: Display error message overlay with file name

**Missing Required Fields**:
- Detection: Level data missing required fields (spawn_point, geometry, etc.)
- Response: Log error with missing field names
- Recovery: Use default values for missing fields where possible, otherwise fall back to error level
- User Feedback: Display warning about incomplete level data

**Invalid Geometry Data**:
- Detection: Geometry coordinates outside valid ranges or malformed
- Response: Log warning and skip invalid geometry entries
- Recovery: Load level with valid geometry only
- User Feedback: Log warning (no user-facing error unless level is unplayable)

### Save/Load Errors

**Corrupted Save File**:
- Detection: Deserialization fails or checksum mismatch
- Response: Log error with save file path
- Recovery: Ignore corrupted save, start new game from beginning
- User Feedback: Display message "Save file corrupted, starting new game"

**Missing Save File**:
- Detection: File not found when attempting to load
- Response: Log info message
- Recovery: Start new game from beginning
- User Feedback: No error (expected for first-time players)

**Disk Write Failure**:
- Detection: File write operation fails during save
- Response: Log error with IO error details
- Recovery: Retry save operation once, then continue without saving
- User Feedback: Display warning "Failed to save progress"

### Physics Edge Cases

**High Velocity Tunneling**:
- Detection: Swept collision detection finds no collision but entity would pass through thin geometry
- Response: Clamp velocity to maximum safe value based on geometry thickness
- Recovery: Limit velocity to prevent tunneling
- User Feedback: None (invisible to player)

**Stuck in Geometry**:
- Detection: Entity position intersects geometry after collision resolution
- Response: Push entity out along shortest path to valid position
- Recovery: Teleport to nearest valid position
- User Feedback: None (should be rare with proper collision detection)

**Infinite Swing Energy**:
- Detection: Swing energy exceeds physical maximum for rope length
- Response: Clamp angular velocity to maximum physically possible value
- Recovery: Limit swing speed to prevent unrealistic behavior
- User Feedback: None (feels like natural swing limit)

### Input Handling Errors

**Conflicting Input States**:
- Detection: Multiple movement directions pressed simultaneously
- Response: Prioritize most recent input or use vector sum
- Recovery: Continue with resolved input
- User Feedback: None (natural input handling)

**Rapid State Transitions**:
- Detection: Player attempts state transition before previous transition completes
- Response: Queue or ignore new transition until current one completes
- Recovery: Use state machine with transition guards
- User Feedback: None (prevents animation glitches)

## Testing Strategy

### Dual Testing Approach

This project requires both **unit tests** and **property-based tests** for comprehensive coverage:

- **Unit tests**: Verify specific examples, edge cases, and error conditions
- **Property tests**: Verify universal properties across all inputs using randomized testing

Both approaches are complementary and necessary. Unit tests catch concrete bugs in specific scenarios, while property tests verify general correctness across a wide input space.

### Property-Based Testing Configuration

**Library**: Use `proptest` crate for Rust property-based testing

**Configuration**:
- Minimum 100 iterations per property test (due to randomization)
- Each property test must reference its design document property
- Tag format: `// Feature: sidescrolling-adventure-game, Property N: [property text]`
- Each correctness property must be implemented by a SINGLE property-based test

**Example Property Test Structure**:

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    
    // Feature: sidescrolling-adventure-game, Property 1: Consistent horizontal movement speed
    proptest! {
        #[test]
        fn test_consistent_horizontal_movement(
            initial_pos in 0.0f32..1000.0,
            move_right in prop::bool::ANY,
            move_left in prop::bool::ANY,
        ) {
            let mut player = create_test_player(initial_pos);
            let input = PlayerInput { move_right, move_left, ..Default::default() };
            
            apply_movement(&mut player, &input, 0.016);
            
            let expected_velocity = if move_right && !move_left {
                MOVE_SPEED
            } else if move_left && !move_right {
                -MOVE_SPEED
            } else {
                0.0
            };
            
            assert!((player.velocity.x - expected_velocity).abs() < 0.01);
        }
    }
}
```

### Unit Testing Strategy

**Focus Areas for Unit Tests**:
1. **Specific edge cases**: Empty level data, zero-length rope, wall at level boundary
2. **Error conditions**: Invalid JSON, corrupted save files, missing assets
3. **Integration points**: Level transitions, checkpoint activation, ability unlocking
4. **State machine transitions**: Specific sequences like grounded→airborne→wall-cling→airborne

**Example Unit Test**:

```rust
#[test]
fn test_wall_jump_from_right_wall() {
    let mut player = create_test_player_at_wall(WallSide::Right);
    player.state = PlayerMovementState::WallCling;
    player.wall_normal = Vec2::new(-1.0, 0.0); // Wall on right
    
    let input = PlayerInput { jump_pressed: true, ..Default::default() };
    update_player_state(&mut player, &input);
    
    assert_eq!(player.state, PlayerMovementState::Airborne);
    assert!(player.velocity.x < 0.0); // Moving left (away from wall)
    assert!(player.velocity.y < 0.0); // Moving up
}
```

### Test Organization

```
src/
├── player/
│   ├── mod.rs
│   ├── movement.rs
│   └── tests/
│       ├── movement_tests.rs (unit tests)
│       └── movement_properties.rs (property tests)
├── physics/
│   ├── mod.rs
│   ├── collision.rs
│   └── tests/
│       ├── collision_tests.rs (unit tests)
│       └── collision_properties.rs (property tests)
└── abilities/
    ├── mod.rs
    ├── swing.rs
    └── tests/
        ├── swing_tests.rs (unit tests)
        └── swing_properties.rs (property tests)
```

### Integration Testing

**Bevy System Testing**:
- Use Bevy's `App::update()` for integration tests
- Create test worlds with minimal required components
- Verify system interactions and event propagation

**Example Integration Test**:

```rust
#[test]
fn test_checkpoint_save_and_restore() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugin(CheckpointPlugin);
    
    // Create player with abilities
    let player = app.world.spawn()
        .insert(Player)
        .insert(Position::new(100.0, 200.0))
        .insert(AbilitySet::from([Ability::HighJump]))
        .id();
    
    // Activate checkpoint
    app.world.send_event(CheckpointActivated { checkpoint_id: "cp_01" });
    app.update();
    
    // Modify player state
    let mut player_pos = app.world.get_mut::<Position>(player).unwrap();
    player_pos.x = 500.0;
    
    // Restore from checkpoint
    app.world.send_event(RestoreCheckpoint);
    app.update();
    
    // Verify restoration
    let player_pos = app.world.get::<Position>(player).unwrap();
    assert_eq!(player_pos.x, 100.0);
    assert_eq!(player_pos.y, 200.0);
}
```

### Performance Testing

While not part of automated testing, performance should be monitored:
- Target: 60 FPS on mid-range hardware
- Profile physics system (most expensive)
- Monitor entity count and system execution time
- Use Bevy's built-in diagnostics for frame time tracking

### Test Coverage Goals

- **Unit test coverage**: 70%+ of non-trivial logic
- **Property test coverage**: All 36 correctness properties implemented
- **Integration test coverage**: All major feature interactions (level transitions, ability unlocking, checkpoint system)
- **Edge case coverage**: All error conditions in Error Handling section

