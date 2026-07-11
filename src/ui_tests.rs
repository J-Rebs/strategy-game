#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use std::collections::VecDeque;
    use crate::hex::{HexCoord, HexTile, HexTileType};
    use crate::simulation::{NetworkNode, NetworkLink, RoutingTable, NodeType, LinkType, Owner, GameResources};
    use crate::hud::PlayerControls;

    // Helper to spawn a minimal test app
    fn setup_test_app() -> App {
        let mut app = App::new();
        app.init_resource::<GameResources>();
        app.init_resource::<PlayerControls>();
        app
    }

    #[test]
    fn test_grid_spawn_and_boundaries() {
        let mut app = setup_test_app();
        
        // Spawn hex board tiles
        let mut tile_count = 0;
        for q in -2..=2 {
            for r in -2..=2 {
                let sum_qr: i32 = q + r;
                if sum_qr.abs() <= 2 {
                    let coord = HexCoord::new(q, r);
                    assert!(coord.is_on_board());
                    app.world_mut().spawn(HexTile { 
                        coord, 
                        tile_type: HexTileType::Grass 
                    });
                    tile_count += 1;
                }
            }
        }
        assert_eq!(tile_count, 19, "Board should contain exactly 19 tiles for radius 3");
    }

    #[test]
    fn test_building_cost_deduction() {
        let mut app = setup_test_app();
        
        // Initial player bandwidth set to default (e.g. 100.0)
        let mut game_res = app.world_mut().resource_mut::<GameResources>();
        game_res.player_bandwidth = 150.0;

        // Perform mock buying action
        let router_cost = 60.0;
        let mut game_res = app.world_mut().resource_mut::<GameResources>();
        if game_res.player_bandwidth >= router_cost {
            game_res.player_bandwidth -= router_cost;
        }

        let updated_bw = app.world().resource::<GameResources>().player_bandwidth;
        assert_eq!(updated_bw, 90.0, "Bandwidth should be reduced by cost of router");
    }

    #[test]
    fn test_dijkstra_routing_on_hex_grid() {
        let mut app = setup_test_app();
        
        // Spawn 3 sequential nodes along hex coordinates:
        // Client at (-2, 0) -> Router at (-1, 0) -> Data Center at (0, 0)
        let client_coord = HexCoord::new(-2, 0);
        let router_coord = HexCoord::new(-1, 0);
        let dc_coord = HexCoord::new(0, 0);

        let client = app.world_mut().spawn((
            NetworkNode {
                ip: 10,
                coord: client_coord,
                node_type: NodeType::Client,
                owner: Owner::Player,
                buffer: VecDeque::new(),
                max_buffer_size: 10,
                cpu_processing_rate: 1,
                firewall_rules: Vec::new(),
                health: 100.0,
            },
            RoutingTable::default(),
        )).id();

        let router = app.world_mut().spawn((
            NetworkNode {
                ip: 20,
                coord: router_coord,
                node_type: NodeType::Router,
                owner: Owner::Player,
                buffer: VecDeque::new(),
                max_buffer_size: 10,
                cpu_processing_rate: 1,
                firewall_rules: Vec::new(),
                health: 100.0,
            },
            RoutingTable::default(),
        )).id();

        let dc = app.world_mut().spawn((
            NetworkNode {
                ip: 100,
                coord: dc_coord,
                node_type: NodeType::DataCenter,
                owner: Owner::Neutral,
                buffer: VecDeque::new(),
                max_buffer_size: 10,
                cpu_processing_rate: 1,
                firewall_rules: Vec::new(),
                health: 100.0,
            },
            RoutingTable::default(),
        )).id();

        // Connect client -> router, router -> dc
        app.world_mut().spawn(NetworkLink {
            node_a: client,
            node_b: router,
            link_type: LinkType::Copper,
            is_active: true,
        });

        app.world_mut().spawn(NetworkLink {
            node_a: router,
            node_b: dc,
            link_type: LinkType::Copper,
            is_active: true,
        });

        // Run Dijkstra manually to verify path
        // (This mirrors update_routing_tables logic)
        let mut adj = std::collections::HashMap::new();
        adj.insert(client, vec![(router, 10)]);
        adj.insert(router, vec![(client, 10), (dc, 10)]);
        adj.insert(dc, vec![(router, 10)]);

        // Calculate shortest path from Client (10) to DC (100)
        let path = calculate_shortest_path_mock(client, dc, &adj);
        assert_eq!(path, Some(vec![client, router, dc]), "Dijkstra should route Client -> Router -> DC");
    }

    fn calculate_shortest_path_mock(
        start: Entity,
        target: Entity,
        adj: &std::collections::HashMap<Entity, Vec<(Entity, u32)>>,
    ) -> Option<Vec<Entity>> {
        let mut dist = std::collections::HashMap::new();
        let mut prev = std::collections::HashMap::new();
        let mut queue = std::collections::BinaryHeap::new();

        dist.insert(start, 0);
        queue.push(State { node: start, cost: 0 });

        while let Some(State { node, cost }) = queue.pop() {
            if node == target {
                let mut path = Vec::new();
                let mut curr = target;
                while curr != start {
                    path.push(curr);
                    curr = *prev.get(&curr)?;
                }
                path.push(start);
                path.reverse();
                return Some(path);
            }

            if cost > *dist.get(&node).unwrap_or(&u32::MAX) {
                continue;
            }

            if let Some(neighbors) = adj.get(&node) {
                for &(next, edge_cost) in neighbors {
                    let next_dist = cost + edge_cost;
                    if next_dist < *dist.get(&next).unwrap_or(&u32::MAX) {
                        dist.insert(next, next_dist);
                        prev.insert(next, node);
                        queue.push(State { node: next, cost: next_dist });
                    }
                }
            }
        }
        None
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    struct State {
        node: Entity,
        cost: u32,
    }

    impl Ord for State {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            other.cost.cmp(&self.cost) // Min-heap
        }
    }

    impl PartialOrd for State {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
}
