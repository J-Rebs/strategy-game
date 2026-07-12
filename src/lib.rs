// =========================================================================
// PACKETCOMMAND LIBRARY ROOT & BOARD INITIALIZATION
// =========================================================================
// This file registers the modules of the game and sets up the starting map layout.
//
// MODULE STRUCTURE:
//   - hex: The mathematical grid systems for pointy-topped 3D hexagons.
//   - simulation: The backend state machines, network loops, and routing algorithms.
//   - rendering: Visual material setup, cameras, light rays, and mesh updates.
//   - ai: The background AI agent decision cycles.
//   - hud: Egui dashboard panels, stats bars, and user interaction tools.
pub mod hex;
pub mod simulation;
pub mod rendering;
pub mod ai;
pub mod hud;

#[cfg(test)]
mod ui_tests; // Unit and visual mock testing module

use bevy::prelude::*;

// Re-export common symbols so other parts of the application can import them cleanly
pub use hex::{HexCoord, HexTile, HexTileType};
pub use simulation::{
    SimulationPlugin, NetworkNode, NetworkLink, RoutingTable, NodeType, LinkType, 
    Owner, GameResources, Packet, PacketType, CityDominance, CitySize
};
pub use rendering::RenderingPlugin;
pub use ai::AiPlugin;
pub use hud::{HudPlugin, PlayerControls, SelectedTool};

