use crate::components::{AbilitySet, Player, Position};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Checkpoint component - marks an entity as a checkpoint
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Checkpoint {
    pub id: String,
    pub activated: bool,
}

impl Checkpoint {
    pub fn new(id: String) -> Self {
        Self {
            id,
            activated: false,
        }
    }
}

/// Game state that can be saved and loaded
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub checkpoint_id: String,
    pub checkpoint_level: String,
    pub checkpoint_position: Position,
    pub unlocked_abilities: AbilitySet,
    pub timestamp: u64,
}

impl GameState {
    pub fn new(
        checkpoint_id: String,
        checkpoint_level: String,
        checkpoint_position: Position,
        unlocked_abilities: AbilitySet,
    ) -> Self {
        Self {
            checkpoint_id,
            checkpoint_level,
            checkpoint_position,
            unlocked_abilities,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Resource to store the current game state
#[derive(Resource, Clone, Debug, Default)]
pub struct CurrentGameState {
    pub state: Option<GameState>,
}

/// Resource to store the save file path
#[derive(Resource, Clone, Debug)]
pub struct SaveFilePath {
    pub path: PathBuf,
}

impl Default for SaveFilePath {
    fn default() -> Self {
        Self {
            path: PathBuf::from("save_data.json"),
        }
    }
}

/// Event triggered when a checkpoint is activated
#[derive(Event)]
pub struct CheckpointActivated {
    pub checkpoint_id: String,
}

/// Event triggered when requesting to restore from checkpoint
#[derive(Event)]
pub struct RestoreCheckpoint;

/// Event triggered when requesting to save to disk
#[derive(Event)]
pub struct SaveToDisk;

/// Event triggered when requesting to load from disk
#[derive(Event)]
pub struct LoadFromDisk;

/// Plugin for checkpoint and save system
pub struct CheckpointPlugin;

impl Plugin for CheckpointPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentGameState>()
            .init_resource::<SaveFilePath>()
            .add_event::<CheckpointActivated>()
            .add_event::<RestoreCheckpoint>()
            .add_event::<SaveToDisk>()
            .add_event::<LoadFromDisk>()
            .add_systems(
                Update,
                (
                    checkpoint_activation_system,
                    checkpoint_save_system,
                    checkpoint_restore_system,
                    save_to_disk_system,
                    load_from_disk_system,
                ),
            );
    }
}

/// System to detect checkpoint activation
fn checkpoint_activation_system(
    mut checkpoint_query: Query<(&mut Checkpoint, &Position)>,
    player_query: Query<&Position, With<Player>>,
    mut checkpoint_events: EventWriter<CheckpointActivated>,
) {
    let Ok(player_pos) = player_query.get_single() else {
        return;
    };

    for (mut checkpoint, checkpoint_pos) in checkpoint_query.iter_mut() {
        if checkpoint.activated {
            continue;
        }

        // Check if player is within activation range (64 pixels)
        let dx = player_pos.x - checkpoint_pos.x;
        let dy = player_pos.y - checkpoint_pos.y;
        let distance_squared = dx * dx + dy * dy;
        let activation_range = 64.0;

        if distance_squared <= activation_range * activation_range {
            checkpoint.activated = true;
            checkpoint_events.send(CheckpointActivated {
                checkpoint_id: checkpoint.id.clone(),
            });
        }
    }
}

/// System to capture game state when checkpoint is activated
fn checkpoint_save_system(
    mut checkpoint_events: EventReader<CheckpointActivated>,
    player_query: Query<(&Position, &AbilitySet), With<Player>>,
    mut current_state: ResMut<CurrentGameState>,
    mut save_events: EventWriter<SaveToDisk>,
) {
    for event in checkpoint_events.read() {
        let Ok((player_pos, abilities)) = player_query.get_single() else {
            continue;
        };

        // Create game state snapshot
        let game_state = GameState::new(
            event.checkpoint_id.clone(),
            "current_level".to_string(), // TODO: Get actual level ID from LevelPlugin
            *player_pos,
            abilities.clone(),
        );

        current_state.state = Some(game_state);

        // Trigger save to disk
        save_events.send(SaveToDisk);
    }
}

/// System to restore game state from checkpoint
fn checkpoint_restore_system(
    mut restore_events: EventReader<RestoreCheckpoint>,
    current_state: Res<CurrentGameState>,
    mut player_query: Query<(&mut Position, &mut AbilitySet), With<Player>>,
) {
    for _ in restore_events.read() {
        let Some(ref game_state) = current_state.state else {
            warn!("No checkpoint state to restore from");
            continue;
        };

        let Ok((mut player_pos, mut abilities)) = player_query.get_single_mut() else {
            warn!("Player not found for checkpoint restore");
            continue;
        };

        // Restore player position and abilities
        *player_pos = game_state.checkpoint_position;
        *abilities = game_state.unlocked_abilities.clone();

        info!("Restored from checkpoint: {}", game_state.checkpoint_id);
    }
}

/// System to save game state to disk
fn save_to_disk_system(
    mut save_events: EventReader<SaveToDisk>,
    current_state: Res<CurrentGameState>,
    save_path: Res<SaveFilePath>,
) {
    for _ in save_events.read() {
        let Some(ref game_state) = current_state.state else {
            warn!("No game state to save");
            continue;
        };

        // Serialize to JSON
        match serde_json::to_string_pretty(game_state) {
            Ok(json) => {
                // Write to disk
                match fs::write(&save_path.path, json) {
                    Ok(_) => {
                        info!("Game saved to {:?}", save_path.path);
                    }
                    Err(e) => {
                        error!("Failed to write save file: {}", e);
                        // Retry once
                        if let Err(retry_err) = fs::write(
                            &save_path.path,
                            serde_json::to_string_pretty(game_state).unwrap_or_default(),
                        ) {
                            error!("Retry failed: {}", retry_err);
                            warn!("Failed to save progress");
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to serialize game state: {}", e);
            }
        }
    }
}

/// System to load game state from disk
fn load_from_disk_system(
    mut load_events: EventReader<LoadFromDisk>,
    mut current_state: ResMut<CurrentGameState>,
    save_path: Res<SaveFilePath>,
    mut restore_events: EventWriter<RestoreCheckpoint>,
) {
    for _ in load_events.read() {
        // Check if save file exists
        if !save_path.path.exists() {
            info!("No save file found, starting new game");
            continue;
        }

        // Read from disk
        match fs::read_to_string(&save_path.path) {
            Ok(json) => {
                // Deserialize from JSON
                match serde_json::from_str::<GameState>(&json) {
                    Ok(game_state) => {
                        info!("Loaded save from {:?}", save_path.path);
                        current_state.state = Some(game_state);

                        // Trigger restore to apply loaded state
                        restore_events.send(RestoreCheckpoint);
                    }
                    Err(e) => {
                        error!("Failed to deserialize save file: {}", e);
                        warn!("Save file corrupted, starting new game");
                    }
                }
            }
            Err(e) => {
                error!("Failed to read save file: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_creation() {
        let checkpoint = Checkpoint::new("cp_01".to_string());
        assert_eq!(checkpoint.id, "cp_01");
        assert!(!checkpoint.activated);
    }

    #[test]
    fn test_game_state_creation() {
        let position = Position::new(100.0, 200.0);
        let abilities = AbilitySet::from(vec![crate::enums::Ability::HighJump]);

        let state = GameState::new(
            "cp_01".to_string(),
            "level_01".to_string(),
            position,
            abilities.clone(),
        );

        assert_eq!(state.checkpoint_id, "cp_01");
        assert_eq!(state.checkpoint_level, "level_01");
        assert_eq!(state.checkpoint_position, position);
        assert_eq!(state.unlocked_abilities, abilities);
        assert!(state.timestamp > 0);
    }

    #[test]
    fn test_game_state_serialization() {
        let position = Position::new(100.0, 200.0);
        let abilities = AbilitySet::from(vec![crate::enums::Ability::HighJump]);

        let state = GameState::new(
            "cp_01".to_string(),
            "level_01".to_string(),
            position,
            abilities,
        );

        // Serialize to JSON
        let json = serde_json::to_string(&state).unwrap();

        // Deserialize back
        let deserialized: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.checkpoint_id, deserialized.checkpoint_id);
        assert_eq!(state.checkpoint_level, deserialized.checkpoint_level);
        assert_eq!(state.checkpoint_position, deserialized.checkpoint_position);
        assert_eq!(state.unlocked_abilities, deserialized.unlocked_abilities);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_save_and_load_cycle() {
        // Create a temporary save file path
        let temp_path = PathBuf::from("test_save.json");

        // Clean up any existing test file
        let _ = fs::remove_file(&temp_path);

        // Create game state
        let position = Position::new(150.0, 250.0);
        let abilities = AbilitySet::from(vec![
            crate::enums::Ability::HighJump,
            crate::enums::Ability::WallClimb,
        ]);

        let original_state = GameState::new(
            "cp_test".to_string(),
            "level_test".to_string(),
            position,
            abilities,
        );

        // Serialize and save
        let json = serde_json::to_string_pretty(&original_state).unwrap();
        fs::write(&temp_path, json).unwrap();

        // Load and deserialize
        let loaded_json = fs::read_to_string(&temp_path).unwrap();
        let loaded_state: GameState = serde_json::from_str(&loaded_json).unwrap();

        // Verify
        assert_eq!(original_state.checkpoint_id, loaded_state.checkpoint_id);
        assert_eq!(
            original_state.checkpoint_level,
            loaded_state.checkpoint_level
        );
        assert_eq!(
            original_state.checkpoint_position,
            loaded_state.checkpoint_position
        );
        assert_eq!(
            original_state.unlocked_abilities,
            loaded_state.unlocked_abilities
        );

        // Clean up
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_corrupted_save_file() {
        let temp_path = PathBuf::from("test_corrupted.json");

        // Write invalid JSON
        fs::write(&temp_path, "{ invalid json }").unwrap();

        // Try to load
        let loaded_json = fs::read_to_string(&temp_path).unwrap();
        let result = serde_json::from_str::<GameState>(&loaded_json);

        // Should fail to deserialize
        assert!(result.is_err());

        // Clean up
        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_missing_save_file() {
        let temp_path = PathBuf::from("nonexistent_save.json");

        // Ensure file doesn't exist
        let _ = fs::remove_file(&temp_path);

        // Check if file exists
        assert!(!temp_path.exists());
    }
}
