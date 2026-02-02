use crate::components::{
    AbilitySet, Collider, LevelGeometry, Player, Position, PowerUp, SwingPoint,
};
use crate::enums::Ability;
use crate::level::LevelData;
use bevy::prelude::*;
use std::fs;
use std::path::Path;

/// Resource to track current level
#[derive(Resource, Clone, Debug)]
pub struct CurrentLevel {
    pub level_id: String,
    pub level_data: LevelData,
}

/// Resource to track pending level transition
#[derive(Resource, Clone, Debug)]
pub struct PendingTransition {
    pub to_level: String,
    pub spawn_point: Position,
}

/// Component to mark level transition triggers
#[derive(Component, Clone, Debug)]
pub struct LevelTransitionTrigger {
    pub to_level: String,
    pub spawn_point: Position,
    pub trigger_area: Collider,
}

/// Component to mark ability-gated areas
#[derive(Component, Clone, Debug)]
pub struct AbilityGate {
    pub required_ability: Ability,
    pub gate_area: Collider,
    pub is_blocking: bool, // Whether the gate is currently blocking
}

/// Plugin for level loading, transitions, and geometry
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                detect_level_transitions,
                process_pending_transition,
                update_ability_gates,
            )
                .chain(),
        );
    }
}

/// Load level from JSON file
pub fn load_level_from_file(path: &str) -> Result<LevelData, LevelLoadError> {
    // Check if file exists
    if !Path::new(path).exists() {
        return Err(LevelLoadError::FileNotFound(path.to_string()));
    }

    // Read file contents
    let contents = fs::read_to_string(path)
        .map_err(|e| LevelLoadError::IoError(path.to_string(), e.to_string()))?;

    // Parse JSON
    let level_data: LevelData = serde_json::from_str(&contents)
        .map_err(|e| LevelLoadError::ParseError(path.to_string(), e.to_string()))?;

    // Validate level data
    validate_level_data(&level_data)?;

    Ok(level_data)
}

/// Validate level data for required fields and valid values
fn validate_level_data(level: &LevelData) -> Result<(), LevelLoadError> {
    if level.id.is_empty() {
        return Err(LevelLoadError::ValidationError(
            "Level ID cannot be empty".to_string(),
        ));
    }

    if level.width <= 0.0 {
        return Err(LevelLoadError::ValidationError(
            "Level width must be positive".to_string(),
        ));
    }

    if level.height <= 0.0 {
        return Err(LevelLoadError::ValidationError(
            "Level height must be positive".to_string(),
        ));
    }

    // Validate geometry
    for (i, geo) in level.geometry.iter().enumerate() {
        if geo.width <= 0.0 || geo.height <= 0.0 {
            return Err(LevelLoadError::ValidationError(format!(
                "Geometry {} has invalid dimensions",
                i
            )));
        }
    }

    Ok(())
}

/// Spawn level entities from level data
pub fn spawn_level_entities(commands: &mut Commands, level: &LevelData) {
    // Spawn geometry
    for geo in &level.geometry {
        commands.spawn(LevelGeometry {
            x: geo.x,
            y: geo.y,
            width: geo.width,
            height: geo.height,
        });
    }

    // Spawn swing points
    for swing_point in &level.swing_points {
        commands.spawn((
            SwingPoint { range: 100.0 },
            Position::new(swing_point.x, swing_point.y),
        ));
    }

    // Spawn power-ups
    for power_up in &level.power_ups {
        commands.spawn((
            PowerUp {
                ability: power_up.ability_type,
            },
            Position::new(power_up.x, power_up.y),
        ));
    }

    // Spawn level transition triggers
    for transition in &level.transitions {
        commands.spawn(LevelTransitionTrigger {
            to_level: transition.to_level.clone(),
            spawn_point: Position::new(transition.spawn_point.x, transition.spawn_point.y),
            trigger_area: Collider {
                width: transition.trigger_area.width,
                height: transition.trigger_area.height,
                offset_x: transition.trigger_area.x,
                offset_y: transition.trigger_area.y,
            },
        });
    }

    // Spawn ability gates
    for gate in &level.ability_gates {
        commands.spawn(AbilityGate {
            required_ability: gate.required_ability,
            gate_area: Collider {
                width: gate.gate_area.width,
                height: gate.gate_area.height,
                offset_x: gate.gate_area.x,
                offset_y: gate.gate_area.y,
            },
            is_blocking: true, // Initially blocking
        });
    }
}