/// Startup System: Creates the initial 3D board.
///
/// In Bevy ECS, functions designated as systems take arguments representing queries, resources,
/// or commands. Here, `commands: Commands` is Bevy's buffer to spawn, modify, or kill Entities.
pub fn setup_initial_map(mut commands: Commands) {
    // -------------------------------------------------------------------------
    // 1. GENERATE THE HEXAGONAL PLAYING FIELD
    // -------------------------------------------------------------------------
    // Pointy-topped hex boards use axial coordinate loops (q, r).
    // The equation (q + r).abs() <= 3 limits the boundaries to a radius-4 hexagon (37 tiles total).
    for q in -3..=3 {
        for r in -3..=3 {
            let sum: i32 = q + r;
            if sum.abs() <= 3 {
                let coord = HexCoord::new(q, r);
                
                // Establish a pretty visual terrain layout:
                // - Center tile is a Data Center sand hub.
                // - Outer boundary rings are water (representing natural isolators).
                // - The rest are pleasant sage-green hills.
                let tile_type = if q == 0 && r == 0 {
                    HexTileType::DataCenterCenter
                } else if q.abs() == 3 || r.abs() == 3 {
                    HexTileType::Water
                } else {
                    HexTileType::Grass
                };

                // Spawn a HexTile entity. Bevy's ECS accepts any struct that derives `Component`.
                commands.spawn(HexTile { coord, tile_type });
            }
        }
    }

    // -------------------------------------------------------------------------
    // 2. SPAWN PLAYER SUBNET (Cornflower Blue)
    // -------------------------------------------------------------------------
    // To bundle multiple components onto a single Entity, Bevy uses tuple parameters
    // e.g. `commands.spawn((ComponentA, ComponentB, ComponentC))`.
    // `.id()` captures the Entity's unique identifier, which is needed to wire links.
    let p_dc_coord = HexCoord::new(-3, 0);
    let player_dc = commands.spawn((
        NetworkNode {
            ip: 10,
            coord: p_dc_coord,
            node_type: NodeType::DataCenter,
            owner: Owner::Player,
        },
        RoutingTable::default(), // Keeps track of OSPF paths calculated via Dijkstra
        Transform::from_translation(p_dc_coord.to_world(1.0)), // Set physical coordinates
    )).id();

    let p_router_coord = HexCoord::new(-2, 0);
    let player_router = commands.spawn((
        NetworkNode {
            ip: 20,
            coord: p_router_coord,
            node_type: NodeType::Router,
            owner: Owner::Player,
        },
        RoutingTable::default(),
        Transform::from_translation(p_router_coord.to_world(1.0)),
    )).id();

    // Spawn a wire connection (NetworkLink) connecting the Player's Main DC to their router
    commands.spawn(NetworkLink {
        node_a: player_dc,
        node_b: player_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // -------------------------------------------------------------------------
    // 3. SPAWN AI1 SUBNET (Terracotta clay red)
    // -------------------------------------------------------------------------
    let ai1_dc_coord = HexCoord::new(3, -3);
    let ai1_dc = commands.spawn((
        NetworkNode {
            ip: 100,
            coord: ai1_dc_coord,
            node_type: NodeType::DataCenter,
            owner: Owner::AI1,
        },
        RoutingTable::default(),
        Transform::from_translation(ai1_dc_coord.to_world(1.0)),
    )).id();

    let ai1_router_coord = HexCoord::new(2, -2);
    let ai1_router = commands.spawn((
        NetworkNode {
            ip: 110,
            coord: ai1_router_coord,
            node_type: NodeType::Router,
            owner: Owner::AI1,
        },
        RoutingTable::default(),
        Transform::from_translation(ai1_router_coord.to_world(1.0)),
    )).id();

    commands.spawn(NetworkLink {
        node_a: ai1_dc,
        node_b: ai1_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // -------------------------------------------------------------------------
    // 4. SPAWN AI2 SUBNET (Buttercup Yellow)
    // -------------------------------------------------------------------------
    let ai2_dc_coord = HexCoord::new(0, 3);
    let ai2_dc = commands.spawn((
        NetworkNode {
            ip: 200,
            coord: ai2_dc_coord,
            node_type: NodeType::DataCenter,
            owner: Owner::AI2,
        },
        RoutingTable::default(),
        Transform::from_translation(ai2_dc_coord.to_world(1.0)),
    )).id();

    let ai2_router_coord = HexCoord::new(0, 2);
    let ai2_router = commands.spawn((
        NetworkNode {
            ip: 210,
            coord: ai2_router_coord,
            node_type: NodeType::Router,
            owner: Owner::AI2,
        },
        RoutingTable::default(),
        Transform::from_translation(ai2_router_coord.to_world(1.0)),
    )).id();

    commands.spawn(NetworkLink {
        node_a: ai2_dc,
        node_b: ai2_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // -------------------------------------------------------------------------
    // 5. SPAWN AI3 SUBNET (Lilac Purple)
    // -------------------------------------------------------------------------
    let ai3_dc_coord = HexCoord::new(0, -3);
    let ai3_dc = commands.spawn((
        NetworkNode {
            ip: 300,
            coord: ai3_dc_coord,
            node_type: NodeType::DataCenter,
            owner: Owner::AI3,
        },
        RoutingTable::default(),
        Transform::from_translation(ai3_dc_coord.to_world(1.0)),
    )).id();

    let ai3_router_coord = HexCoord::new(0, -2);
    let ai3_router = commands.spawn((
        NetworkNode {
            ip: 310,
            coord: ai3_router_coord,
            node_type: NodeType::Router,
            owner: Owner::AI3,
        },
        RoutingTable::default(),
        Transform::from_translation(ai3_router_coord.to_world(1.0)),
    )).id();

    commands.spawn(NetworkLink {
        node_a: ai3_dc,
        node_b: ai3_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // -------------------------------------------------------------------------
    // 6. SPAWN NEUTRAL CITIES (Center fighting territory)
    // -------------------------------------------------------------------------
    // Cities are target destinations where players capture territory and extract passive income.
    let city_coords = [
        (HexCoord::new(0, 0), CitySize::Medium, 150, 25.0),
        (HexCoord::new(-1, 1), CitySize::Small, 151, 10.0),
        (HexCoord::new(1, -1), CitySize::Small, 152, 10.0),
        (HexCoord::new(0, -1), CitySize::Large, 153, 50.0),
    ];

    for (coord, size, ip, payout) in city_coords {
        commands.spawn((
            NetworkNode {
                ip,
                coord,
                node_type: NodeType::City,
                owner: Owner::Neutral, // Starts neutral
            },
            RoutingTable::default(),
            CityDominance {
                size,
                total_payout_rate: payout, // Raw bandwidth generation capacity of this city
                ..default()
            },
            Transform::from_translation(coord.to_world(1.0)),
        ));
    }
}
