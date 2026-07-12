# PacketCommand Coding & Educational Standards

This file contains rules for all AI agents working on the PacketCommand strategy game.

---

## 1. Educational Commenting Rules
Since this codebase is used for learning and pairing, all new and modified code MUST include beginner-friendly commentary detailing:
- **Rust Concepts**:
  - Explain ownership, borrowing (`&`), mutability (`&mut`), and dereferencing (`*`).
  - Explain pattern matching (`match`, `if let Some(x) = y`).
  - Explain Rust collections and basic syntax conventions.
- **Bevy ECS Concepts**:
  - Explain what **Entities**, **Components** (pure data structs), and **Resources** (global shared state) represent.
  - Explain **Systems** (functions that execute logic) and their query parameters (e.g., `Query<(&NetworkNode, &mut RoutingTable)>`, `Res<GameResources>`).
  - Keep comments highly readable, avoiding jargon, and framing them as interactive tutorial notes.

---

## 2. Modular Plug-and-Play Architecture
- The game is built using Bevy plugins (`SimulationPlugin`, `RenderingPlugin`, `AiPlugin`, `HudPlugin`).
- Keep these modules strictly separated:
  - **Simulation (`src/simulation.rs`)**: Owns the rules, routing algorithms, and resource balances. It does NOT depend on visuals, materials, or windows.
  - **Rendering (`src/rendering.rs`)**: Handles camera placement, lights, tile meshes, and color updates. It queries the simulation state to render it.
  - **AI (`src/ai.rs`)**: Independent agent loop. It queries the simulation state and mutates resources or node connections.
  - **HUD (`src/hud.rs`)**: User interface dashboard using Egui.
- Always write clean systems that can be easily commented out or replaced (plug-and-play).
