use bevy::prelude::*;
use crate::simulation::{NetworkNode, NetworkLink, Owner, LinkType, GameResources, RoutingTable, NodeType};

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
) {
    let tick = game_resources.game_tick;
    // Run decision cycle every 120 ticks (~2 seconds)
    if tick % 120 != 0 {
        return;
    }

    // Collect AI-owned nodes and target cities
    let mut ai_nodes = Vec::new();
    let mut target_cities = Vec::new();

    for (entity, node, transform, routing) in nodes.iter() {
        if node.owner == Owner::AI {
            let is_connected_to_main = routing.route_costs.contains_key(&200) || node.ip == 200;
            ai_nodes.push((entity, node.ip, node.node_type, transform.translation, is_connected_to_main));
        } else if node.node_type == NodeType::City {
            target_cities.push((entity, transform.translation));
        }
    }

    if ai_nodes.is_empty() {
        return;
    }

    // 1. UPGRADE: Check if we have any Routers we can upgrade to Data Centers
    if game_resources.ai_bandwidth >= 120.0 {
        for (entity, _, node_type, _, is_connected_to_main) in &ai_nodes {
            if *node_type == NodeType::Router && *is_connected_to_main {
                if let Ok((_, mut node, _, _)) = nodes.get_mut(*entity) {
                    game_resources.ai_bandwidth -= 120.0;
                    node.node_type = NodeType::DataCenter;
                    return; // Upgrade consumes the action for this cycle
                }
            }
        }
    }

    // 2. EXPANSION: Find the closest neutral city that we are not linked to and link to it
    if game_resources.ai_bandwidth >= 60.0 {
        let mut best_connection = None;
        let mut min_distance = f32::MAX;

        for &(ai_entity, _, _, ai_pos, _) in &ai_nodes {
            for &(city_entity, city_pos) in &target_cities {
                // Check if a link already exists
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
            // Check adjacency distance
            if min_distance < 2.0 {
                game_resources.ai_bandwidth -= 60.0;
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
