use bevy::prelude::*;
use strategy_game::{setup_initial_map, SimulationPlugin, RenderingPlugin, AiPlugin, HudPlugin};

// =========================================================================
// ENTRY POINT & PLUG-AND-PLAY ARCHITECTURE
// =========================================================================
// This is the main entry point for the Bevy game engine application.
// Bevy is an ECS (Entity Component System) framework. Instead of structuring
// code with traditional object inheritance (OOP), Bevy uses:
//   - Entities: Unique IDs (like "game objects").
//   - Components: Small, modular data structs attached to Entities.
//   - Systems: Standalone functions that query and execute logic on Components.
//   - Resources: Global, shared singletons (e.g., resource balances).
//
// The game is structured into modular Bevy Plugins. You can comment any
// plugin out in the list below to "plug-and-play" or disable features:
//   - Comment out `AiPlugin` to play sandbox mode with idle opponents.
//   - Comment out `HudPlugin` to hide the egui user interface overlay.
//   - Comment out `RenderingPlugin` to run the game as a fast headless simulation.
fn main() {
    App::new()
        // 1. DefaultPlugins: Standard engine features (windowing, input, asset loading, etc.)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "PacketCommand - Cyber Grid Network Dominance".to_string(),
                resolution: (1200.0_f32, 800.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        // 2. Custom Plugins: Modularity at its finest. Each plugin wraps its own Bevy systems
        //    and runs them concurrently or in specific execution phases.
        .add_plugins((
            SimulationPlugin, // Core rules, packet spawning, routing calculations
            RenderingPlugin,  // 3D camera, lighting, tile spawning, tower meshes, colors
            AiPlugin,         // Smart AI decision-making loop
            HudPlugin,         // egui developer hud panels & user inputs
        ))
        // 3. Startup Systems: Runs exactly once when the game starts up.
        //    Here we spawn the pointy-topped hex tiles and Main Data Center nodes.
        .add_systems(Startup, setup_initial_map)
        // 4. Run: Starts the main game loop (usually targetting 60 frames per second).
        .run();
}