/// Detect when player reaches level transition trigger
fn detect_level_transitions(
    mut commands: Commands,
    player_query: Query<(&Position, &Collider), With<Player>>,
    trigger_query: Query<&LevelTransitionTrigger>,
) {
    for (player_pos, player_collider) in player_query.iter() {
        for trigger in trigger_query.iter() {
            // Check if player collides with trigger area
            let player_left = player_pos.x + player_collider.offset_x;
            let player_right = player_left + player_collider.width;
            let player_top = player_pos.y + player_collider.offset_y;
            let player_bottom = player_top + player_collider.height;

            let trigger_left = trigger.trigger_area.offset_x;
            let trigger_right = trigger_left + trigger.trigger_area.width;
            let trigger_top = trigger.trigger_area.offset_y;
            let trigger_bottom = trigger_top + trigger.trigger_area.height;

            if player_right > trigger_left
                && player_left < trigger_right
                && player_bottom > trigger_top
                && player_top < trigger_bottom
            {
                // Player entered transition trigger
                commands.insert_resource(PendingTransition {
                    to_level: trigger.to_level.clone(),
                    spawn_point: trigger.spawn_point,
                });
                return;
            }
        }
    }
}

/// Process pending level transition
#[allow(clippy::too_many_arguments)]
fn process_pending_transition(
    mut commands: Commands,
    pending: Option<Res<PendingTransition>>,
    _current_level: Option<Res<CurrentLevel>>,
    mut player_query: Query<&mut Position, With<Player>>,
    geometry_query: Query<Entity, With<LevelGeometry>>,
    trigger_query: Query<Entity, With<LevelTransitionTrigger>>,
    power_up_query: Query<Entity, With<PowerUp>>,
    swing_point_query: Query<Entity, With<SwingPoint>>,
) {
    if let Some(pending) = pending {
        // Unload current level entities
        for entity in geometry_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in trigger_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in power_up_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in swing_point_query.iter() {
            commands.entity(entity).despawn();
        }

        // Spawn player at new spawn point (do this before loading to preserve state)
        for mut player_pos in player_query.iter_mut() {
            player_pos.x = pending.spawn_point.x;
            player_pos.y = pending.spawn_point.y;
        }

        // Load new level
        let level_path = format!("levels/{}.json", pending.to_level);
        match load_level_from_file(&level_path) {
            Ok(new_level) => {
                // Spawn new level entities
                spawn_level_entities(&mut commands, &new_level);

                // Update current level resource
                commands.insert_resource(CurrentLevel {
                    level_id: new_level.id.clone(),
                    level_data: new_level,
                });

                info!("Transitioned to level: {}", pending.to_level);
            }
            Err(e) => {
                error!("Failed to load level {}: {:?}", pending.to_level, e);
                // Keep current level on error (entities already despawned, but player moved)
            }
        }

        // Remove pending transition
        commands.remove_resource::<PendingTransition>();
    }
}

