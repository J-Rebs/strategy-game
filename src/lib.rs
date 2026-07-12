pub mod hex;
pub mod simulation;
pub mod rendering;
pub mod ai;
pub mod hud;

#[cfg(test)]
mod ui_tests;

use bevy::prelude::*;

pub use hex::{HexCoord, HexTile, HexTileType};
pub use simulation::{SimulationPlugin, NetworkNode, NetworkLink, RoutingTable, NodeType, LinkType, Owner, GameResources, Packet, PacketType, CityDominance, CitySize};
pub use rendering::RenderingPlugin;
pub use ai::AiPlugin;
pub use hud::{HudPlugin, PlayerControls, SelectedTool};

pub fn setup_initial_map(mut commands: Commands) {
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
    let p_dc_coord = HexCoord::new(-2, 0);
    let player_dc = commands.spawn((
        NetworkNode {
            ip: 10,
            coord: p_dc_coord,
            node_type: NodeType::DataCenter,
            owner: Owner::Player,
        },
        RoutingTable::default(),
        Transform::from_translation(p_dc_coord.to_world(1.0)),
    )).id();

    let p_router_coord = HexCoord::new(-1, 0);
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

    commands.spawn(NetworkLink {
        node_a: player_dc,
        node_b: player_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // 3. Spawn AI Subnet (East side)
    let ai_dc_coord = HexCoord::new(2, 0);
    let ai_dc = commands.spawn((
        NetworkNode {
            ip: 200,
            coord: ai_dc_coord,
            node_type: NodeType::DataCenter,
            owner: Owner::AI,
        },
        RoutingTable::default(),
        Transform::from_translation(ai_dc_coord.to_world(1.0)),
    )).id();

    let ai_router_coord = HexCoord::new(1, 0);
    let ai_router = commands.spawn((
        NetworkNode {
            ip: 210,
            coord: ai_router_coord,
            node_type: NodeType::Router,
            owner: Owner::AI,
        },
        RoutingTable::default(),
        Transform::from_translation(ai_router_coord.to_world(1.0)),
    )).id();

    commands.spawn(NetworkLink {
        node_a: ai_dc,
        node_b: ai_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // 4. Spawn Cities (Center line)
    // Small City at (0, -1) -> 10 BW/s
    let small_city_coord = HexCoord::new(0, -1);
    commands.spawn((
        NetworkNode {
            ip: 150,
            coord: small_city_coord,
            node_type: NodeType::City,
            owner: Owner::Neutral,
        },
        RoutingTable::default(),
        CityDominance {
            size: CitySize::Small,
            total_payout_rate: 10.0,
            ..default()
        },
        Transform::from_translation(small_city_coord.to_world(1.0)),
    ));

    // Medium City at (0, 0) -> 25 BW/s
    let med_city_coord = HexCoord::new(0, 0);
    commands.spawn((
        NetworkNode {
            ip: 151,
            coord: med_city_coord,
            node_type: NodeType::City,
            owner: Owner::Neutral,
        },
        RoutingTable::default(),
        CityDominance {
            size: CitySize::Medium,
            total_payout_rate: 25.0,
            ..default()
        },
        Transform::from_translation(med_city_coord.to_world(1.0)),
    ));

    // Large City at (0, 1) -> 50 BW/s
    let large_city_coord = HexCoord::new(0, 1);
    commands.spawn((
        NetworkNode {
            ip: 152,
            coord: large_city_coord,
            node_type: NodeType::City,
            owner: Owner::Neutral,
        },
        RoutingTable::default(),
        CityDominance {
            size: CitySize::Large,
            total_payout_rate: 50.0,
            ..default()
        },
        Transform::from_translation(large_city_coord.to_world(1.0)),
    ));
}
