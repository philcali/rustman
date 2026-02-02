# Implementation Plan: Sidescrolling Adventure Game

## Overview

This implementation plan breaks down the sidescrolling adventure game into incremental coding tasks. Each task builds on previous work, with property-based tests integrated throughout to catch errors early. The implementation follows Bevy's plugin architecture for modularity.

## Tasks

- [x] 1. Set up project structure and core components
  - Initialize Rust project with Bevy and proptest dependencies
  - Create plugin structure (PlayerPlugin, PhysicsPlugin, AbilityPlugin, etc.)
  - Define core components (Position, Velocity, Collider, GroundedState, AbilitySet)
  - Define core enums (Ability, PlayerMovementState, AnimationType)
  - Set up basic Bevy app with minimal plugins
  - _Requirements: 10.1, 10.5_

- [ ]* 1.1 Write property test for component serialization
  - **Property 7: Ability persistence round-trip**
  - **Validates: Requirements 2.2, 5.3**

- [x] 2. Implement basic physics system
  - [x] 2.1 Create physics system with gravity and velocity integration
    - Implement gravity application for airborne entities
    - Integrate velocity to update position each frame
    - Use fixed timestep for deterministic physics
    - _Requirements: 1.4, 4.5_
  
  - [ ]* 2.2 Write property test for gravity application
    - **Property 4: Consistent gravity application**
    - **Validates: Requirements 1.4**
  
  - [ ]* 2.3 Write property test for deterministic physics
    - **Property 18: Deterministic physics**
    - **Validates: Requirements 4.5**

- [x] 3. Implement collision detection system
  - [x] 3.1 Create AABB collision detection with swept collision
    - Implement swept AABB algorithm to prevent tunneling
    - Detect collisions with level geometry
    - Resolve collisions by adjusting position and velocity
    - Update grounded state based on ground contact
    - _Requirements: 1.5, 4.1, 4.2, 4.3_
  
  - [ ]* 3.2 Write property test for no tunneling
    - **Property 5: No terrain tunneling**
    - **Validates: Requirements 1.5, 4.2**
  
  - [ ]* 3.3 Write property test for velocity projection
    - **Property 16: Velocity projection on collision**
    - **Validates: Requirements 4.3**
  
  - [x] 3.4 Implement slope physics
    - Calculate slope angle from collision normal
    - Apply friction and sliding forces on slopes
    - _Requirements: 4.4_
  
  - [ ]* 3.5 Write property test for slope physics
    - **Property 17: Slope physics**
    - **Validates: Requirements 4.4**

- [x] 4. Implement input system and basic player movement
  - [x] 4.1 Create input processing system
    - Read keyboard input for movement and jump
    - Translate input to PlayerIntent component
    - _Requirements: 1.1, 1.2_
  
  - [x] 4.2 Implement horizontal movement
    - Apply horizontal velocity based on input
    - Maintain consistent movement speed
    - _Requirements: 1.1_
  
  - [ ]* 4.3 Write property test for horizontal movement
    - **Property 1: Consistent horizontal movement speed**
    - **Validates: Requirements 1.1**
  
  - [x] 4.3 Implement basic jump mechanic
    - Apply fixed jump velocity when grounded and jump pressed
    - Implement variable jump height (reduce velocity on key release)
    - _Requirements: 1.2, 1.3_
  
  - [ ]* 4.4 Write property test for fixed jump velocity
    - **Property 2: Fixed jump initial velocity**
    - **Validates: Requirements 1.2**
  
  - [ ]* 4.5 Write property test for variable jump height
    - **Property 3: Variable jump height**
    - **Validates: Requirements 1.3**

- [x] 5. Checkpoint - Ensure basic movement and physics work
  - Run all tests to verify basic movement, jumping, and collision detection
  - Manually test player movement in a simple test level
  - Ask the user if questions arise

- [x] 6. Implement ability system
  - [x] 6.1 Create ability management system
    - Implement AbilitySet component and methods
    - Create power-up collection logic
    - Add abilities to player's set when collected
    - _Requirements: 2.1, 2.6_
  
  - [ ]* 6.2 Write property test for ability collection
    - **Property 6: Ability collection adds to set**
    - **Validates: Requirements 2.1**
  
  - [ ]* 6.3 Write property test for locked abilities
    - **Property 11: Locked abilities cannot be used**
    - **Validates: Requirements 2.6**
  
  - [x] 6.4 Implement high jump ability
    - Check for high jump in ability set
    - Apply increased jump velocity when unlocked
    - _Requirements: 2.3_
  
  - [ ]* 6.5 Write property test for high jump
    - **Property 8: High jump increases velocity**
    - **Validates: Requirements 2.3**