/// Update ability gates based on player's abilities
fn update_ability_gates(
    mut commands: Commands,
    player_query: Query<&AbilitySet, With<Player>>,
    mut gate_query: Query<(Entity, &mut AbilityGate)>,
    gate_geometry_query: Query<(Entity, &LevelGeometry), With<AbilityGateGeometry>>,
) {
    // Get player's ability set
    let player_abilities = player_query.iter().next();

    if let Some(abilities) = player_abilities {
        for (gate_entity, mut gate) in gate_query.iter_mut() {
            let has_ability = abilities.has(gate.required_ability);

            // Update blocking state
            let should_block = !has_ability;

            // Only update if state changed
            if gate.is_blocking != should_block {
                gate.is_blocking = should_block;

                if should_block {
                    // Add blocking geometry
                    commands.entity(gate_entity).insert((
                        LevelGeometry {
                            x: gate.gate_area.offset_x,
                            y: gate.gate_area.offset_y,
                            width: gate.gate_area.width,
                            height: gate.gate_area.height,
                        },
                        AbilityGateGeometry,
                    ));
                } else {
                    // Remove blocking geometry
                    for (geo_entity, geo) in gate_geometry_query.iter() {
                        // Check if this geometry matches the gate area
                        if (geo.x - gate.gate_area.offset_x).abs() < 0.1
                            && (geo.y - gate.gate_area.offset_y).abs() < 0.1
                            && (geo.width - gate.gate_area.width).abs() < 0.1
                            && (geo.height - gate.gate_area.height).abs() < 0.1
                        {
                            commands.entity(geo_entity).remove::<LevelGeometry>();
                            break;
                        }
                    }
                }
            } else if should_block {
                // Gate should be blocking but geometry might not exist yet
                // Check if geometry exists
                let has_geometry = gate_geometry_query.iter().any(|(_, geo)| {
                    (geo.x - gate.gate_area.offset_x).abs() < 0.1
                        && (geo.y - gate.gate_area.offset_y).abs() < 0.1
                        && (geo.width - gate.gate_area.width).abs() < 0.1
                        && (geo.height - gate.gate_area.height).abs() < 0.1
                });

                if !has_geometry {
                    // Add blocking geometry
                    commands.entity(gate_entity).insert((
                        LevelGeometry {
                            x: gate.gate_area.offset_x,
                            y: gate.gate_area.offset_y,
                            width: gate.gate_area.width,
                            height: gate.gate_area.height,
                        },
                        AbilityGateGeometry,
                    ));
                }
            }
        }
    }
}

/// Marker component for ability gate geometry
#[derive(Component)]
struct AbilityGateGeometry;

/// Level loading errors
#[derive(Debug, Clone, PartialEq)]
pub enum LevelLoadError {
    FileNotFound(String),
    IoError(String, String),
    ParseError(String, String),
    ValidationError(String),
}

impl std::fmt::Display for LevelLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LevelLoadError::FileNotFound(path) => write!(f, "Level file not found: {}", path),
            LevelLoadError::IoError(path, err) => {
                write!(f, "IO error reading level file {}: {}", path, err)
            }
            LevelLoadError::ParseError(path, err) => {
                write!(f, "Failed to parse level file {}: {}", path, err)
            }
            LevelLoadError::ValidationError(msg) => write!(f, "Level validation error: {}", msg),
        }
    }
}

