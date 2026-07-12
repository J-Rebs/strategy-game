use bevy::prelude::*;
use crate::simulation::{NetworkNode, NetworkLink, Owner, LinkType, GameResources, RoutingTable, NodeType, CityDominance};

// =========================================================================
// PACKETCOMMAND AI PLUG-IN
// =========================================================================
// This file implements the automated decision logic for the three AI opponents
// (AI1, AI2, AI3). It runs independently, executing decisions to buy out competitors,
// upgrade routers to Data Centers, or wire links to adjacent cities.

pub struct AiPlugin;

impl Plugin for AiPlugin {
    /// Registers the AI system to run on the standard update tick.
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ai_decision_loop);
    }
}

/// System: Drives AI actions. Runs periodically to prevent lag and simulate human reaction time.
fn ai_decision_loop(
    mut commands: Commands,
    // Access game resource values (mutable because AI consumes bandwidth and sets eliminations)
    mut game_resources: ResMut<GameResources>,
    // Queries all nodes: Entity ID, NetworkNode component (mutable to update type), Transform position, RoutingTable
    mut nodes: Query<(Entity, &mut NetworkNode, &Transform, &RoutingTable)>,
    // Queries all wires/links in the network
    links: Query<(Entity, &NetworkLink)>,
    // Queries all cities' dominance values to calculate buyout cost
    cities_query: Query<&CityDominance>,
) {
    let tick = game_resources.game_tick;
    
    // To prevent the AI from playing too quickly (and consuming excessive CPU), we use
    // the modulo operator `%`. The AI decision branch executes once every 120 ticks (~2 seconds).
    if tick % 120 != 0 {
        return;
    }

    // Loop through each active AI player in the game
    for current_ai in [Owner::AI1, Owner::AI2, Owner::AI3] {
        // Skip this AI if they have been bought out/eliminated
        let is_eliminated = match current_ai {
            Owner::AI1 => game_resources.ai1_eliminated,
            Owner::AI2 => game_resources.ai2_eliminated,
            Owner::AI3 => game_resources.ai3_eliminated,
            _ => true,
        };
        if is_eliminated {
            continue;
        }

        // Determine target IPs and bandwidth accounts
        let main_dc_ip = match current_ai {
            Owner::AI1 => 100,
            Owner::AI2 => 200,
            Owner::AI3 => 300,
            _ => 0,
        };

        let current_bw = match current_ai {
            Owner::AI1 => game_resources.ai1_bandwidth,
            Owner::AI2 => game_resources.ai2_bandwidth,
            Owner::AI3 => game_resources.ai3_bandwidth,
            _ => 0.0,
        };

        // -------------------------------------------------------------------------
        // DECISION 1: MAIN DATA CENTER BUYOUT
        // -------------------------------------------------------------------------
        // If the 5-minute safety lock has expired, the AI checks if it can afford to
        // buy out any competing team. It prioritizes the cheapest (weakest) competitor.
        if tick >= crate::simulation::BUYOUT_LOCK_TICKS {
            let mut target_to_buyout = None;
            let mut min_buyout_cost = f32::MAX;
            
            // Build a list of potential targets
            let mut target_teams = vec![Owner::Player, Owner::AI1, Owner::AI2, Owner::AI3];
            target_teams.retain(|&t| t != current_ai); // Do not target self
            
            for target in target_teams {
                let target_eliminated = match target {
                    Owner::Player => game_resources.player_eliminated,
                    Owner::AI1 => game_resources.ai1_eliminated,
                    Owner::AI2 => game_resources.ai2_eliminated,
                    Owner::AI3 => game_resources.ai3_eliminated,
                    Owner::Neutral => true,
                };
                if target_eliminated {
                    continue;
                }
                
                // Get dynamic buyout cost (cheaper if they control less map)
                let cost = crate::simulation::get_buyout_cost(target, &cities_query);
                if cost < min_buyout_cost {
                    min_buyout_cost = cost;
                    target_to_buyout = Some(target);
                }
            }

            if let Some(target) = target_to_buyout {
                if current_bw >= min_buyout_cost {
                    // Deduct bandwidth cost
                    match current_ai {
                        Owner::AI1 => game_resources.ai1_bandwidth -= min_buyout_cost,
                        Owner::AI2 => game_resources.ai2_bandwidth -= min_buyout_cost,
                        Owner::AI3 => game_resources.ai3_bandwidth -= min_buyout_cost,
                        _ => {}
                    }
                    // Mark target as eliminated
                    match target {
                        Owner::Player => game_resources.player_eliminated = true,
                        Owner::AI1 => game_resources.ai1_eliminated = true,
                        Owner::AI2 => game_resources.ai2_eliminated = true,
                        Owner::AI3 => game_resources.ai3_eliminated = true,
                        _ => {}
                    }
                    continue; // Action consumed this cycle
                }
            }
        }

        // -------------------------------------------------------------------------
        // DECISION 2: ROUTER TO DATA CENTER UPGRADES
        // -------------------------------------------------------------------------
        // Collect all nodes currently owned by this AI and identify neutral cities.
        let mut ai_nodes = Vec::new();
        let mut target_cities = Vec::new();

        for (entity, node, transform, routing) in nodes.iter() {
            if node.owner == current_ai {
                // Check if the node is linked back to the Main DC via OSPF path
                let is_connected_to_main = routing.route_costs.contains_key(&main_dc_ip) || node.ip == main_dc_ip;
                ai_nodes.push((entity, node.ip, node.node_type, transform.translation, is_connected_to_main));
            } else if node.node_type == NodeType::City {
                target_cities.push((entity, transform.translation));
            }
        }

        if ai_nodes.is_empty() {
            continue;
        }

        // Upgrading a router to a Data Center costs 120 BW and permits branching links.
        let upgrade_cost = 120.0;
        if current_bw >= upgrade_cost {
            let mut upgraded = false;
            for (entity, _, node_type, _, is_connected_to_main) in &ai_nodes {
                if *node_type == NodeType::Router && *is_connected_to_main {
                    // Call get_mut to modify Bevy's NetworkNode component value
                    if let Ok((_, mut node, _, _)) = nodes.get_mut(*entity) {
                        match current_ai {
                            Owner::AI1 => game_resources.ai1_bandwidth -= upgrade_cost,
                            Owner::AI2 => game_resources.ai2_bandwidth -= upgrade_cost,
                            Owner::AI3 => game_resources.ai3_bandwidth -= upgrade_cost,
                            _ => {}
                        }
                        node.node_type = NodeType::DataCenter;
                        upgraded = true;
                        break;
                    }
                }
            }
            if upgraded {
                continue;
            }
        }

        // -------------------------------------------------------------------------
        // DECISION 3: MAP EXPANSION (Lay Wires)
        // -------------------------------------------------------------------------
        // Laying a new copper link costs 60 BW. The AI searches for the closest
        // neutral city within reach and wires to it.
        let link_cost = 60.0;
        if current_bw >= link_cost {
            let mut best_connection = None;
            let mut min_distance = f32::MAX;

            for &(ai_entity, _, _, ai_pos, _) in &ai_nodes {
                for &(city_entity, city_pos) in &target_cities {
                    // Ensure a wire does not already exist
                    let mut link_exists = false;
                    for (_, link) in links.iter() {
                        if (link.node_a == ai_entity && link.node_b == city_entity)
                            || (link.node_a == city_entity && link.node_b == ai_entity)
                        {
                            link_exists = true;
                            break;
                        }
                    }

                    if link_exists {
                        continue;
                    }

                    // Calculate distance to city
                    let dist = ai_pos.distance(city_pos);
                    if dist < min_distance {
                        min_distance = dist;
                        best_connection = Some((ai_entity, city_entity));
                    }
                }
            }

            if let Some((ai_src, target_dst)) = best_connection {
                // Limit connection length to adjacent tiles (within board distance radius)
                if min_distance < 3.2 {
                    match current_ai {
                        Owner::AI1 => game_resources.ai1_bandwidth -= link_cost,
                        Owner::AI2 => game_resources.ai2_bandwidth -= link_cost,
                        Owner::AI3 => game_resources.ai3_bandwidth -= link_cost,
                        _ => {}
                    }
                    // Spawn the connection wire Entity
                    commands.spawn(NetworkLink {
                        node_a: ai_src,
                        node_b: target_dst,
                        link_type: LinkType::Copper,
                        is_active: true,
                    });
                }
            }
        }
    }
}