- [x] 7. Implement wall climb mechanic
  - [x] 7.1 Create wall detection system
    - Detect adjacent walls using raycasts or collision checks
    - Store wall normal in WallClimbState component
    - _Requirements: 2.4, 7.1_
  
  - [x] 7.2 Implement wall-cling state
    - Enter wall-cling when conditions met and ability unlocked
    - Negate gravity during wall-cling
    - Allow vertical movement input
    - _Requirements: 7.1, 7.2, 7.4_
  
  - [ ]* 7.3 Write property test for wall climb ability gating
    - **Property 9: Wall climb gated by ability**
    - **Validates: Requirements 2.4, 7.1**
  
  - [ ]* 7.4 Write property test for wall-cling gravity negation
    - **Property 25: Wall-cling negates gravity**
    - **Validates: Requirements 7.2**
  
  - [ ]* 7.5 Write property test for climb speed
    - **Property 27: Climb speed less than ground speed**
    - **Validates: Requirements 7.4**
  
  - [x] 7.6 Implement wall jump
    - Apply velocity away from wall and upward on jump press
    - Transition to airborne state
    - _Requirements: 7.3_
  
  - [ ]* 7.7 Write property test for wall jump direction
    - **Property 26: Wall jump velocity direction**
    - **Validates: Requirements 7.3**
  
  - [x] 7.8 Implement wall top transition
    - Detect when player reaches top of wall
    - Transition to grounded state
    - _Requirements: 7.5_
  
  - [ ]* 7.9 Write property test for wall top transition
    - **Property 28: Wall top transition**
    - **Validates: Requirements 7.5**

- [x] 8. Implement swing mechanic
  - [x] 8.1 Create swing point detection system
    - Detect nearby swing points
    - Check if player is within range
    - _Requirements: 2.5, 6.1_
  
  - [x] 8.2 Implement swing attachment
    - Enter swing state when conditions met and ability unlocked
    - Store anchor point and rope length in SwingState
    - _Requirements: 6.1_
  
  - [ ]* 8.3 Write property test for swing ability gating
    - **Property 10: Swing gated by ability**
    - **Validates: Requirements 2.5, 6.1**
  
  - [x] 8.4 Implement pendulum physics
    - Calculate angular acceleration based on gravity and rope length
    - Apply player input as torque
    - Update position along swing arc
    - Apply damping to angular velocity
    - _Requirements: 6.2, 6.4, 6.5_
  
  - [ ]* 8.5 Write property test for swing energy conservation
    - **Property 22: Swing energy conservation**
    - **Validates: Requirements 6.2, 6.5**
  
  - [ ]* 8.6 Write property test for swing input influence
    - **Property 24: Swing input affects angular velocity**
    - **Validates: Requirements 6.4**
  
  - [x] 8.7 Implement swing release
    - Detach from swing point on key release
    - Preserve tangential velocity as linear velocity
    - _Requirements: 6.3_
  
  - [ ]* 8.8 Write property test for momentum preservation
    - **Property 23: Swing momentum preservation on release**
    - **Validates: Requirements 6.3**

- [ ] 9. Checkpoint - Ensure all abilities work correctly
  - Run all tests to verify ability system and mechanics
  - Manually test each ability (high jump, wall climb, swing)
  - Ask the user if questions arise

- [ ] 10. Implement level loading system
  - [ ] 10.1 Create level data structures
    - Define LevelData struct matching JSON format
    - Implement serde serialization/deserialization
    - _Requirements: 3.1_
  
  - [ ]* 10.2 Write property test for level data round-trip
    - **Property 12: Level data round-trip**
    - **Validates: Requirements 3.1**
  
  - [ ] 10.3 Implement level loading from JSON
    - Load level JSON files from disk
    - Parse into LevelData structures
    - Spawn level geometry entities
    - Handle loading errors gracefully
    - _Requirements: 3.1_
  
  - [ ]* 10.4 Write unit tests for level loading errors
    - Test invalid JSON format handling
    - Test missing required fields handling
    - Test invalid geometry data handling
  
  - [ ] 10.5 Implement level transitions
    - Detect when player reaches transition trigger
    - Unload current level and load new level
    - Preserve game state across transition
    - Spawn player at correct entry point
    - _Requirements: 3.2, 3.5_
  
  - [ ]* 10.6 Write property test for state preservation
    - **Property 13: State preservation across level transitions**
    - **Validates: Requirements 3.2**
  
  - [ ]* 10.7 Write property test for spawn point correctness
    - **Property 15: Correct spawn point after transition**
    - **Validates: Requirements 3.5**
  
  - [ ] 10.8 Implement ability-gated areas
    - Check required ability for area access
    - Block or allow passage based on ability unlock
    - _Requirements: 3.3_
  
  - [ ]* 10.9 Write property test for ability gating
    - **Property 14: Ability-gated area access**
    - **Validates: Requirements 3.3**