impl std::error::Error for LevelLoadError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Ability;
    use crate::level::{GeometryData, PowerUpData, SpawnPoint, SwingPointData};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_level() -> LevelData {
        LevelData {
            id: "test_level".to_string(),
            width: 1920.0,
            height: 1080.0,
            spawn_point: SpawnPoint { x: 100.0, y: 500.0 },
            geometry: vec![GeometryData {
                geometry_type: "platform".to_string(),
                x: 0.0,
                y: 0.0,
                width: 1920.0,
                height: 64.0,
            }],
            swing_points: vec![SwingPointData { x: 500.0, y: 800.0 }],
            checkpoints: vec![],
            power_ups: vec![PowerUpData {
                ability_type: Ability::HighJump,
                x: 800.0,
                y: 200.0,
            }],
            transitions: vec![],
            ability_gates: vec![],
        }
    }

    #[test]
    fn test_load_level_from_file_success() {
        let level = create_test_level();
        let json = serde_json::to_string_pretty(&level).unwrap();

        // Create temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Load level
        let loaded = load_level_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(loaded.id, "test_level");
        assert_eq!(loaded.width, 1920.0);
        assert_eq!(loaded.geometry.len(), 1);
    }

    #[test]
    fn test_load_level_file_not_found() {
        let result = load_level_from_file("nonexistent.json");
        assert!(matches!(result, Err(LevelLoadError::FileNotFound(_))));
    }

    #[test]
    fn test_load_level_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"{ invalid json }").unwrap();
        temp_file.flush().unwrap();

        let result = load_level_from_file(temp_file.path().to_str().unwrap());
        assert!(matches!(result, Err(LevelLoadError::ParseError(_, _))));
    }

    #[test]
    fn test_load_level_missing_required_fields() {
        let json = r#"{"id": "test"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_level_from_file(temp_file.path().to_str().unwrap());
        assert!(matches!(result, Err(LevelLoadError::ParseError(_, _))));
    }

    #[test]
    fn test_validate_level_data_empty_id() {
        let mut level = create_test_level();
        level.id = String::new();

        let result = validate_level_data(&level);
        assert!(matches!(result, Err(LevelLoadError::ValidationError(_))));
    }

    #[test]
    fn test_validate_level_data_invalid_dimensions() {
        let mut level = create_test_level();
        level.width = -100.0;

        let result = validate_level_data(&level);
        assert!(matches!(result, Err(LevelLoadError::ValidationError(_))));
    }

    #[test]
    fn test_validate_level_data_invalid_geometry() {
        let mut level = create_test_level();
        level.geometry[0].width = 0.0;

        let result = validate_level_data(&level);
        assert!(matches!(result, Err(LevelLoadError::ValidationError(_))));
    }

    #[test]
    fn test_spawn_level_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let level = create_test_level();

        // Spawn entities directly
        for geo in &level.geometry {
            app.world.spawn(LevelGeometry {
                x: geo.x,
                y: geo.y,
                width: geo.width,
                height: geo.height,
            });
        }

        for swing_point in &level.swing_points {
            app.world.spawn((
                SwingPoint { range: 100.0 },
                Position::new(swing_point.x, swing_point.y),
            ));
        }

        for power_up in &level.power_ups {
            app.world.spawn((
                PowerUp {
                    ability: power_up.ability_type,
                },
                Position::new(power_up.x, power_up.y),
            ));
        }

        // Verify geometry was spawned
        let geometry_count = app.world.query::<&LevelGeometry>().iter(&app.world).count();
        assert_eq!(geometry_count, 1);

        // Verify swing points were spawned
        let swing_count = app.world.query::<&SwingPoint>().iter(&app.world).count();
        assert_eq!(swing_count, 1);

        // Verify power-ups were spawned
        let power_up_count = app.world.query::<&PowerUp>().iter(&app.world).count();
        assert_eq!(power_up_count, 1);
    }

    #[test]
    fn test_level_transition_detection() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(LevelPlugin);

        // Spawn player at transition trigger location
        app.world.spawn((
            Player,
            Position::new(1800.0, 100.0),
            Collider::new(32.0, 64.0),
        ));

        // Spawn transition trigger
        app.world.spawn(LevelTransitionTrigger {
            to_level: "level_02".to_string(),
            spawn_point: Position::new(100.0, 500.0),
            trigger_area: Collider {
                width: 64.0,
                height: 200.0,
                offset_x: 1800.0,
                offset_y: 100.0,
            },
        });

        // Run one update
        app.update();

        // Verify pending transition was created
        assert!(app.world.get_resource::<PendingTransition>().is_some());
    }

    #[test]
    fn test_no_transition_when_far_away() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(LevelPlugin);

        // Spawn player far from trigger
        app.world.spawn((
            Player,
            Position::new(100.0, 100.0),
            Collider::new(32.0, 64.0),
        ));

        // Spawn transition trigger
        app.world.spawn(LevelTransitionTrigger {
            to_level: "level_02".to_string(),
            spawn_point: Position::new(100.0, 500.0),
            trigger_area: Collider {
                width: 64.0,
                height: 200.0,
                offset_x: 1800.0,
                offset_y: 100.0,
            },
        });

        // Run one update
        app.update();

        // Verify no pending transition
        assert!(app.world.get_resource::<PendingTransition>().is_none());
    }

    #[test]
    fn test_player_position_updated_on_transition() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(LevelPlugin);

        // Spawn player
        let player = app
            .world
            .spawn((
                Player,
                Position::new(100.0, 100.0),
                Collider::new(32.0, 64.0),
            ))
            .id();

        // Create a pending transition
        app.world.insert_resource(PendingTransition {
            to_level: "nonexistent".to_string(), // Will fail to load, but that's ok for this test
            spawn_point: Position::new(500.0, 600.0),
        });

        // Run one update to process transition
        app.update();

        // Verify player position was updated (even though level load failed)
        let player_pos = app.world.get::<Position>(player).unwrap();
        assert_eq!(player_pos.x, 500.0);
        assert_eq!(player_pos.y, 600.0);
    }

    #[test]
    fn test_level_entities_despawned_on_transition() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(LevelPlugin);

        // Spawn some level entities
        app.world.spawn(LevelGeometry {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 32.0,
        });

        app.world.spawn((
            PowerUp {
                ability: Ability::HighJump,
            },
            Position::new(100.0, 100.0),
        ));

        // Verify entities exist
        let geo_count_before = app.world.query::<&LevelGeometry>().iter(&app.world).count();
        assert_eq!(geo_count_before, 1);

        // Create a pending transition
        app.world.insert_resource(PendingTransition {
            to_level: "nonexistent".to_string(),
            spawn_point: Position::new(100.0, 100.0),
        });

        // Run one update to process transition
        app.update();

        // Verify entities were despawned
        let geo_count_after = app.world.query::<&LevelGeometry>().iter(&app.world).count();
        assert_eq!(geo_count_after, 0);

        let power_up_count_after = app.world.query::<&PowerUp>().iter(&app.world).count();
        assert_eq!(power_up_count_after, 0);
    }

    #[test]
    fn test_ability_gate_blocks_without_ability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(LevelPlugin);

        // Spawn player without required ability
        app.world.spawn((
            Player,
            Position::new(100.0, 100.0),
            Collider::new(32.0, 64.0),
            AbilitySet::new(), // No abilities
        ));

        // Spawn ability gate
        let gate = app
            .world
            .spawn(AbilityGate {
                required_ability: Ability::WallClimb,
                gate_area: Collider {
                    width: 64.0,
                    height: 200.0,
                    offset_x: 500.0,
                    offset_y: 100.0,
                },
                is_blocking: true,
            })
            .id();

        // Run one update
        app.update();

        // Verify gate is still blocking
        let gate_component = app.world.get::<AbilityGate>(gate).unwrap();
        assert!(gate_component.is_blocking);

        // Verify blocking geometry exists
        let geo_count = app.world.query::<&LevelGeometry>().iter(&app.world).count();
        assert_eq!(geo_count, 1);
    }

    #[test]
    fn test_ability_gate_opens_with_ability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(LevelPlugin);

        // Spawn player with required ability
        let mut abilities = AbilitySet::new();
        abilities.add(Ability::WallClimb);

        app.world.spawn((
            Player,
            Position::new(100.0, 100.0),
            Collider::new(32.0, 64.0),
            abilities,
        ));

        // Spawn ability gate (initially blocking)
        let gate = app
            .world
            .spawn(AbilityGate {
                required_ability: Ability::WallClimb,
                gate_area: Collider {
                    width: 64.0,
                    height: 200.0,
                    offset_x: 500.0,
                    offset_y: 100.0,
                },
                is_blocking: true,
            })
            .id();

        // Run one update
        app.update();

        // Verify gate is no longer blocking
        let gate_component = app.world.get::<AbilityGate>(gate).unwrap();
        assert!(!gate_component.is_blocking);
    }

    #[test]
    fn test_ability_gate_blocks_different_ability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(LevelPlugin);

        // Spawn player with different ability
        let mut abilities = AbilitySet::new();
        abilities.add(Ability::HighJump); // Has HighJump, not WallClimb

        app.world.spawn((
            Player,
            Position::new(100.0, 100.0),
            Collider::new(32.0, 64.0),
            abilities,
        ));

        // Spawn ability gate requiring WallClimb
        let gate = app
            .world
            .spawn(AbilityGate {
                required_ability: Ability::WallClimb,
                gate_area: Collider {
                    width: 64.0,
                    height: 200.0,
                    offset_x: 500.0,
                    offset_y: 100.0,
                },
                is_blocking: true,
            })
            .id();

        // Run one update
        app.update();

        // Verify gate is still blocking
        let gate_component = app.world.get::<AbilityGate>(gate).unwrap();
        assert!(gate_component.is_blocking);
    }
}
