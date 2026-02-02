use crate::enums::Ability;
use serde::{Deserialize, Serialize};

/// Level data structure matching JSON format
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelData {
    pub id: String,
    pub width: f32,
    pub height: f32,
    pub spawn_point: SpawnPoint,
    pub geometry: Vec<GeometryData>,
    #[serde(default)]
    pub swing_points: Vec<SwingPointData>,
    #[serde(default)]
    pub checkpoints: Vec<CheckpointData>,
    #[serde(default)]
    pub power_ups: Vec<PowerUpData>,
    #[serde(default)]
    pub transitions: Vec<TransitionData>,
    #[serde(default)]
    pub ability_gates: Vec<AbilityGateData>,
}

/// Spawn point data
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpawnPoint {
    pub x: f32,
    pub y: f32,
}

/// Geometry data for level collision
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometryData {
    #[serde(rename = "type")]
    pub geometry_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Swing point data
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SwingPointData {
    pub x: f32,
    pub y: f32,
}

/// Checkpoint data
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckpointData {
    pub id: String,
    pub x: f32,
    pub y: f32,
}

/// Power-up data
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PowerUpData {
    #[serde(rename = "type")]
    pub ability_type: Ability,
    pub x: f32,
    pub y: f32,
}

/// Level transition data
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransitionData {
    pub to_level: String,
    pub trigger_area: TriggerArea,
    pub spawn_point: SpawnPoint,
}

/// Trigger area for level transitions
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TriggerArea {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Ability-gated area data
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AbilityGateData {
    pub required_ability: Ability,
    pub gate_area: TriggerArea,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_data_serialization() {
        let level = LevelData {
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
            checkpoints: vec![CheckpointData {
                id: "cp_01".to_string(),
                x: 300.0,
                y: 100.0,
            }],
            power_ups: vec![PowerUpData {
                ability_type: Ability::HighJump,
                x: 800.0,
                y: 200.0,
            }],
            transitions: vec![TransitionData {
                to_level: "level_02".to_string(),
                trigger_area: TriggerArea {
                    x: 1800.0,
                    y: 100.0,
                    width: 64.0,
                    height: 200.0,
                },
                spawn_point: SpawnPoint { x: 100.0, y: 500.0 },
            }],
            ability_gates: vec![AbilityGateData {
                required_ability: Ability::WallClimb,
                gate_area: TriggerArea {
                    x: 1000.0,
                    y: 100.0,
                    width: 64.0,
                    height: 200.0,
                },
            }],
        };

        // Serialize to JSON
        let json = serde_json::to_string(&level).unwrap();
        assert!(json.contains("test_level"));

        // Deserialize back
        let deserialized: LevelData = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }

    #[test]
    fn test_level_data_round_trip() {
        let level = LevelData {
            id: "level_01".to_string(),
            width: 1920.0,
            height: 1080.0,
            spawn_point: SpawnPoint { x: 100.0, y: 500.0 },
            geometry: vec![
                GeometryData {
                    geometry_type: "platform".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 1920.0,
                    height: 64.0,
                },
                GeometryData {
                    geometry_type: "wall".to_string(),
                    x: 500.0,
                    y: 64.0,
                    width: 32.0,
                    height: 200.0,
                },
            ],
            swing_points: vec![],
            checkpoints: vec![],
            power_ups: vec![],
            transitions: vec![],
            ability_gates: vec![],
        };

        // Round-trip through JSON
        let json = serde_json::to_string_pretty(&level).unwrap();
        let deserialized: LevelData = serde_json::from_str(&json).unwrap();

        assert_eq!(level.id, deserialized.id);
        assert_eq!(level.width, deserialized.width);
        assert_eq!(level.height, deserialized.height);
        assert_eq!(level.spawn_point, deserialized.spawn_point);
        assert_eq!(level.geometry.len(), deserialized.geometry.len());
        assert_eq!(level, deserialized);
    }

    #[test]
    fn test_minimal_level_data() {
        // Test with minimal required fields
        let json = r#"{
            "id": "minimal",
            "width": 800.0,
            "height": 600.0,
            "spawn_point": {"x": 50.0, "y": 50.0},
            "geometry": []
        }"#;

        let level: LevelData = serde_json::from_str(json).unwrap();
        assert_eq!(level.id, "minimal");
        assert_eq!(level.width, 800.0);
        assert_eq!(level.height, 600.0);
        assert_eq!(level.spawn_point.x, 50.0);
        assert_eq!(level.spawn_point.y, 50.0);
        assert!(level.geometry.is_empty());
        assert!(level.swing_points.is_empty());
        assert!(level.checkpoints.is_empty());
        assert!(level.power_ups.is_empty());
        assert!(level.transitions.is_empty());
    }

    #[test]
    fn test_geometry_type_field() {
        let json = r#"{
            "type": "platform",
            "x": 0.0,
            "y": 0.0,
            "width": 100.0,
            "height": 32.0
        }"#;

        let geometry: GeometryData = serde_json::from_str(json).unwrap();
        assert_eq!(geometry.geometry_type, "platform");
    }

    #[test]
    fn test_power_up_type_field() {
        let json = r#"{
            "type": "HighJump",
            "x": 100.0,
            "y": 200.0
        }"#;

        let power_up: PowerUpData = serde_json::from_str(json).unwrap();
        assert_eq!(power_up.ability_type, Ability::HighJump);
    }

    #[test]
    fn test_transition_data() {
        let json = r#"{
            "to_level": "level_02",
            "trigger_area": {
                "x": 1800.0,
                "y": 100.0,
                "width": 64.0,
                "height": 200.0
            },
            "spawn_point": {
                "x": 100.0,
                "y": 500.0
            }
        }"#;

        let transition: TransitionData = serde_json::from_str(json).unwrap();
        assert_eq!(transition.to_level, "level_02");
        assert_eq!(transition.trigger_area.x, 1800.0);
        assert_eq!(transition.spawn_point.x, 100.0);
    }

    #[test]
    fn test_ability_gate_data() {
        let json = r#"{
            "required_ability": "WallClimb",
            "gate_area": {
                "x": 1000.0,
                "y": 100.0,
                "width": 64.0,
                "height": 200.0
            }
        }"#;

        let gate: AbilityGateData = serde_json::from_str(json).unwrap();
        assert_eq!(gate.required_ability, Ability::WallClimb);
        assert_eq!(gate.gate_area.x, 1000.0);
    }
}
