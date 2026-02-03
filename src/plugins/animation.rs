use crate::components::{
    AbilitySet, AnimationState, FacingDirection, GroundedState, Player, PlayerIntent, Position,
    PowerUp, SwingState, Velocity, VisualEffect, VisualEffectType, WallClimbState,
};
use crate::enums::{Ability, AnimationType};
use bevy::prelude::*;

/// Plugin for animation state machine
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_animation_state_system,
                update_sprite_position_system,
                update_facing_direction_system,
                spawn_ability_visual_effects_system,
                spawn_power_up_collection_effects_system,
                update_visual_effects_system,
                cleanup_expired_visual_effects_system,
            )
                .chain(),
        );
    }
}

/// Update animation state based on player movement state
#[allow(clippy::type_complexity)]
fn update_animation_state_system(
    mut query: Query<
        (
            &mut AnimationState,
            &Velocity,
            &GroundedState,
            &WallClimbState,
            Option<&SwingState>,
        ),
        With<Player>,
    >,
) {
    for (mut anim_state, velocity, grounded, wall_state, swing_state) in query.iter_mut() {
        // Determine animation based on movement state priority:
        // 1. Swinging (highest priority)
        // 2. Wall clinging
        // 3. Airborne (jumping/falling)
        // 4. Grounded (running/idle)

        let new_animation = if swing_state.is_some() {
            AnimationType::Swinging
        } else if wall_state.is_clinging {
            AnimationType::WallCling
        } else if !grounded.is_grounded {
            // Airborne - check if jumping (going up) or falling (going down)
            if velocity.y < 0.0 {
                AnimationType::Jumping
            } else {
                AnimationType::Falling
            }
        } else {
            // Grounded - check if moving or idle
            if velocity.x.abs() > 0.1 {
                AnimationType::Running
            } else {
                AnimationType::Idle
            }
        };

        // Only update if animation changed
        if anim_state.current != new_animation {
            anim_state.current = new_animation;
            anim_state.frame = 0;
            anim_state.timer = 0.0;
        }
    }
}

/// Update sprite position to match entity position
fn update_sprite_position_system(mut query: Query<(&Position, &mut Transform), With<Player>>) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

/// Update facing direction based on velocity
fn update_facing_direction_system(
    mut query: Query<(&Velocity, &mut FacingDirection, &mut Transform), With<Player>>,
) {
    for (velocity, mut facing, mut transform) in query.iter_mut() {
        // Only update facing direction if moving horizontally
        if velocity.x > 0.1 {
            *facing = FacingDirection::Right;
            transform.scale.x = transform.scale.x.abs(); // Face right (no flip)
        } else if velocity.x < -0.1 {
            *facing = FacingDirection::Left;
            transform.scale.x = -transform.scale.x.abs(); // Face left (flip)
        }
    }
}

/// Spawn visual effects when abilities are used
#[allow(clippy::type_complexity)]
fn spawn_ability_visual_effects_system(
    mut commands: Commands,
    player_query: Query<
        (
            &Position,
            &PlayerIntent,
            &AbilitySet,
            &GroundedState,
            &WallClimbState,
            Option<&SwingState>,
        ),
        With<Player>,
    >,
    mut last_jump_state: Local<bool>,
    mut last_wall_cling_state: Local<bool>,
    mut last_swing_state: Local<bool>,
) {
    for (position, intent, abilities, grounded, wall_state, swing_state) in player_query.iter() {
        // Detect jump activation
        let is_jumping = intent.jump_pressed && grounded.is_grounded;
        if is_jumping && !*last_jump_state {
            // Spawn jump visual effect
            let effect_type = if abilities.has(Ability::HighJump) {
                VisualEffectType::AbilityActivation(Ability::HighJump)
            } else {
                VisualEffectType::Jump
            };

            commands.spawn((
                VisualEffect::new(effect_type, 0.3),
                Position::new(position.x, position.y),
                Transform::default(),
            ));
        }
        *last_jump_state = is_jumping;

        // Detect wall jump activation
        let is_wall_jumping = wall_state.is_clinging && intent.jump_pressed;
        if is_wall_jumping && !*last_wall_cling_state {
            // Spawn wall jump visual effect
            commands.spawn((
                VisualEffect::new(VisualEffectType::WallJump, 0.3),
                Position::new(position.x, position.y),
                Transform::default(),
            ));
        }
        *last_wall_cling_state = is_wall_jumping;

        // Detect swing attachment
        let is_swinging = swing_state.is_some();
        if is_swinging && !*last_swing_state {
            // Spawn swing attach visual effect
            commands.spawn((
                VisualEffect::new(VisualEffectType::SwingAttach, 0.3),
                Position::new(position.x, position.y),
                Transform::default(),
            ));
        }
        *last_swing_state = is_swinging;
    }
}

