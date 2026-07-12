use bevy::prelude::*;
use strategy_game::{setup_initial_map, SimulationPlugin, RenderingPlugin, AiPlugin, HudPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "PacketCommand - Undersea Ocean Reef".to_string(),
                resolution: (1200.0_f32, 800.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((SimulationPlugin, RenderingPlugin, AiPlugin, HudPlugin))
        .add_systems(Startup, setup_initial_map)
        .run();
}
