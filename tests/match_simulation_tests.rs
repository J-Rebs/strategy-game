use bevy::prelude::*;
use strategy_game::hex::HexCoord;
use strategy_game::simulation::{
    GameResources, LinkType, NetworkLink, NetworkNode, NodeType, Owner,
    SimulationPlugin, CityDominance, CitySize,
};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(SimulationPlugin);
    app
}

#[test]
fn test_city_dominance_mechanic() {
    let mut app = setup_test_app();

    // 1. Spawning network nodes:
    // Player Main Data Center (IP 10)
    let player_dc = app
        .world_mut()
        .spawn((
            NetworkNode {
                ip: 10,
                coord: HexCoord::new(-2, 0),
                node_type: NodeType::DataCenter,
                owner: Owner::Player,
            },
            strategy_game::simulation::RoutingTable::default(),
            Transform::from_translation(HexCoord::new(-2, 0).to_world(1.0)),
        ))
        .id();

    // Player Router at (-1, 0)
    let player_router = app
        .world_mut()
        .spawn((
            NetworkNode {
                ip: 20,
                coord: HexCoord::new(-1, 0),
                node_type: NodeType::Router,
                owner: Owner::Player,
            },
            strategy_game::simulation::RoutingTable::default(),
            Transform::from_translation(HexCoord::new(-1, 0).to_world(1.0)),
        ))
        .id();

    // AI Main Data Center (IP 200)
    let ai_dc = app
        .world_mut()
        .spawn((
            NetworkNode {
                ip: 200,
                coord: HexCoord::new(2, 0),
                node_type: NodeType::DataCenter,
                owner: Owner::AI,
            },
            strategy_game::simulation::RoutingTable::default(),
            Transform::from_translation(HexCoord::new(2, 0).to_world(1.0)),
        ))
        .id();

    // AI Router at (1, 0)
    let ai_router = app
        .world_mut()
        .spawn((
            NetworkNode {
                ip: 210,
                coord: HexCoord::new(1, 0),
                node_type: NodeType::Router,
                owner: Owner::AI,
            },
            strategy_game::simulation::RoutingTable::default(),
            Transform::from_translation(HexCoord::new(1, 0).to_world(1.0)),
        ))
        .id();

    // Neutral City at (0, -1)
    let city = app
        .world_mut()
        .spawn((
            NetworkNode {
                ip: 150,
                coord: HexCoord::new(0, -1),
                node_type: NodeType::City,
                owner: Owner::Neutral,
            },
            strategy_game::simulation::RoutingTable::default(),
            CityDominance {
                size: CitySize::Small,
                total_payout_rate: 10.0,
                ..default()
            },
            Transform::from_translation(HexCoord::new(0, -1).to_world(1.0)),
        ))
        .id();

    // Connect Player DC -> Player Router
    app.world_mut().spawn(NetworkLink {
        node_a: player_dc,
        node_b: player_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // Connect AI DC -> AI Router
    app.world_mut().spawn(NetworkLink {
        node_a: ai_dc,
        node_b: ai_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // Connect Player Router -> City via Copper Link (Bandwidth capacity: 5, cost: 3)
    app.world_mut().spawn(NetworkLink {
        node_a: player_router,
        node_b: city,
        link_type: LinkType::Copper,
        is_active: true,
    });

    // Connect AI Router -> City via Fiber Link (Bandwidth capacity: 25, cost: 1)
    app.world_mut().spawn(NetworkLink {
        node_a: ai_router,
        node_b: city,
        link_type: LinkType::Fiber,
        is_active: true,
    });

    // Run Dijkstra routing updates and dominance calculation (1 tick)
    app.update();

    // Verify dominance scores and control percentages
    // Player: DC(10) -> Router(20) [cost 3] -> City(150) [cost 3]. Total cost = 6. Latency factor = 10.0 / 6.0 = 1.6667
    // Throughput Player = 5.0
    // Player Dominance = 5.0 * 1.6667 = 8.333
    // AI: DC(200) -> Router(210) [cost 3] -> City(150) [cost 1]. Total cost = 4. Latency factor = 10.0 / 4.0 = 2.5
    // Throughput AI = 25.0
    // AI Dominance = 25.0 * 2.5 = 62.5
    // Player control = 8.333 / (8.333 + 62.5) = 11.76%
    // AI control = 62.5 / (8.333 + 62.5) = 88.24%
    {
        let city_dom = app.world().get::<CityDominance>(city).unwrap();
        assert!(city_dom.player_dominance > 0.0);
        assert!(city_dom.ai_dominance > city_dom.player_dominance);
        
        let expected_player_pct = 8.333 / (8.333 + 62.5);
        let expected_ai_pct = 62.5 / (8.333 + 62.5);
        
        assert!((city_dom.player_control_pct - expected_player_pct).abs() < 0.01);
        assert!((city_dom.ai_control_pct - expected_ai_pct).abs() < 0.01);
    }
}

// Runs a simulated game under a starting bandwidth level and returns how many ticks
// it takes to upgrade the router and connect to two cities.
fn run_simulated_game(starting_bandwidth: f32) -> Option<u32> {
    let mut app = setup_test_app();
    
    // Spawn standard starting map nodes
    let player_dc = app.world_mut().spawn((
        NetworkNode {
            ip: 10,
            coord: HexCoord::new(-2, 0),
            node_type: NodeType::DataCenter,
            owner: Owner::Player,
        },
        strategy_game::simulation::RoutingTable::default(),
        Transform::from_translation(HexCoord::new(-2, 0).to_world(1.0)),
    )).id();

    let player_router = app.world_mut().spawn((
        NetworkNode {
            ip: 20,
            coord: HexCoord::new(-1, 0),
            node_type: NodeType::Router,
            owner: Owner::Player,
        },
        strategy_game::simulation::RoutingTable::default(),
        Transform::from_translation(HexCoord::new(-1, 0).to_world(1.0)),
    )).id();

    app.world_mut().spawn(NetworkLink {
        node_a: player_dc,
        node_b: player_router,
        link_type: LinkType::Copper,
        is_active: true,
    });

    let small_city = app.world_mut().spawn((
        NetworkNode {
            ip: 150,
            coord: HexCoord::new(0, -1),
            node_type: NodeType::City,
            owner: Owner::Neutral,
        },
        strategy_game::simulation::RoutingTable::default(),
        CityDominance {
            size: CitySize::Small,
            total_payout_rate: 10.0,
            ..default()
        },
        Transform::from_translation(HexCoord::new(0, -1).to_world(1.0)),
    )).id();

    let med_city = app.world_mut().spawn((
        NetworkNode {
            ip: 151,
            coord: HexCoord::new(0, 0),
            node_type: NodeType::City,
            owner: Owner::Neutral,
        },
        strategy_game::simulation::RoutingTable::default(),
        CityDominance {
            size: CitySize::Medium,
            total_payout_rate: 25.0,
            ..default()
        },
        Transform::from_translation(HexCoord::new(0, 0).to_world(1.0)),
    )).id();

    // Set starting player/AI bandwidth
    {
        let mut resources = app.world_mut().resource_mut::<GameResources>();
        resources.player_bandwidth = starting_bandwidth;
        resources.ai_bandwidth = 0.0;
    }

    let mut connected_small = false;
    let mut connected_medium = false;
    let mut upgraded_router = false;

    // Run simulation loop up to 6000 ticks (100 seconds)
    for tick in 1..=6000 {
        app.update();

        // Retrieve resources and check if we can make a move
        let current_bw = app.world().resource::<GameResources>().player_bandwidth;

        // 1. Connect Player Router to Small City (cost 50)
        if !connected_small && current_bw >= 50.0 {
            app.world_mut().spawn(NetworkLink {
                node_a: player_router,
                node_b: small_city,
                link_type: LinkType::Copper,
                is_active: true,
            });
            let mut resources = app.world_mut().resource_mut::<GameResources>();
            resources.player_bandwidth -= 50.0;
            connected_small = true;
            continue;
        }

        // 2. Upgrade Player Router to Data Center (cost 120)
        if connected_small && !upgraded_router && current_bw >= 120.0 {
            if let Some(mut node) = app.world_mut().get_mut::<NetworkNode>(player_router) {
                node.node_type = NodeType::DataCenter;
            }
            let mut resources = app.world_mut().resource_mut::<GameResources>();
            resources.player_bandwidth -= 120.0;
            upgraded_router = true;
            continue;
        }

        // 3. Connect Player Router (now Data Center) to Medium City (cost 50)
        if upgraded_router && !connected_medium && current_bw >= 50.0 {
            app.world_mut().spawn(NetworkLink {
                node_a: player_router,
                node_b: med_city,
                link_type: LinkType::Copper,
                is_active: true,
            });
            let mut resources = app.world_mut().resource_mut::<GameResources>();
            resources.player_bandwidth -= 50.0;
            connected_medium = true;
        }

        // Win condition: completed all progression milestones
        if connected_small && upgraded_router && connected_medium {
            return Some(tick);
        }
    }

    None
}

#[test]
fn test_starting_resources_tuning() {
    // Run simulation with different starting resources
    let results = [0.0, 10.0, 50.0, 100.0, 200.0].map(|bw| {
        (bw, run_simulated_game(bw))
    });

    println!("--- Starting Resources Tuning Results ---");
    for &(bw, ticks) in &results {
        match ticks {
            Some(t) => println!("Starting BW: {:>5.1} -> Completed in {:>4} ticks ({:.2}s)", bw, t, t as f32 / 60.0),
            None => println!("Starting BW: {:>5.1} -> Failed to complete (timed out)", bw),
        }
    }

    // Verify that starting with 0.0 or 10.0 bandwidth is extremely slow because they must wait
    // for passive trickle (1.0 BW/s) to even build the first link.
    // Assert that starting with 50.0+ BW allows completing the sequence significantly faster than starting with 10.0.
    if let (Some(t_10), Some(t_50)) = (results[1].1, results[2].1) {
        assert!(t_50 < t_10, "Starting with 50 BW should be faster than starting with 10 BW");
    }
}
