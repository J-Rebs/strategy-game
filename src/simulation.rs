use bevy::prelude::*;
use bevy::utils::HashMap;
use std::collections::VecDeque;
use crate::hex::HexCoord;

// --- Owners ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, serde::Serialize, serde::Deserialize)]
pub enum Owner {
    Player,
    AI,
    Neutral,
}

// --- Link Types ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, serde::Serialize, serde::Deserialize)]
pub enum LinkType {
    Copper,
    Fiber,
    Wireless,
}

#[allow(dead_code)]
impl LinkType {
    pub fn speed(&self) -> f32 {
        match self {
            LinkType::Copper => 2.0,   // slower propagation
            LinkType::Fiber => 5.0,    // fast propagation
            LinkType::Wireless => 3.5, // medium propagation
        }
    }

    pub fn cost(&self) -> f32 {
        match self {
            LinkType::Copper => 50.0,
            LinkType::Fiber => 150.0,
            LinkType::Wireless => 80.0,
        }
    }

    pub fn bandwidth_limit(&self) -> usize {
        match self {
            LinkType::Copper => 5,
            LinkType::Fiber => 25,
            LinkType::Wireless => 10,
        }
    }

    pub fn packet_loss_rate(&self) -> f32 {
        match self {
            LinkType::Copper => 0.01,
            LinkType::Fiber => 0.0,
            LinkType::Wireless => 0.08, // Wireless has higher drop rate
        }
    }
}

// --- Node Types ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, serde::Serialize, serde::Deserialize)]
pub enum NodeType {
    Client,
    Router,
    Firewall,
    DataCenter, // Resource node that pays out
    Ixp,        // Internet Exchange Point (performance booster)
}

// --- Firewall Rules ---
#[derive(Debug, Clone, Reflect, serde::Serialize, serde::Deserialize)]
pub struct FirewallRule {
    pub src_ip: Option<u32>,
    pub packet_type: Option<PacketType>,
    pub action: FirewallAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, serde::Serialize, serde::Deserialize)]
pub enum FirewallAction {
    Allow,
    Drop,
}

// --- Packet Types ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, serde::Serialize, serde::Deserialize)]
pub enum PacketType {
    Syn,
    SynAck,
    Ack,
    Data,
    Response,
    Ddos,
    Worm,
}

// --- Packets ---
#[derive(Component, Debug, Clone, Reflect)]
pub struct Packet {
    pub id: u32,
    pub src_ip: u32,
    pub dst_ip: u32,
    pub packet_type: PacketType,
    pub payload_size: usize,
    pub link: Entity,
    pub progress: f32, // 0.0 to 1.0 along the link
    pub from_node: Entity,
    pub to_node: Entity,
    pub spawn_tick: u64,
}

// --- Nodes ---
#[derive(Component, Debug, Clone, Reflect)]
pub struct NetworkNode {
    pub ip: u32,
    pub coord: HexCoord,
    pub node_type: NodeType,
    pub owner: Owner,
    pub buffer: VecDeque<PacketData>,
    pub max_buffer_size: usize,
    pub cpu_processing_rate: usize, // packets processed per tick
    pub firewall_rules: Vec<FirewallRule>,
    pub health: f32, // 0.0 to 100.0 (Worm payload damages health, capturing the node)
}

#[derive(Debug, Clone, Reflect)]
pub struct PacketData {
    pub id: u32,
    pub src_ip: u32,
    pub dst_ip: u32,
    pub packet_type: PacketType,
    pub payload_size: usize,
    pub spawn_tick: u64,
}

// --- Links ---
#[derive(Component, Debug, Clone, Reflect)]
pub struct NetworkLink {
    pub node_a: Entity,
    pub node_b: Entity,
    pub link_type: LinkType,
    pub is_active: bool,
}

// --- Routing Tables ---
#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct RoutingTable {
    // Maps destination IP -> next-hop Node Entity
    pub routes: HashMap<u32, Entity>,
}

// --- Global Resources ---
#[derive(Resource, Debug, Clone, Reflect)]
pub struct GameResources {
    pub player_bandwidth: f32,
    pub ai_bandwidth: f32,
    pub game_tick: u64,
}

impl Default for GameResources {
    fn default() -> Self {
        Self {
            player_bandwidth: 200.0, // starting balance
            ai_bandwidth: 200.0,
            game_tick: 0,
        }
    }
}