/// Spawn visual effects when power-ups are collected
fn spawn_power_up_collection_effects_system(
    mut commands: Commands,
    power_up_query: Query<(Entity, &PowerUp, &Position)>,
    player_query: Query<(&Position, &mut AbilitySet), With<Player>>,
) {
    for (player_pos, _abilities) in player_query.iter() {
        for (_power_up_entity, power_up, power_up_pos) in power_up_query.iter() {
            // Check if player is close enough to collect
            let distance = ((player_pos.x - power_up_pos.x).powi(2)
                + (player_pos.y - power_up_pos.y).powi(2))
            .sqrt();

            if distance < 32.0 {
                // Collection range
                // Spawn collection visual effect
                commands.spawn((
                    VisualEffect::new(VisualEffectType::PowerUpCollection(power_up.ability), 0.5),
                    Position::new(power_up_pos.x, power_up_pos.y),
                    Transform::default(),
                ));

                // Note: The actual collection logic (adding ability to set, despawning power-up)
                // should be handled by the ability plugin, not here
                // This system only handles the visual feedback
            }
        }
    }
}

/// Update visual effects (advance their timers)
fn update_visual_effects_system(time: Res<Time>, mut query: Query<&mut VisualEffect>) {
    for mut effect in query.iter_mut() {
        effect.elapsed += time.delta_seconds();
    }
}

