use crate::{LevelData, Player, Position};
use bevy::prelude::*;

/// Camera follow speed constant - interpolation factor
const CAMERA_FOLLOW_SPEED: f32 = 3.0;

/// Target viewport dimensions in game units
/// This ensures consistent viewport size across different screen resolutions
const TARGET_VIEWPORT_WIDTH: f32 = 1280.0;
const TARGET_VIEWPORT_HEIGHT: f32 = 720.0;

/// Camera plugin - handles camera following and constraints
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(PostUpdate, (camera_follow_system, update_camera_projection));
    }
}

/// Camera target component - marks the camera entity
#[derive(Component)]
pub struct GameCamera;

/// Setup camera entity
fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), GameCamera));
}

/// Camera follow system - smoothly follows player with lag
/// Requirements: 9.1, 9.2, 9.3
fn camera_follow_system(
    time: Res<Time>,
    level_data: Option<Res<LevelData>>,
    player_query: Query<&Position, With<Player>>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
) {
    let Ok(player_pos) = player_query.get_single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    // Target position is the player's position
    let target_x = player_pos.x;
    let target_y = player_pos.y;

    // Smooth interpolation with lag
    let delta = time.delta_seconds();
    let lerp_factor = 1.0 - (-CAMERA_FOLLOW_SPEED * delta).exp();

    let mut new_x =
        camera_transform.translation.x + (target_x - camera_transform.translation.x) * lerp_factor;
    let mut new_y =
        camera_transform.translation.y + (target_y - camera_transform.translation.y) * lerp_factor;

    // Apply camera bounds constraint if level data is available
    if let Some(level) = level_data {
        // Use target viewport dimensions for consistent bounds calculation
        let viewport_width = TARGET_VIEWPORT_WIDTH;
        let viewport_height = TARGET_VIEWPORT_HEIGHT;

        // Calculate half viewport in world units
        let half_viewport_width = viewport_width / 2.0;
        let half_viewport_height = viewport_height / 2.0;

        // Constrain camera to level boundaries
        // Camera center should not show areas outside level bounds
        // Only apply constraints if level is larger than viewport
        if level.width > viewport_width {
            let min_x = half_viewport_width;
            let max_x = level.width - half_viewport_width;
            new_x = new_x.clamp(min_x, max_x);
        } else {
            // Center camera on small level
            new_x = level.width / 2.0;
        }

        if level.height > viewport_height {
            let min_y = half_viewport_height;
            let max_y = level.height - half_viewport_height;
            new_y = new_y.clamp(min_y, max_y);
        } else {
            // Center camera on small level
            new_y = level.height / 2.0;
        }
    }

    camera_transform.translation.x = new_x;
    camera_transform.translation.y = new_y;
}