// --- Simulation Plugin ---
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameResources>()
            .register_type::<Owner>()
            .register_type::<LinkType>()
            .register_type::<NodeType>()
            .register_type::<PacketType>()
            .register_type::<Packet>()
            .register_type::<NetworkNode>()
            .register_type::<NetworkLink>()
            .register_type::<RoutingTable>()
            .register_type::<GameResources>()
            .add_systems(Update, (
                update_game_ticks,
                client_spawn_requests,
                move_packets,
                process_node_queues,
                update_routing_tables, // runs dynamically
            ).chain());
    }
}

// --- Systems ---

fn update_game_ticks(mut game_res: ResMut<GameResources>) {
    game_res.game_tick += 1;
}

// Client nodes periodically generate SYN requests to Data Centers
fn client_spawn_requests(
    mut commands: Commands,
    game_res: Res<GameResources>,
    nodes: Query<(Entity, &NetworkNode, &RoutingTable)>,
    links: Query<(Entity, &NetworkLink)>,
    mut packet_id_seq: Local<u32>,
) {
    // Only spawn requests every 30 ticks (e.g., 0.5s at 60fps)
    if game_res.game_tick % 30 != 0 {
        return;
    }

    // Find all Data Centers (which must be claimed/active)
    let dcs: Vec<(Owner, u32)> = nodes
        .iter()
        .filter(|(_, node, _)| node.node_type == NodeType::DataCenter)
        .map(|(_, node, _)| (node.owner, node.ip))
        .filter(|(owner, _)| *owner != Owner::Neutral)
        .collect();

    if dcs.is_empty() {
        return;
    }

    // Each Client sends a packet to a Data Center owned by the same team
    for (client_entity, client_node, routing_table) in nodes.iter() {
        if client_node.node_type != NodeType::Client || client_node.owner == Owner::Neutral {
            continue;
        }

        // Try to find a DC owned by the same team, or fall back to any DC
        let target_dc = dcs.iter()
            .find(|(dc_owner, _)| *dc_owner == client_node.owner)
            .or_else(|| dcs.first())
            .map(|(_, dc_ip)| *dc_ip);

        if let Some(dest_ip) = target_dc {
            // Find next hop towards destination DC
            if let Some(&next_hop_entity) = routing_table.routes.get(&dest_ip) {
                // Find the link connecting client_entity to next_hop_entity
                if let Some(link_entity) = find_link_between(client_entity, next_hop_entity, &links) {
                    *packet_id_seq += 1;
                    commands.spawn(Packet {
                        id: *packet_id_seq,
                        src_ip: client_node.ip,
                        dst_ip: dest_ip,
                        packet_type: PacketType::Syn,
                        payload_size: 64,
                        link: link_entity,
                        progress: 0.0,
                        from_node: client_entity,
                        to_node: next_hop_entity,
                        spawn_tick: game_res.game_tick,
                    });
                }
            }
        }
    }
}

// Clean helper to query link entities
fn find_link_between(
    node_a: Entity,
    node_b: Entity,
    links: &Query<(Entity, &NetworkLink)>,
) -> Option<Entity> {
    for (entity, link) in links.iter() {
        if (link.node_a == node_a && link.node_b == node_b)
            || (link.node_a == node_b && link.node_b == node_a)
        {
            return Some(entity);
        }
    }
    None
}

// Packet movement system
fn move_packets(
    mut commands: Commands,
    _game_res: Res<GameResources>,
    links: Query<&NetworkLink>,
    mut packets: Query<(Entity, &mut Packet)>,
    mut nodes: Query<&mut NetworkNode>,
) {
    for (packet_entity, mut packet) in packets.iter_mut() {
        let link_entity = packet.link;
        if let Ok(link) = links.get(link_entity) {
            // Calculate speed factor
            let speed = link.link_type.speed() * 0.01;
            packet.progress += speed;

            if packet.progress >= 1.0 {
                // Arrived at destination node!
                let to_node_entity = packet.to_node;
                if let Ok(mut node) = nodes.get_mut(to_node_entity) {
                    // Process firewall rules
                    let mut is_dropped = false;
                    for rule in &node.firewall_rules {
                        let match_src = rule.src_ip.map_or(true, |ip| ip == packet.src_ip);
                        let match_type = rule.packet_type.map_or(true, |t| t == packet.packet_type);
                        if match_src && match_type {
                            if rule.action == FirewallAction::Drop {
                                is_dropped = true;
                            }
                            break;
                        }
                    }

                    if is_dropped {
                        // Drop packet
                        commands.entity(packet_entity).despawn();
                        continue;
                    }

                    // Check buffer space
                    if node.buffer.len() < node.max_buffer_size {
                        node.buffer.push_back(PacketData {
                            id: packet.id,
                            src_ip: packet.src_ip,
                            dst_ip: packet.dst_ip,
                            packet_type: packet.packet_type,
                            payload_size: packet.payload_size,
                            spawn_tick: packet.spawn_tick,
                        });
                    } else {
                        // Queue full -> packet drop!
                        // (visualize packet loss)
                    }
                }
                commands.entity(packet_entity).despawn();
            }
        } else {
            // Link went down, packet lost in transit
            commands.entity(packet_entity).despawn();
        }
    }
}