- [ ] 11. Implement checkpoint and save system
  - [ ] 11.1 Create checkpoint activation system
    - Detect when player activates checkpoint
    - Capture current game state (position, abilities, checkpoint ID)
    - _Requirements: 5.1_
  
  - [ ]* 11.2 Write property test for checkpoint save completeness
    - **Property 19: Complete checkpoint save**
    - **Validates: Requirements 5.1**
  
  - [ ] 11.3 Implement save to disk
    - Serialize GameState to JSON
    - Write to save file on disk
    - Handle disk write errors
    - _Requirements: 5.3_
  
  - [ ]* 11.4 Write property test for save data round-trip
    - **Property 21: Save data round-trip**
    - **Validates: Requirements 5.3, 5.4**
  
  - [ ]* 11.5 Write unit tests for save/load errors
    - Test corrupted save file handling
    - Test missing save file handling
    - Test disk write failure handling
  
  - [ ] 11.6 Implement checkpoint restore
    - Load saved game state from checkpoint
    - Restore player position and abilities
    - _Requirements: 5.2, 5.4_
  
  - [ ]* 11.7 Write property test for checkpoint restore
    - **Property 20: Checkpoint restore correctness**
    - **Validates: Requirements 5.2**

- [ ] 12. Implement animation system
  - [ ] 12.1 Create animation state machine
    - Define AnimationState component
    - Map movement states to animation types
    - Update animation based on player state
    - _Requirements: 8.1, 8.5_
  
  - [ ]* 12.2 Write property test for animation state matching
    - **Property 29: Animation matches movement state**
    - **Validates: Requirements 8.1, 8.5**
  
  - [ ] 12.3 Implement sprite rendering
    - Render player sprite at correct position
    - Update facing direction based on velocity
    - _Requirements: 8.4_
  
  - [ ]* 12.4 Write property test for sprite position
    - **Property 32: Sprite position matches entity position**
    - **Validates: Requirements 8.4**
  
  - [ ] 12.5 Implement visual effects for abilities
    - Spawn visual effect entities on ability use
    - Create collection animation for power-ups
    - _Requirements: 8.2, 8.3_
  
  - [ ]* 12.6 Write property test for ability visual effects
    - **Property 30: Ability usage spawns visual effect**
    - **Validates: Requirements 8.2**
  
  - [ ]* 12.7 Write property test for power-up feedback
    - **Property 31: Power-up collection triggers feedback**
    - **Validates: Requirements 8.3**

- [ ] 13. Implement camera system
  - [ ] 13.1 Create camera following system
    - Position camera to keep player visible
    - Implement smooth interpolation with lag
    - _Requirements: 9.1, 9.2_
  
  - [ ]* 13.2 Write property test for player visibility
    - **Property 33: Player always visible**
    - **Validates: Requirements 9.1, 9.5**
  
  - [ ]* 13.3 Write property test for smooth following
    - **Property 34: Smooth camera following**
    - **Validates: Requirements 9.2**
  
  - [ ] 13.4 Implement camera bounds constraint
    - Constrain camera to level boundaries
    - Prevent showing out-of-bounds areas
    - _Requirements: 9.3_
  
  - [ ]* 13.5 Write property test for camera bounds
    - **Property 35: Camera bounds constraint**
    - **Validates: Requirements 9.3**
  
  - [ ] 13.6 Implement viewport scaling
    - Maintain consistent viewport size across resolutions
    - Calculate game units to screen pixels ratio
    - _Requirements: 9.4_
  
  - [ ]* 13.7 Write property test for viewport consistency
    - **Property 36: Consistent viewport scale**
    - **Validates: Requirements 9.4**

- [ ] 14. Create test levels and integrate all systems
  - [ ] 14.1 Create sample level JSON files
    - Design test level with platforms, walls, swing points
    - Include power-ups and checkpoints
    - Create multiple connected levels for transitions
    - _Requirements: 3.1_
  
  - [ ] 14.2 Wire all plugins together in main app
    - Add all plugins to Bevy app
    - Configure system ordering and stages
    - Set up resources and initial game state
    - _Requirements: 10.1, 10.3, 10.5_
  
  - [ ] 14.3 Implement basic retro sprite rendering
    - Load sprite assets
    - Set up sprite sheets for animations
    - Configure retro pixel art rendering settings
    - _Requirements: 8.4_

- [ ] 15. Final checkpoint - Comprehensive testing
  - Run complete test suite (all property tests and unit tests)
  - Manually playtest all abilities and level transitions
  - Verify save/load functionality
  - Test error handling scenarios
  - Ask the user if questions arise

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests validate universal correctness properties with 100+ iterations
- Unit tests validate specific examples, edge cases, and error conditions
- Checkpoints ensure incremental validation at major milestones
- All code should follow Bevy's ECS patterns and plugin architecture