/// Update camera projection to maintain consistent viewport scale
/// Requirements: 9.4
fn update_camera_projection(
    windows: Query<&Window>,
    mut camera_query: Query<&mut OrthographicProjection, With<GameCamera>>,
) {
    let Ok(mut projection) = camera_query.get_single_mut() else {
        return;
    };

    let window = windows.iter().next();
    let (window_width, window_height) = if let Some(win) = window {
        (win.width(), win.height())
    } else {
        // Use target dimensions if no window
        (TARGET_VIEWPORT_WIDTH, TARGET_VIEWPORT_HEIGHT)
    };

    // Calculate scale to maintain consistent viewport size
    // The scale determines how many game units fit in the viewport
    let scale_x = window_width / TARGET_VIEWPORT_WIDTH;
    let scale_y = window_height / TARGET_VIEWPORT_HEIGHT;

    // Use the smaller scale to ensure the entire game area is visible
    let scale = scale_x.min(scale_y);

    // Set the projection scale
    projection.scale = scale;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(CameraPlugin);
        // If this compiles and runs, the plugin is valid
    }

    #[test]
    fn test_camera_follow_interpolation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CameraPlugin);

        // Spawn player at a specific position
        app.world.spawn((Player, Position::new(500.0, 300.0)));

        // Run one update to setup camera
        app.update();

        // Get camera position
        let mut camera_query = app.world.query_filtered::<&Transform, With<GameCamera>>();
        let camera_transform = camera_query.iter(&app.world).next().unwrap();
        let initial_x = camera_transform.translation.x;
        let initial_y = camera_transform.translation.y;

        // Run several updates
        for _ in 0..10 {
            app.update();
        }

        // Camera should have moved toward player
        let mut camera_query = app.world.query_filtered::<&Transform, With<GameCamera>>();
        let camera_transform = camera_query.iter(&app.world).next().unwrap();

        // Camera should be closer to player position (500, 300) than initial position (0, 0)
        let distance_to_player = ((camera_transform.translation.x - 500.0).powi(2)
            + (camera_transform.translation.y - 300.0).powi(2))
        .sqrt();
        let initial_distance = ((initial_x - 500.0).powi(2) + (initial_y - 300.0).powi(2)).sqrt();

        assert!(
            distance_to_player < initial_distance,
            "Camera should move closer to player over time"
        );
    }

    #[test]
    fn test_camera_smooth_following() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CameraPlugin);

        // Spawn player
        app.world.spawn((Player, Position::new(100.0, 100.0)));

        // Run updates
        app.update();
        app.update();

        // Get initial camera position
        let mut camera_query = app.world.query_filtered::<&Transform, With<GameCamera>>();
        let camera_transform = camera_query.iter(&app.world).next().unwrap();
        let pos1 = camera_transform.translation;

        // Run one more update
        app.update();

        // Get new camera position
        let mut camera_query = app.world.query_filtered::<&Transform, With<GameCamera>>();
        let camera_transform = camera_query.iter(&app.world).next().unwrap();
        let pos2 = camera_transform.translation;

        // Camera should not snap instantly - should show gradual movement
        // (unless already at target, but with player at 100,100 and camera starting at 0,0, it won't be)
        let movement = ((pos2.x - pos1.x).powi(2) + (pos2.y - pos1.y).powi(2)).sqrt();
        assert!(movement > 0.0, "Camera should continue moving smoothly");
    }

    #[test]
    fn test_camera_bounds_constraint() {
        use crate::level::*;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CameraPlugin);

        // Create a large level (larger than default viewport 1280x720)
        let level = LevelData {
            id: "test_level".to_string(),
            width: 2000.0,
            height: 1500.0,
            spawn_point: SpawnPoint {
                x: 1000.0,
                y: 750.0,
            },
            geometry: vec![],
            swing_points: vec![],
            checkpoints: vec![],
            power_ups: vec![],
            transitions: vec![],
            ability_gates: vec![],
        };
        app.insert_resource(level);

        // Spawn player at far right edge (beyond level bounds)
        app.world.spawn((Player, Position::new(2500.0, 750.0)));

        // Run several updates to let camera catch up
        for _ in 0..20 {
            app.update();
        }

        // Camera should be constrained to level bounds
        let mut camera_query = app.world.query_filtered::<&Transform, With<GameCamera>>();
        let camera_transform = camera_query.iter(&app.world).next().unwrap();

        // Camera X should not exceed level width minus half viewport
        // With default viewport of 1280x720, half width is 640
        // Max camera X should be 2000 - 640 = 1360
        assert!(
            camera_transform.translation.x <= 2000.0 - 640.0 + 1.0,
            "Camera X should be constrained to level bounds, got {}",
            camera_transform.translation.x
        );
    }

    #[test]
    fn test_camera_bounds_constraint_left_edge() {
        use crate::level::*;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CameraPlugin);

        // Create a large level
        let level = LevelData {
            id: "test_level".to_string(),
            width: 2000.0,
            height: 1500.0,
            spawn_point: SpawnPoint {
                x: 1000.0,
                y: 750.0,
            },
            geometry: vec![],
            swing_points: vec![],
            checkpoints: vec![],
            power_ups: vec![],
            transitions: vec![],
            ability_gates: vec![],
        };
        app.insert_resource(level);

        // Spawn player at far left edge (negative position)
        app.world.spawn((Player, Position::new(-500.0, 750.0)));

        // Run several updates
        for _ in 0..20 {
            app.update();
        }

        // Camera should be constrained to level bounds
        let mut camera_query = app.world.query_filtered::<&Transform, With<GameCamera>>();
        let camera_transform = camera_query.iter(&app.world).next().unwrap();

        // Camera X should not be less than half viewport width (640)
        assert!(
            camera_transform.translation.x >= 640.0 - 1.0,
            "Camera X should be constrained to level left bound, got {}",
            camera_transform.translation.x
        );
    }

    #[test]
    fn test_camera_without_level_data() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CameraPlugin);

        // Spawn player without level data
        app.world.spawn((Player, Position::new(2000.0, 1000.0)));

        // Run updates - should not crash
        for _ in 0..10 {
            app.update();
        }

        // Camera should follow player without bounds constraint
        let mut camera_query = app.world.query_filtered::<&Transform, With<GameCamera>>();
        let camera_transform = camera_query.iter(&app.world).next().unwrap();

        // Camera should be moving toward player position
        assert!(
            camera_transform.translation.x > 0.0,
            "Camera should follow player even without level data"
        );
    }

    #[test]
    fn test_camera_small_level_centering() {
        use crate::level::*;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CameraPlugin);

        // Create a small level (smaller than viewport)
        let level = LevelData {
            id: "small_level".to_string(),
            width: 800.0,
            height: 600.0,
            spawn_point: SpawnPoint { x: 400.0, y: 300.0 },
            geometry: vec![],
            swing_points: vec![],
            checkpoints: vec![],
            power_ups: vec![],
            transitions: vec![],
            ability_gates: vec![],
        };
        app.insert_resource(level);

        // Spawn player
        app.world.spawn((Player, Position::new(400.0, 300.0)));

        // Run updates
        for _ in 0..20 {
            app.update();
        }

        // Camera should be centered on the small level
        let mut camera_query = app.world.query_filtered::<&Transform, With<GameCamera>>();
        let camera_transform = camera_query.iter(&app.world).next().unwrap();

        // Camera should be at center of level (400, 300)
        assert!(
            (camera_transform.translation.x - 400.0).abs() < 1.0,
            "Camera X should be centered on small level, got {}",
            camera_transform.translation.x
        );
        assert!(
            (camera_transform.translation.y - 300.0).abs() < 1.0,
            "Camera Y should be centered on small level, got {}",
            camera_transform.translation.y
        );
    }

    #[test]
    fn test_viewport_scaling_consistency() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CameraPlugin);

        // Spawn player
        app.world.spawn((Player, Position::new(500.0, 300.0)));

        // Run updates
        app.update();

        // Get camera projection
        let mut camera_query = app
            .world
            .query_filtered::<&OrthographicProjection, With<GameCamera>>();
        let projection = camera_query.iter(&app.world).next().unwrap();

        // Projection scale should be set (default is 1.0 for matching window size)
        assert!(
            projection.scale > 0.0,
            "Projection scale should be positive"
        );

        // With no window, scale should be 1.0 (target dimensions match default)
        assert!(
            (projection.scale - 1.0).abs() < 0.01,
            "Projection scale should be 1.0 with default dimensions, got {}",
            projection.scale
        );
    }

    #[test]
    fn test_viewport_maintains_aspect_ratio() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(CameraPlugin);

        // Spawn player
        app.world.spawn((Player, Position::new(500.0, 300.0)));

        // Run updates
        app.update();

        // Get camera projection
        let mut camera_query = app
            .world
            .query_filtered::<&OrthographicProjection, With<GameCamera>>();
        let projection = camera_query.iter(&app.world).next().unwrap();

        // Verify projection exists and has valid scale
        assert!(
            projection.scale.is_finite(),
            "Projection scale should be finite"
        );
        assert!(
            projection.scale > 0.0,
            "Projection scale should be positive"
        );
    }
}
