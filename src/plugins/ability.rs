use crate::components::{AbilitySet, Collider, Player, Position, PowerUp};
use bevy::prelude::*;

/// Plugin for ability unlocking and usage
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, collect_power_ups_system);
    }
}

/// System to detect and collect power-ups
fn collect_power_ups_system(
    mut commands: Commands,
    mut player_query: Query<(&Position, &Collider, &mut AbilitySet), With<Player>>,
    power_up_query: Query<(Entity, &Position, &PowerUp)>,
) {
    for (player_pos, player_collider, mut ability_set) in player_query.iter_mut() {
        for (power_up_entity, power_up_pos, power_up) in power_up_query.iter() {
            // Simple AABB collision detection for power-up collection
            let player_left = player_pos.x + player_collider.offset_x - player_collider.width / 2.0;
            let player_right =
                player_pos.x + player_collider.offset_x + player_collider.width / 2.0;
            let player_top = player_pos.y + player_collider.offset_y - player_collider.height / 2.0;
            let player_bottom =
                player_pos.y + player_collider.offset_y + player_collider.height / 2.0;

            // Assume power-ups have a fixed size of 32x32 for collection
            let power_up_size = 32.0;
            let power_up_left = power_up_pos.x - power_up_size / 2.0;
            let power_up_right = power_up_pos.x + power_up_size / 2.0;
            let power_up_top = power_up_pos.y - power_up_size / 2.0;
            let power_up_bottom = power_up_pos.y + power_up_size / 2.0;

            // Check for collision
            if player_right > power_up_left
                && player_left < power_up_right
                && player_bottom > power_up_top
                && player_top < power_up_bottom
            {
                // Add ability to player's set
                ability_set.add(power_up.ability);

                // Despawn the power-up entity
                commands.entity(power_up_entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::enums::Ability;

    #[test]
    fn test_ability_collection_adds_to_set() {
        // Create a test app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(AbilityPlugin);

        // Spawn player with empty ability set
        let player = app
            .world
            .spawn((
                Player,
                Position::new(100.0, 100.0),
                Collider::new(32.0, 64.0),
                AbilitySet::new(),
            ))
            .id();

        // Spawn power-up near player
        app.world.spawn((
            PowerUp {
                ability: Ability::HighJump,
            },
            Position::new(100.0, 100.0),
        ));

        // Run one update cycle
        app.update();

        // Verify ability was added
        let ability_set = app.world.get::<AbilitySet>(player).unwrap();
        assert!(ability_set.has(Ability::HighJump));
    }

    #[test]
    fn test_power_up_despawns_after_collection() {
        // Create a test app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(AbilityPlugin);

        // Spawn player
        app.world.spawn((
            Player,
            Position::new(100.0, 100.0),
            Collider::new(32.0, 64.0),
            AbilitySet::new(),
        ));

        // Spawn power-up near player
        let power_up = app
            .world
            .spawn((
                PowerUp {
                    ability: Ability::HighJump,
                },
                Position::new(100.0, 100.0),
            ))
            .id();

        // Run one update cycle
        app.update();

        // Verify power-up was despawned
        assert!(app.world.get_entity(power_up).is_none());
    }

    #[test]
    fn test_no_collection_when_far_away() {
        // Create a test app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(AbilityPlugin);

        // Spawn player
        let player = app
            .world
            .spawn((
                Player,
                Position::new(100.0, 100.0),
                Collider::new(32.0, 64.0),
                AbilitySet::new(),
            ))
            .id();

        // Spawn power-up far from player
        let power_up = app
            .world
            .spawn((
                PowerUp {
                    ability: Ability::HighJump,
                },
                Position::new(500.0, 500.0),
            ))
            .id();

        // Run one update cycle
        app.update();

        // Verify ability was NOT added
        let ability_set = app.world.get::<AbilitySet>(player).unwrap();
        assert!(!ability_set.has(Ability::HighJump));

        // Verify power-up still exists
        assert!(app.world.get_entity(power_up).is_some());
    }

    #[test]
    fn test_multiple_abilities_collection() {
        // Create a test app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(AbilityPlugin);

        // Spawn player
        let player = app
            .world
            .spawn((
                Player,
                Position::new(100.0, 100.0),
                Collider::new(32.0, 64.0),
                AbilitySet::new(),
            ))
            .id();

        // Spawn multiple power-ups near player
        app.world.spawn((
            PowerUp {
                ability: Ability::HighJump,
            },
            Position::new(100.0, 100.0),
        ));

        app.world.spawn((
            PowerUp {
                ability: Ability::WallClimb,
            },
            Position::new(105.0, 105.0),
        ));

        // Run one update cycle
        app.update();

        // Verify both abilities were added
        let ability_set = app.world.get::<AbilitySet>(player).unwrap();
        assert!(ability_set.has(Ability::HighJump));
        assert!(ability_set.has(Ability::WallClimb));
    }

    #[test]
    fn test_locked_ability_cannot_be_used() {
        // This test verifies that abilities not in the set are not present
        let mut ability_set = AbilitySet::new();

        // Verify locked abilities return false
        assert!(!ability_set.has(Ability::HighJump));
        assert!(!ability_set.has(Ability::WallClimb));
        assert!(!ability_set.has(Ability::Swing));

        // Add one ability
        ability_set.add(Ability::HighJump);

        // Verify only the added ability is present
        assert!(ability_set.has(Ability::HighJump));
        assert!(!ability_set.has(Ability::WallClimb));
        assert!(!ability_set.has(Ability::Swing));
    }
}
