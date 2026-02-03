use bevy::prelude::*;
use sidescrolling_adventure_game::components::{AbilitySet, Player, Position};
use sidescrolling_adventure_game::enums::Ability;
use sidescrolling_adventure_game::plugins::checkpoint::{
    Checkpoint, CheckpointPlugin, LoadFromDisk, RestoreCheckpoint, SaveToDisk,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CheckpointPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, demo_checkpoint_system)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn player
    commands.spawn((
        Player,
        Position::new(100.0, 200.0),
        AbilitySet::from(vec![Ability::HighJump]),
    ));

    // Spawn checkpoint
    commands.spawn((
        Checkpoint::new("cp_demo".to_string()),
        Position::new(150.0, 200.0),
    ));

    info!("Checkpoint demo started!");
    info!("Player spawned at (100, 200) with HighJump ability");
    info!("Checkpoint spawned at (150, 200)");
}

fn demo_checkpoint_system(
    mut exit: EventWriter<bevy::app::AppExit>,
    keyboard: Res<Input<KeyCode>>,
    mut save_events: EventWriter<SaveToDisk>,
    mut load_events: EventWriter<LoadFromDisk>,
    mut restore_events: EventWriter<RestoreCheckpoint>,
    player_query: Query<(&Position, &AbilitySet), With<Player>>,
) {
    // Press S to manually trigger save
    if keyboard.just_pressed(KeyCode::S) {
        info!("Manual save triggered");
        save_events.send(SaveToDisk);
    }

    // Press L to load from disk
    if keyboard.just_pressed(KeyCode::L) {
        info!("Loading from disk");
        load_events.send(LoadFromDisk);
    }

    // Press R to restore from current checkpoint
    if keyboard.just_pressed(KeyCode::R) {
        info!("Restoring from checkpoint");
        restore_events.send(RestoreCheckpoint);
    }

    // Press P to print current player state
    if keyboard.just_pressed(KeyCode::P)
        && let Ok((pos, abilities)) = player_query.get_single()
    {
        info!("Player position: ({}, {})", pos.x, pos.y);
        info!("Player abilities: {:?}", abilities.abilities);
    }

    // Press ESC to quit
    if keyboard.just_pressed(KeyCode::Escape) {
        info!("Exiting demo");
        exit.send(bevy::app::AppExit);
    }
}
