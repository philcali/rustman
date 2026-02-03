use bevy::prelude::*;
use sidescrolling_adventure_game::plugins::{
    AbilityPlugin, CheckpointPlugin, LevelPlugin, PhysicsPlugin, PlayerPlugin, SwingPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(AbilityPlugin)
        .add_plugins(SwingPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(CheckpointPlugin)
        .run();
}
