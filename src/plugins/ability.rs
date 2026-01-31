use bevy::prelude::*;

/// Plugin for ability unlocking and usage
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        // Systems will be added in future tasks
        app.add_systems(Update, placeholder_system);
    }
}

fn placeholder_system() {
    // Placeholder system to be replaced in future tasks
}
