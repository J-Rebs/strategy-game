use bevy::prelude::*;
use crate::simulation::{NetworkNode, NetworkLink, Owner, LinkType, GameResources, RoutingTable, NodeType, CityDominance};

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ai_decision_loop);
    }
}

fn ai_decision_loop(
    mut commands: Commands,
    mut game_resources: ResMut<GameResources>,
    mut nodes: Query<(Entity, &mut NetworkNode, &Transform, &RoutingTable)>,
    links: Query<(Entity, &NetworkLink)>,
    cities_query: Query<&CityDominance>,
) {
    let tick = game_resources.game_tick;
    // Run decision cycle every 120 ticks (~2 seconds)
    if tick % 120 != 0 {
        return;
    }

    // Run AI decision loops for all active AI players
    for current_ai in [Owner::AI1, Owner::AI2, Owner::AI3] {
        let is_eliminated = match current_ai {
            Owner::AI1 => game_resources.ai1_eliminated,
            Owner::AI2 => game_resources.ai2_eliminated,
            Owner::AI3 => game_resources.ai3_eliminated,
            _ => true,
        };
        if is_eliminated {
            continue;
        }

        let main_dc_ip = match current_ai {
            Owner::AI1 => 100,
            Owner::AI2 => 200,
            Owner::AI3 => 300,
            _ => 0,
        };

        // Determine current AI bandwidth balance
        let current_bw = match current_ai {
            Owner::AI1 => game_resources.ai1_bandwidth,
            Owner::AI2 => game_resources.ai2_bandwidth,
            Owner::AI3 => game_resources.ai3_bandwidth,
            _ => 0.0,
        };

        // 1. BUYOUT: Check if we can buy out any other active player
        let mut target_to_buyout = None;
        let mut min_buyout_cost = f32::MAX;
        
        let mut target_teams = vec![Owner::Player, Owner::AI1, Owner::AI2, Owner::AI3];
        target_teams.retain(|&t| t != current_ai);
        
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
            
            let cost = crate::simulation::get_buyout_cost(target, &cities_query);
            if cost < min_buyout_cost {
                min_buyout_cost = cost;
                target_to_buyout = Some(target);
            }
        }

        if let Some(target) = target_to_buyout {
            if current_bw >= min_buyout_cost {
                // Execute buyout!
                match current_ai {
                    Owner::AI1 => game_resources.ai1_bandwidth -= min_buyout_cost,
                    Owner::AI2 => game_resources.ai2_bandwidth -= min_buyout_cost,
                    Owner::AI3 => game_resources.ai3_bandwidth -= min_buyout_cost,
                    _ => {}
                }
                match target {
                    Owner::Player => game_resources.player_eliminated = true,
                    Owner::AI1 => game_resources.ai1_eliminated = true,
                    Owner::AI2 => game_resources.ai2_eliminated = true,
                    Owner::AI3 => game_resources.ai3_eliminated = true,
                    _ => {}
                }
                continue; // Consumed action for this team this cycle
            }
        }

        // Collect AI-owned nodes and target cities
        let mut ai_nodes = Vec::new();
        let mut target_cities = Vec::new();

        for (entity, node, transform, routing) in nodes.iter() {
            if node.owner == current_ai {
                let is_connected_to_main = routing.route_costs.contains_key(&main_dc_ip) || node.ip == main_dc_ip;
                ai_nodes.push((entity, node.ip, node.node_type, transform.translation, is_connected_to_main));
            } else if node.node_type == NodeType::City {
                target_cities.push((entity, transform.translation));
            }
        }

        if ai_nodes.is_empty() {
            continue;
        }

        // 2. UPGRADE: Check if we have any Routers we can upgrade to Data Centers
        let upgrade_cost = 120.0;
        if current_bw >= upgrade_cost {
            let mut upgraded = false;
            for (entity, _, node_type, _, is_connected_to_main) in &ai_nodes {
                if *node_type == NodeType::Router && *is_connected_to_main {
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

        // 3. EXPANSION: Find the closest neutral city that we are not linked to and link to it
        let link_cost = 60.0;
        if current_bw >= link_cost {
            let mut best_connection = None;
            let mut min_distance = f32::MAX;

            for &(ai_entity, _, _, ai_pos, _) in &ai_nodes {
                for &(city_entity, city_pos) in &target_cities {
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

                    let dist = ai_pos.distance(city_pos);
                    if dist < min_distance {
                        min_distance = dist;
                        best_connection = Some((ai_entity, city_entity));
                    }
                }
            }

            if let Some((ai_src, target_dst)) = best_connection {
                if min_distance < 3.2 { // Allow building across larger board distances
                    match current_ai {
                        Owner::AI1 => game_resources.ai1_bandwidth -= link_cost,
                        Owner::AI2 => game_resources.ai2_bandwidth -= link_cost,
                        Owner::AI3 => game_resources.ai3_bandwidth -= link_cost,
                        _ => {}
                    }
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