// Processes queues at each node (Router forwarding, Server response spawning)
fn process_node_queues(
    mut commands: Commands,
    mut game_resources: ResMut<GameResources>,
    mut nodes: Query<(Entity, &mut NetworkNode, &RoutingTable)>,
    links: Query<(Entity, &NetworkLink)>,
    mut packet_id_seq: Local<u32>,
) {
    let tick = game_resources.game_tick;
    // We need to collect actions first to avoid multiple mutable borrows of the nodes query
    let mut packets_to_spawn = Vec::new();
    let mut worm_captures = Vec::new();
    let mut balance_adjustments = Vec::new();

    for (node_entity, mut node, routing_table) in nodes.iter_mut() {
        // Process up to cpu_processing_rate packets
        let process_count = node.cpu_processing_rate.min(node.buffer.len());
        for _ in 0..process_count {
            if let Some(packet_data) = node.buffer.pop_front() {
                // If packet is destined for this node's IP
                if packet_data.dst_ip == node.ip {
                    match (node.node_type, packet_data.packet_type) {
                        (NodeType::DataCenter, PacketType::Syn) => {
                            // DC received request: respond with SynAck
                            packets_to_spawn.push((
                                node_entity,
                                packet_data.dst_ip, // src is now DC
                                packet_data.src_ip, // dst is Client
                                PacketType::SynAck,
                                64,
                                packet_data.spawn_tick,
                            ));
                        }
                        (NodeType::Client, PacketType::SynAck) => {
                            // Client received SynAck: respond with Ack
                            packets_to_spawn.push((
                                node_entity,
                                packet_data.dst_ip,
                                packet_data.src_ip,
                                PacketType::Ack,
                                64,
                                packet_data.spawn_tick,
                            ));
                        }
                        (NodeType::DataCenter, PacketType::Ack) => {
                            // DC received Ack: start transmitting Data
                            packets_to_spawn.push((
                                node_entity,
                                packet_data.dst_ip,
                                packet_data.src_ip,
                                PacketType::Data,
                                1024,
                                packet_data.spawn_tick,
                            ));
                        }
                        (NodeType::Client, PacketType::Data) => {
                            // Transaction completed successfully!
                            // Client pays out resources
                            let rtt = tick.saturating_sub(packet_data.spawn_tick);
                            let payout = 10.0 * (100.0 / (rtt as f32).max(10.0)).min(3.0);
                            balance_adjustments.push((node.owner, payout));
                        }
                        (_, PacketType::Worm) => {
                            // Infected! Damage health
                            node.health -= 25.0;
                            if node.health <= 0.0 {
                                worm_captures.push((node_entity, packet_data.src_ip));
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Forwarding: Route the packet to the next hop
                    if let Some(&next_hop) = routing_table.routes.get(&packet_data.dst_ip) {
                        if find_link_between(node_entity, next_hop, &links).is_some() {
                            packets_to_spawn.push((
                                node_entity,
                                packet_data.src_ip,
                                packet_data.dst_ip,
                                packet_data.packet_type,
                                packet_data.payload_size,
                                packet_data.spawn_tick,
                            ));
                        }
                    }
                }
            }
        }
    }

    // Apply adjustments
    for (owner, payout) in balance_adjustments {
        match owner {
            Owner::Player => game_resources.player_bandwidth += payout,
            Owner::AI => game_resources.ai_bandwidth += payout,
            Owner::Neutral => {}
        }
    }

    for (node_entity, attacker_ip) in worm_captures {
        // Change ownership of the node to the attacker
        if let Ok((_, mut node, _)) = nodes.get_mut(node_entity) {
            // Find which owner has the attacker IP. For simplicity:
            // If player IP starts with 10.0.0.X and AI starts with 10.0.1.X:
            let new_owner = if attacker_ip >= 256 { Owner::AI } else { Owner::Player };
            node.owner = new_owner;
            node.health = 100.0; // reset health
            node.buffer.clear(); // clear backlog
        }
    }

    // Spawn new packets in transit
    for (from_node_entity, src_ip, dst_ip, packet_type, size, spawn_tick) in packets_to_spawn {
        // We need to look up the routing next-hop again to assign the link
        if let Ok((_, _, routing_table)) = nodes.get(from_node_entity) {
            if let Some(&next_hop) = routing_table.routes.get(&dst_ip) {
                if let Some(link_entity) = find_link_between(from_node_entity, next_hop, &links) {
                    *packet_id_seq += 1;
                    commands.spawn(Packet {
                        id: *packet_id_seq,
                        src_ip,
                        dst_ip,
                        packet_type,
                        payload_size: size,
                        link: link_entity,
                        progress: 0.0,
                        from_node: from_node_entity,
                        to_node: next_hop,
                        spawn_tick,
                    });
                }
            }
        }
    }
}

// OSPF Dijkstra shortest-path to update routing tables
pub fn update_routing_tables(
    mut nodes: Query<(Entity, &NetworkNode, &mut RoutingTable)>,
    links: Query<&NetworkLink>,
) {
    // 1. Build a list of active links/edges in the topology
    let mut adj = HashMap::new();
    let mut ip_to_entity = HashMap::new();

    for (entity, node, _) in nodes.iter() {
        adj.insert(entity, Vec::new());
        ip_to_entity.insert(node.ip, entity);
    }

    for link in links.iter() {
        if !link.is_active {
            continue;
        }
        // Link cost is based on link type speed (lower is better/faster)
        let cost = match link.link_type {
            LinkType::Fiber => 1,
            LinkType::Wireless => 2,
            LinkType::Copper => 3,
        };

        if adj.contains_key(&link.node_a) && adj.contains_key(&link.node_b) {
            adj.get_mut(&link.node_a).unwrap().push((link.node_b, cost));
            adj.get_mut(&link.node_b).unwrap().push((link.node_a, cost));
        }
    }

    // 2. Compute Dijkstra shortest path for each node source
    let node_entities: Vec<Entity> = adj.keys().cloned().collect();

    for &src in &node_entities {
        let mut dist = HashMap::new();
        let mut prev = HashMap::new();
        let mut unvisited = std::collections::BinaryHeap::new();

        for &node in &node_entities {
            dist.insert(node, u32::MAX);
        }
        dist.insert(src, 0);
        unvisited.push(DijkstraItem { node: src, cost: 0 });

        while let Some(DijkstraItem { node: u, cost: d }) = unvisited.pop() {
            if d > *dist.get(&u).unwrap_or(&u32::MAX) {
                continue;
            }

            if let Some(neighbors) = adj.get(&u) {
                for &(v, weight) in neighbors {
                    let alt = d + weight;
                    if alt < *dist.get(&v).unwrap_or(&u32::MAX) {
                        dist.insert(v, alt);
                        prev.insert(v, u);
                        unvisited.push(DijkstraItem { node: v, cost: alt });
                    }
                }
            }
        }

        // 3. Reconstruct next-hop router for each destination
        let mut routes = HashMap::new();
        for &dest in &node_entities {
            if src == dest {
                continue;
            }

            // Backtrack from dest to src
            let mut curr = dest;
            let mut path = Vec::new();
            while let Some(&p) = prev.get(&curr) {
                path.push(curr);
                if p == src {
                    break;
                }
                curr = p;
            }

            if let Some(&next_hop) = path.last() {
                // Get IP of destination node
                if let Ok((_, dest_node, _)) = nodes.get(dest) {
                    routes.insert(dest_node.ip, next_hop);
                }
            }
        }

        // Update routing table component
        if let Ok((_, _, mut node_routing)) = nodes.get_mut(src) {
            node_routing.routes = routes;
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct DijkstraItem {
    node: Entity,
    cost: u32,
}

impl Ord for DijkstraItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order for min-heap
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for DijkstraItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
