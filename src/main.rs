mod simulation;
mod rendering;
mod ai;
mod hud;
mod hex;

#[cfg(test)]
mod ui_tests;
use bevy::prelude::*;
use std::collections::VecDeque;

use hex::{HexCoord, HexTile, HexTileType};
use simulation::{SimulationPlugin, NetworkNode, NetworkLink, RoutingTable, NodeType, LinkType, Owner};
use rendering::RenderingPlugin;
use ai::AiPlugin;
use hud::HudPlugin;

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

fn setup_initial_map(mut commands: Commands) {
    // 1. Generate 19 Hexagonal Tiles in a radius-3 board
    // Pointy-topped coordinates satisfying |q| <= 2, |r| <= 2, |q+r| <= 2
    for q in -2..=2 {
        for r in -2..=2 {
            let sum_qr: i32 = q + r;
            if sum_qr.abs() <= 2 {
                let coord = HexCoord::new(q, r);
                
                // Determine tile type to make a pretty layout (like Catan)
                let diff_qr: i32 = q - r;
                let tile_type = if q == 0 && r == 0 {
                    HexTileType::DataCenterCenter // center hub
                } else if q.abs() == 2 && r.abs() == 2 {
                    HexTileType::Water // water corners
                } else if diff_qr.abs() == 1 {
                    HexTileType::Mountain // mountain ridges
                } else {
                    HexTileType::Grass // normal grass
                };

                commands.spawn(HexTile { coord, tile_type });
            }
        }
    }

    // 2. Spawn Player Subnet (West side)
    let p_client_coord = HexCoord::new(-2, 0);
    let player_client = commands.spawn((
        NetworkNode {
            ip: 10,
            coord: p_client_coord,
            node_type: NodeType::Client,
            owner: Owner::Player,
            buffer: VecDeque::new(),
            max_buffer_size: 15,
            cpu_processing_rate: 3,
            firewall_rules: Vec::new(),
            health: 100.0,
        },
        RoutingTable::default(),
        Transform::from_translation(p_client_coord.to_world(1.0)),
    )).id();

    let p_router_coord = HexCoord::new(-1, 0);
    let player_router = commands.spawn((
        NetworkNode {
            ip: 20,
            coord: p_router_coord,
            node_type: NodeType::Router,
            owner: Owner::Player,
            buffer: VecDeque::new(),
            max_buffer_size: 20,
            cpu_processing_rate: 5,
            firewall_rules: Vec::new(),
            health: 100.0,
        },
        RoutingTable::default(),
        Transform::from_translation(p_router_coord.to_world(1.0)),
    )).id();

    commands.spawn(NetworkLink {
        node_a: player_client,
        node_b: player_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // 3. Spawn AI Subnet (East side)
    let ai_client_coord = HexCoord::new(2, 0);
    let ai_client = commands.spawn((
        NetworkNode {
            ip: 200,
            coord: ai_client_coord,
            node_type: NodeType::Client,
            owner: Owner::AI,
            buffer: VecDeque::new(),
            max_buffer_size: 15,
            cpu_processing_rate: 3,
            firewall_rules: Vec::new(),
            health: 100.0,
        },
        RoutingTable::default(),
        Transform::from_translation(ai_client_coord.to_world(1.0)),
    )).id();

    let ai_router_coord = HexCoord::new(1, 0);
    let ai_router = commands.spawn((
        NetworkNode {
            ip: 210,
            coord: ai_router_coord,
            node_type: NodeType::Router,
            owner: Owner::AI,
            buffer: VecDeque::new(),
            max_buffer_size: 20,
            cpu_processing_rate: 5,
            firewall_rules: Vec::new(),
            health: 100.0,
        },
        RoutingTable::default(),
        Transform::from_translation(ai_router_coord.to_world(1.0)),
    )).id();

    commands.spawn(NetworkLink {
        node_a: ai_client,
        node_b: ai_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // 4. Spawn Data Centers in the middle
    let dc1_coord = HexCoord::new(0, 0);
    commands.spawn((
        NetworkNode {
            ip: 100,
            coord: dc1_coord,
            node_type: NodeType::DataCenter,
            owner: Owner::Neutral,
            buffer: VecDeque::new(),
            max_buffer_size: 30,
            cpu_processing_rate: 6,
            firewall_rules: Vec::new(),
            health: 100.0,
        },
        RoutingTable::default(),
        Transform::from_translation(dc1_coord.to_world(1.0)),
    ));

    let dc2_coord = HexCoord::new(0, 1);
    commands.spawn((
        NetworkNode {
            ip: 101,
            coord: dc2_coord,
            node_type: NodeType::DataCenter,
            owner: Owner::Neutral,
            buffer: VecDeque::new(),
            max_buffer_size: 30,
            cpu_processing_rate: 6,
            firewall_rules: Vec::new(),
            health: 100.0,
        },
        RoutingTable::default(),
        Transform::from_translation(dc2_coord.to_world(1.0)),
    ));
}
