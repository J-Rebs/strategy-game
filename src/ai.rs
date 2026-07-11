use bevy::prelude::*;
use crate::simulation::{NetworkNode, NetworkLink, Packet, Owner, LinkType, PacketType, GameResources, RoutingTable};

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ai_decision_loop);
    }
}

fn ai_decision_loop(
    mut commands: Commands,
    mut game_resources: ResMut<GameResources>,
    nodes: Query<(Entity, &NetworkNode, &Transform, &RoutingTable)>,
    links: Query<(Entity, &NetworkLink)>,
    mut packet_id_seq: Local<u32>,
) {
    let tick = game_resources.game_tick;
    // Run decision cycle every 120 ticks (~2 seconds)
    if tick % 120 != 0 {
        return;
    }

    // Collect AI-owned nodes and Target nodes (Neutral/Player)
    let mut ai_nodes = Vec::new();
    let mut target_nodes = Vec::new();

    for (entity, node, transform, routing) in nodes.iter() {
        if node.owner == Owner::AI {
            ai_nodes.push((entity, node.ip, transform.translation, routing));
        } else {
            target_nodes.push((entity, node.ip, node.owner, node.node_type, transform.translation));
        }
    }

    if ai_nodes.is_empty() || target_nodes.is_empty() {
        return;
    }

    // 1. EXPANSION: Find the closest neutral/player node to our AI subnet and link to it
    let mut best_connection = None;
    let mut min_distance = f32::MAX;

    for &(ai_entity, _, ai_pos, _) in &ai_nodes {
        for &(target_entity, _, _, _, target_pos) in &target_nodes {
            // Check if a link already exists between them
            let mut link_exists = false;
            for (_, link) in links.iter() {
                if (link.node_a == ai_entity && link.node_b == target_entity)
                    || (link.node_a == target_entity && link.node_b == ai_entity)
                {
                    link_exists = true;
                    break;
                }
            }

            if link_exists {
                continue;
            }

            let dist = ai_pos.distance(target_pos);
            if dist < min_distance {
                min_distance = dist;
                best_connection = Some((ai_entity, target_entity));
            }
        }
    }

    if let Some((ai_src, target_dst)) = best_connection {
        // Base link cost is based on distance
        let link_cost = (min_distance * 12.0).max(60.0);

        if game_resources.ai_bandwidth >= link_cost {
            game_resources.ai_bandwidth -= link_cost;

            // Spawn a copper link
            commands.spawn(NetworkLink {
                node_a: ai_src,
                node_b: target_dst,
                link_type: LinkType::Copper,
                is_active: true,
            });
            return; // only make one link action per decision cycle
        }
    }

    // 2. OFFENSE: Send Worm packets to capture linked neutral/player nodes
    // Cost per Worm packet = 15.0 bandwidth
    if game_resources.ai_bandwidth >= 15.0 {
        for &(ai_entity, ai_ip, _, _) in &ai_nodes {
            for &(target_entity, target_ip, _, _, _) in &target_nodes {
                // If it is adjacent (linked) and not owned by AI
                let adjacent_link = links.iter().find(|(_, link)| {
                    (link.node_a == ai_entity && link.node_b == target_entity)
                        || (link.node_a == target_entity && link.node_b == ai_entity)
                });

                if let Some((link_entity, _)) = adjacent_link {
                    if game_resources.ai_bandwidth >= 15.0 {
                        game_resources.ai_bandwidth -= 15.0;

                        // Spawn a Worm packet targeting the node
                        *packet_id_seq += 1;
                        commands.spawn(Packet {
                            id: *packet_id_seq,
                            src_ip: ai_ip,
                            dst_ip: target_ip,
                            packet_type: PacketType::Worm,
                            payload_size: 256,
                            link: link_entity,
                            progress: 0.0,
                            from_node: ai_entity,
                            to_node: target_entity,
                            spawn_tick: tick,
                        });
                    }
                }
            }
        }
    }
}
