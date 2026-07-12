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
    // 1. Generate 37 Hexagonal Tiles in a radius-4 board
    for q in -3..=3 {
        for r in -3..=3 {
            let sum: i32 = q + r;
            if sum.abs() <= 3 {
                let coord = HexCoord::new(q, r);
                
                // Determine tile type to make a pretty layout
                let tile_type = if q == 0 && r == 0 {
                    HexTileType::DataCenterCenter // center hub
                } else if q.abs() == 3 || r.abs() == 3 {
                    HexTileType::Water // outer boundary
                } else {
                    HexTileType::Grass
                };

                commands.spawn(HexTile { coord, tile_type });
            }
        }
    }

    // 2. Spawn Player Subnet (West side: Teal)
    let p_dc_coord = HexCoord::new(-3, 0);
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

    commands.spawn(NetworkLink {
        node_a: player_dc,
        node_b: player_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // 3. Spawn AI1 Subnet (East side: Crimson Red)
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

    // 4. Spawn AI2 Subnet (North side: Lime Green)
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

    // 5. Spawn AI3 Subnet (South side: Orchid Purple)
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

    // 6. Spawn Cities (Center region)
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
                owner: Owner::Neutral,
            },
            RoutingTable::default(),
            CityDominance {
                size,
                total_payout_rate: payout,
                ..default()
            },
            Transform::from_translation(coord.to_world(1.0)),
        ));
    }
}