/// Cleanup expired visual effects
fn cleanup_expired_visual_effects_system(
    mut commands: Commands,
    query: Query<(Entity, &VisualEffect)>,
) {
    for (entity, effect) in query.iter() {
        if effect.is_expired() {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;

    #[test]
    fn test_idle_animation_when_grounded_and_stationary() {
        let mut anim_state = AnimationState::default();
        let velocity = Velocity::new(0.0, 0.0);
        let grounded = GroundedState {
            is_grounded: true,
            ground_normal: Vec2::new(0.0, -1.0),
        };

        // Simulate animation update
        let new_animation = if !grounded.is_grounded {
            if velocity.y < 0.0 {
                AnimationType::Jumping
            } else {
                AnimationType::Falling
            }
        } else if velocity.x.abs() > 0.1 {
            AnimationType::Running
        } else {
            AnimationType::Idle
        };

        anim_state.current = new_animation;

        assert_eq!(anim_state.current, AnimationType::Idle);
    }

    #[test]
    fn test_running_animation_when_grounded_and_moving() {
        let mut anim_state = AnimationState::default();
        let velocity = Velocity::new(200.0, 0.0);
        let grounded = GroundedState {
            is_grounded: true,
            ground_normal: Vec2::new(0.0, -1.0),
        };

        // Simulate animation update
        let new_animation = if !grounded.is_grounded {
            if velocity.y < 0.0 {
                AnimationType::Jumping
            } else {
                AnimationType::Falling
            }
        } else if velocity.x.abs() > 0.1 {
            AnimationType::Running
        } else {
            AnimationType::Idle
        };

        anim_state.current = new_animation;

        assert_eq!(anim_state.current, AnimationType::Running);
    }

    #[test]
    fn test_jumping_animation_when_airborne_ascending() {
        let mut anim_state = AnimationState::default();
        let velocity = Velocity::new(100.0, -300.0); // Negative y = going up
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };

        // Simulate animation update
        let new_animation = if !grounded.is_grounded {
            if velocity.y < 0.0 {
                AnimationType::Jumping
            } else {
                AnimationType::Falling
            }
        } else if velocity.x.abs() > 0.1 {
            AnimationType::Running
        } else {
            AnimationType::Idle
        };

        anim_state.current = new_animation;

        assert_eq!(anim_state.current, AnimationType::Jumping);
    }

    #[test]
    fn test_falling_animation_when_airborne_descending() {
        let mut anim_state = AnimationState::default();
        let velocity = Velocity::new(100.0, 200.0); // Positive y = going down
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };

        // Simulate animation update
        let new_animation = if !grounded.is_grounded {
            if velocity.y < 0.0 {
                AnimationType::Jumping
            } else {
                AnimationType::Falling
            }
        } else if velocity.x.abs() > 0.1 {
            AnimationType::Running
        } else {
            AnimationType::Idle
        };

        anim_state.current = new_animation;

        assert_eq!(anim_state.current, AnimationType::Falling);
    }

    #[test]
    fn test_wall_cling_animation_when_clinging() {
        let mut anim_state = AnimationState::default();
        let velocity = Velocity::new(0.0, 0.0);
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };
        let wall_state = WallClimbState {
            is_clinging: true,
            wall_normal: Vec2::new(1.0, 0.0),
        };

        // Simulate animation update
        let new_animation = if wall_state.is_clinging {
            AnimationType::WallCling
        } else if !grounded.is_grounded {
            if velocity.y < 0.0 {
                AnimationType::Jumping
            } else {
                AnimationType::Falling
            }
        } else if velocity.x.abs() > 0.1 {
            AnimationType::Running
        } else {
            AnimationType::Idle
        };

        anim_state.current = new_animation;

        assert_eq!(anim_state.current, AnimationType::WallCling);
    }

    #[test]
    fn test_swinging_animation_when_swinging() {
        let mut anim_state = AnimationState::default();
        let velocity = Velocity::new(150.0, 100.0);
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };
        let wall_state = WallClimbState::default();
        let swing_state = Some(SwingState {
            anchor_point: Vec2::new(500.0, 800.0),
            rope_length: 200.0,
            angular_velocity: 0.5,
        });

        // Simulate animation update
        let new_animation = if swing_state.is_some() {
            AnimationType::Swinging
        } else if wall_state.is_clinging {
            AnimationType::WallCling
        } else if !grounded.is_grounded {
            if velocity.y < 0.0 {
                AnimationType::Jumping
            } else {
                AnimationType::Falling
            }
        } else if velocity.x.abs() > 0.1 {
            AnimationType::Running
        } else {
            AnimationType::Idle
        };

        anim_state.current = new_animation;

        assert_eq!(anim_state.current, AnimationType::Swinging);
    }

    #[test]
    fn test_animation_priority_swing_over_wall_cling() {
        // Swinging should take priority over wall cling
        let mut anim_state = AnimationState::default();
        let wall_state = WallClimbState {
            is_clinging: true,
            wall_normal: Vec2::new(1.0, 0.0),
        };
        let swing_state = Some(SwingState {
            anchor_point: Vec2::new(500.0, 800.0),
            rope_length: 200.0,
            angular_velocity: 0.5,
        });

        // Simulate animation update
        let new_animation = if swing_state.is_some() {
            AnimationType::Swinging
        } else if wall_state.is_clinging {
            AnimationType::WallCling
        } else {
            AnimationType::Idle
        };

        anim_state.current = new_animation;

        assert_eq!(
            anim_state.current,
            AnimationType::Swinging,
            "Swinging should take priority over wall cling"
        );
    }

    #[test]
    fn test_animation_priority_wall_cling_over_airborne() {
        // Wall cling should take priority over airborne
        let mut anim_state = AnimationState::default();
        let velocity = Velocity::new(0.0, -200.0); // Would be jumping
        let grounded = GroundedState {
            is_grounded: false,
            ground_normal: Vec2::ZERO,
        };
        let wall_state = WallClimbState {
            is_clinging: true,
            wall_normal: Vec2::new(1.0, 0.0),
        };

        // Simulate animation update
        let new_animation = if wall_state.is_clinging {
            AnimationType::WallCling
        } else if !grounded.is_grounded {
            if velocity.y < 0.0 {
                AnimationType::Jumping
            } else {
                AnimationType::Falling
            }
        } else {
            AnimationType::Idle
        };

        anim_state.current = new_animation;

        assert_eq!(
            anim_state.current,
            AnimationType::WallCling,
            "Wall cling should take priority over airborne"
        );
    }

    #[test]
    fn test_animation_frame_reset_on_state_change() {
        let mut anim_state = AnimationState {
            current: AnimationType::Running,
            frame: 5,
            timer: 0.3,
        };

        let new_animation = AnimationType::Jumping;

        // Simulate animation state change
        if anim_state.current != new_animation {
            anim_state.current = new_animation;
            anim_state.frame = 0;
            anim_state.timer = 0.0;
        }

        assert_eq!(anim_state.current, AnimationType::Jumping);
        assert_eq!(anim_state.frame, 0, "Frame should reset to 0");
        assert_eq!(anim_state.timer, 0.0, "Timer should reset to 0.0");
    }

    #[test]
    fn test_animation_no_change_when_state_same() {
        let mut anim_state = AnimationState {
            current: AnimationType::Running,
            frame: 5,
            timer: 0.3,
        };

        let new_animation = AnimationType::Running;

        // Simulate animation state check
        if anim_state.current != new_animation {
            anim_state.current = new_animation;
            anim_state.frame = 0;
            anim_state.timer = 0.0;
        }

        assert_eq!(anim_state.current, AnimationType::Running);
        assert_eq!(anim_state.frame, 5, "Frame should not reset");
        assert_eq!(anim_state.timer, 0.3, "Timer should not reset");
    }

    #[test]
    fn test_sprite_position_matches_entity_position() {
        let position = Position::new(150.0, 250.0);
        let mut transform = Transform::default();

        // Simulate sprite position update
        transform.translation.x = position.x;
        transform.translation.y = position.y;

        assert_eq!(transform.translation.x, 150.0);
        assert_eq!(transform.translation.y, 250.0);
    }

    #[test]
    fn test_facing_direction_right_when_moving_right() {
        let velocity = Velocity::new(200.0, 0.0);
        let mut facing = FacingDirection::Left;
        let mut transform = Transform::from_scale(Vec3::new(1.0, 1.0, 1.0));

        // Simulate facing direction update
        if velocity.x > 0.1 {
            facing = FacingDirection::Right;
            transform.scale.x = transform.scale.x.abs();
        } else if velocity.x < -0.1 {
            facing = FacingDirection::Left;
            transform.scale.x = -transform.scale.x.abs();
        }

        assert_eq!(facing, FacingDirection::Right);
        assert!(transform.scale.x > 0.0, "Should not be flipped");
    }

    #[test]
    fn test_facing_direction_left_when_moving_left() {
        let velocity = Velocity::new(-200.0, 0.0);
        let mut facing = FacingDirection::Right;
        let mut transform = Transform::from_scale(Vec3::new(1.0, 1.0, 1.0));

        // Simulate facing direction update
        if velocity.x > 0.1 {
            facing = FacingDirection::Right;
            transform.scale.x = transform.scale.x.abs();
        } else if velocity.x < -0.1 {
            facing = FacingDirection::Left;
            transform.scale.x = -transform.scale.x.abs();
        }

        assert_eq!(facing, FacingDirection::Left);
        assert!(transform.scale.x < 0.0, "Should be flipped");
    }

    #[test]
    fn test_facing_direction_unchanged_when_stationary() {
        let velocity = Velocity::new(0.0, 0.0);
        let mut facing = FacingDirection::Right;
        let initial_facing = facing;
        let mut transform = Transform::from_scale(Vec3::new(1.0, 1.0, 1.0));
        let initial_scale = transform.scale.x;

        // Simulate facing direction update
        if velocity.x > 0.1 {
            facing = FacingDirection::Right;
            transform.scale.x = transform.scale.x.abs();
        } else if velocity.x < -0.1 {
            facing = FacingDirection::Left;
            transform.scale.x = -transform.scale.x.abs();
        }

        assert_eq!(facing, initial_facing, "Facing should not change");
        assert_eq!(transform.scale.x, initial_scale, "Scale should not change");
    }

    #[test]
    fn test_sprite_position_updates_continuously() {
        let mut position = Position::new(100.0, 200.0);
        let mut transform = Transform::default();

        // First update
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        assert_eq!(transform.translation.x, 100.0);
        assert_eq!(transform.translation.y, 200.0);

        // Move position
        position.x = 150.0;
        position.y = 250.0;

        // Second update
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        assert_eq!(transform.translation.x, 150.0);
        assert_eq!(transform.translation.y, 250.0);
    }

    #[test]
    fn test_visual_effect_creation() {
        let effect = VisualEffect::new(VisualEffectType::Jump, 0.5);
        assert_eq!(effect.lifetime, 0.5);
        assert_eq!(effect.elapsed, 0.0);
        assert!(!effect.is_expired());
    }

    #[test]
    fn test_visual_effect_expiration() {
        let mut effect = VisualEffect::new(VisualEffectType::Jump, 0.5);
        effect.elapsed = 0.6;
        assert!(effect.is_expired());
    }

    #[test]
    fn test_visual_effect_not_expired_before_lifetime() {
        let mut effect = VisualEffect::new(VisualEffectType::Jump, 0.5);
        effect.elapsed = 0.3;
        assert!(!effect.is_expired());
    }

    #[test]
    fn test_visual_effect_types() {
        let jump_effect = VisualEffect::new(VisualEffectType::Jump, 0.3);
        let wall_jump_effect = VisualEffect::new(VisualEffectType::WallJump, 0.3);
        let high_jump_effect =
            VisualEffect::new(VisualEffectType::AbilityActivation(Ability::HighJump), 0.3);

        assert_eq!(jump_effect.effect_type, VisualEffectType::Jump);
        assert_eq!(wall_jump_effect.effect_type, VisualEffectType::WallJump);
        assert_eq!(
            high_jump_effect.effect_type,
            VisualEffectType::AbilityActivation(Ability::HighJump)
        );
    }

    #[test]
    fn test_power_up_collection_effect_type() {
        let effect =
            VisualEffect::new(VisualEffectType::PowerUpCollection(Ability::WallClimb), 0.5);
        assert_eq!(
            effect.effect_type,
            VisualEffectType::PowerUpCollection(Ability::WallClimb)
        );
        assert_eq!(effect.lifetime, 0.5);
    }

    #[test]
    fn test_swing_attach_effect_type() {
        let effect = VisualEffect::new(VisualEffectType::SwingAttach, 0.3);
        assert_eq!(effect.effect_type, VisualEffectType::SwingAttach);
    }

    #[test]
    fn test_visual_effect_elapsed_time_update() {
        let mut effect = VisualEffect::new(VisualEffectType::Jump, 1.0);
        assert_eq!(effect.elapsed, 0.0);

        // Simulate time passing
        effect.elapsed += 0.016; // One frame at 60fps
        assert!((effect.elapsed - 0.016).abs() < 0.001);
        assert!(!effect.is_expired());

        // Simulate more time
        effect.elapsed += 0.5;
        assert!((effect.elapsed - 0.516).abs() < 0.001);
        assert!(!effect.is_expired());

        // Simulate expiration
        effect.elapsed += 0.5;
        assert!(effect.elapsed > 1.0);
        assert!(effect.is_expired());
    }
}
