use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::hex::HexCoord;

// =========================================================================
// PACKETCOMMAND SIMULATION BACKEND
// =========================================================================
// This file implements the core strategy gameplay logic, network protocols,
// resource trickle rates, OSPF Dijkstra routing tables, and player eliminations.
//
// Because this module is purely simulation logic, it runs completely
// independent of rendering/mesh setup. This separation of concerns allows the
// backend to be easily tested in headless unit tests (e.g. tests/match_simulation_tests.rs).

pub const BUYOUT_LOCK_TICKS: u64 = 18000; // 5 minutes safety lock at 60 ticks per second

// -------------------------------------------------------------------------
// DATA STRUCTS (Bevy ECS Components and Resources)
// -------------------------------------------------------------------------

/// Represents the owner of a node or hex tile.
///
/// In Rust, `#[derive(...)]` is a macro that automatically implements standard helper traits
/// (like comparison `PartialEq`, debugging outputs `Debug`, or serialize/deserialize).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, serde::Serialize, serde::Deserialize)]
pub enum Owner {
    Player,
    AI1,
    AI2,
    AI3,
    Neutral,
}

/// The connection type between two routers. Different links have different costs & speeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, serde::Serialize, serde::Deserialize)]
pub enum LinkType {
    Copper,
    Fiber,
    Wireless,
}

// In Rust, we use `impl` blocks to attach methods (functions) to a struct or enum type.
// This is similar to OOP methods, but separates the raw data definition from its behavior.
#[allow(dead_code)]
impl LinkType {
    /// Returns the visual travel speed of packets along this cable type.
    pub fn speed(&self) -> f32 {
        match self {
            LinkType::Copper => 2.0,   // Slower signal propagation
            LinkType::Fiber => 5.0,    // Light-speed propagation
            LinkType::Wireless => 3.5, // Airwaves propagation
        }
    }

    /// The cost in Bandwidth (BW) points to construct this type of link.
    pub fn cost(&self) -> f32 {
        match self {
            LinkType::Copper => 50.0,
            LinkType::Fiber => 150.0,
            LinkType::Wireless => 80.0,
        }
    }

    /// The maximum number of boxes that can travel on this link simultaneously.
    pub fn bandwidth_limit(&self) -> usize {
        match self {
            LinkType::Copper => 5,
            LinkType::Fiber => 25,
            LinkType::Wireless => 10,
        }
    }

    /// Probability that a packet is dropped due to line noise or interference.
    pub fn packet_loss_rate(&self) -> f32 {
        match self {
            LinkType::Copper => 0.01,
            LinkType::Fiber => 0.0,
            LinkType::Wireless => 0.08,
        }
    }
}

/// NetworkNode Category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, serde::Serialize, serde::Deserialize)]
pub enum NodeType {
    Router,
    DataCenter,
    City,
}

/// Packet Category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, serde::Serialize, serde::Deserialize)]
pub enum PacketType {
    Data,
}

/// ECS Component: Represents a packet traveling across a link.
///
/// In Bevy, any struct with `#[derive(Component)]` can be attached to an Entity.
#[derive(Component, Debug, Clone, Reflect)]
pub struct Packet {
    pub id: u32,
    pub src_ip: u32,
    pub dst_ip: u32,
    pub packet_type: PacketType,
    pub payload_size: usize,
    pub link: Entity,   // Reference to the NetworkLink Entity it is traveling on
    pub progress: f32,   // Travel progress from 0.0 (start) to 1.0 (destination)
    pub from_node: Entity,
    pub to_node: Entity,
    pub spawn_tick: u64,
}

/// ECS Component: Represents a network node (Router, Data Center, or City).
#[derive(Component, Debug, Clone, Reflect)]
pub struct NetworkNode {
    pub ip: u32,
    pub coord: HexCoord,
    pub node_type: NodeType,
    pub owner: Owner,
}

/// The physical size categories of neutral Cities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, serde::Serialize, serde::Deserialize)]
pub enum CitySize {
    Small,
    Medium,
    Large,
}

/// ECS Component: Tracks control percentages and score for neutral Cities.
#[derive(Component, Debug, Clone, Reflect)]
pub struct CityDominance {
    pub size: CitySize,
    pub player_dominance: f32,
    pub ai1_dominance: f32,
    pub ai2_dominance: f32,
    pub ai3_dominance: f32,
    pub player_control_pct: f32,
    pub ai1_control_pct: f32,
    pub ai2_control_pct: f32,
    pub ai3_control_pct: f32,
    pub total_payout_rate: f32, // Raw BW points generated per second by this city
}

impl Default for CityDominance {
    fn default() -> Self {
        Self {
            size: CitySize::Medium,
            player_dominance: 0.0,
            ai1_dominance: 0.0,
            ai2_dominance: 0.0,
            ai3_dominance: 0.0,
            player_control_pct: 0.0,
            ai1_control_pct: 0.0,
            ai2_control_pct: 0.0,
            ai3_control_pct: 0.0,
            total_payout_rate: 25.0,
        }
    }
}

/// ECS Component: Represents a wire link connecting two node Entities.
#[derive(Component, Debug, Clone, Reflect)]
pub struct NetworkLink {
    pub node_a: Entity,
    pub node_b: Entity,
    pub link_type: LinkType,
    pub is_active: bool,
}

/// ECS Component: Stores computed path costs for OSPF routing.
///
/// In Rust, `HashMap<K, V>` is a collection mapping keys of type `K` to values of type `V`.
#[derive(Component, Debug, Clone, Default, Reflect)]
pub struct RoutingTable {
    // Maps destination IP -> next-hop Router Entity
    pub routes: HashMap<u32, Entity>,
    // Maps destination IP -> shortest total path cost (latency)
    pub route_costs: HashMap<u32, u32>,
}

/// ECS Resource: Global singletons storing shared game state.
///
/// Resources are registered using `app.init_resource::<T>()` and accessed in systems using
/// `Res<GameResources>` (read-only) or `ResMut<GameResources>` (mutable).
#[derive(Resource, Debug, Clone, Reflect)]
pub struct GameResources {
    pub player_bandwidth: f32,
    pub ai1_bandwidth: f32,
    pub ai2_bandwidth: f32,
    pub ai3_bandwidth: f32,
    pub player_eliminated: bool,
    pub ai1_eliminated: bool,
    pub ai2_eliminated: bool,
    pub ai3_eliminated: bool,
    pub game_tick: u64, // Counts the total simulation ticks elapsed since start
}

impl Default for GameResources {
    fn default() -> Self {
        Self {
            player_bandwidth: 200.0,
            ai1_bandwidth: 200.0,
            ai2_bandwidth: 200.0,
            ai3_bandwidth: 200.0,
            player_eliminated: false,
            ai1_eliminated: false,
            ai2_eliminated: false,
            ai3_eliminated: false,
            game_tick: 0,
        }
    }
}

// -------------------------------------------------------------------------
// SIMULATION PLUGIN REGISTRATION
// -------------------------------------------------------------------------

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    /// Registers all component structures for Bevy's reflection engine and adds update systems.
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
            .register_type::<CityDominance>()
            // `.add_systems(Update, ...)` registers these systems to run every tick of the update loop.
            // `.chain()` ensures they run sequentially in the order listed, preventing race conditions.
            .add_systems(Update, (
                update_game_ticks,
                passive_resource_income,
                spawn_throughput_boxes,
                move_packets,
                update_routing_tables,
                update_city_dominance,
                handle_eliminations,
            ).chain());
    }
}

// -------------------------------------------------------------------------
// SYSTEMS LOGIC (Core Simulation loops)
// -------------------------------------------------------------------------

/// System: Increments the global cycle clock.
fn update_game_ticks(mut game_res: ResMut<GameResources>) {
    game_res.game_tick += 1;
}

/// System: Computes and awards trickle bandwidth to each team.
fn passive_resource_income(
    mut game_res: ResMut<GameResources>,
    cities: Query<&CityDominance>,
) {
    // We iterate through all cities to count how many are dominated (>50% control) by each team.
    let mut player_cities = 0;
    let mut ai1_cities = 0;
    let mut ai2_cities = 0;
    let mut ai3_cities = 0;

    // `cities.iter()` returns an iterator of read-only `&CityDominance` references.
    for city in cities.iter() {
        if city.player_control_pct > 0.5 { player_cities += 1; }
        if city.ai1_control_pct > 0.5 { ai1_cities += 1; }
        if city.ai2_control_pct > 0.5 { ai2_cities += 1; }
        if city.ai3_control_pct > 0.5 { ai3_cities += 1; }
    }

    // Award income (base trickle of 1.0 BW/s + 3.0 BW/s per dominated city, divided by 60 ticks/sec)
    if !game_res.player_eliminated {
        let player_trickle = (1.0 + player_cities as f32 * 3.0) / 60.0;
        game_res.player_bandwidth += player_trickle;
    }
    if !game_res.ai1_eliminated {
        let ai1_trickle = (1.0 + ai1_cities as f32 * 3.0) / 60.0;
        game_res.ai1_bandwidth += ai1_trickle;
    }
    if !game_res.ai2_eliminated {
        let ai2_trickle = (1.0 + ai2_cities as f32 * 3.0) / 60.0;
        game_res.ai2_bandwidth += ai2_trickle;
    }
    if !game_res.ai3_eliminated {
        let ai3_trickle = (1.0 + ai3_cities as f32 * 3.0) / 60.0;
        game_res.ai3_bandwidth += ai3_trickle;
    }
}

/// System: Spawns visual packet data boxes traveling towards cities.
///
/// Spawning tick frequency scales based on the compounded Dijkstra path cost (latency)
/// from the team's starting Main Data Center.
fn spawn_throughput_boxes(
    mut commands: Commands,
    game_res: Res<GameResources>,
    cities: Query<(Entity, &NetworkNode)>,
    nodes: Query<(Entity, &NetworkNode, &RoutingTable)>,
    links: Query<(Entity, &NetworkLink)>,
    // `Local<u32>` is a static system variable that persists its state across runs (used here for sequential IDs)
    mut packet_id_seq: Local<u32>,
) {
    let tick = game_res.game_tick;

    for (city_entity, city_node) in cities.iter() {
        let city_ip = city_node.ip;

        for (link_entity, link) in links.iter() {
            if !link.is_active {
                continue;
            }

            // Find the node connected to the city via this link
            let other_entity = if link.node_a == city_entity {
                Some(link.node_b)
            } else if link.node_b == city_entity {
                Some(link.node_a)
            } else {
                None
            };

            if let Some(other) = other_entity {
                if let Ok((_, other_node, routing)) = nodes.get(other) {
                    let team = other_node.owner;
                    if team == Owner::Neutral {
                        continue;
                    }

                    // Compute path cost (compounded latency) back to the team's starting Main DC
                    let mut compounded_latency = None;
                    let dc_ip_opt = match team {
                        Owner::Player => Some(10),
                        Owner::AI1 => Some(100),
                        Owner::AI2 => Some(200),
                        Owner::AI3 => Some(300),
                        Owner::Neutral => None,
                    };

                    if let Some(dc_ip) = dc_ip_opt {
                        if let Some(&cost) = routing.route_costs.get(&dc_ip) {
                            compounded_latency = Some(cost);
                        } else if other_node.ip == dc_ip {
                            compounded_latency = Some(0); // Directly connected
                        }
                    }

                    // If connected, spawn packet boxes scaled by latency
                    if let Some(latency) = compounded_latency {
                        let base_rate = match link.link_type {
                            LinkType::Copper => 24,
                            LinkType::Wireless => 12,
                            LinkType::Fiber => 4,
                        };

                        let multiplier = (latency as u32).max(1);
                        let rate = base_rate * multiplier;
                        
                        // Perform modulo check: only spawn packet every `rate` ticks
                        if tick % (rate as u64) == 0 {
                            *packet_id_seq += 1;
                            commands.spawn(Packet {
                                id: *packet_id_seq,
                                src_ip: other_node.ip,
                                dst_ip: city_ip,
                                packet_type: PacketType::Data,
                                payload_size: 64,
                                link: link_entity,
                                progress: 0.0,
                                from_node: other,
                                to_node: city_entity,
                                spawn_tick: tick,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// System: Moves packet data boxes along active links.
fn move_packets(
    mut commands: Commands,
    links: Query<&NetworkLink>,
    mut packets: Query<(Entity, &mut Packet)>,
) {
    // `packets.iter_mut()` allows writing to components during iteration
    for (packet_entity, mut packet) in packets.iter_mut() {
        let link_entity = packet.link;
        if let Ok(link) = links.get(link_entity) {
            // Speed factor is converted to progress per tick
            let speed = link.link_type.speed() * 0.01;
            packet.progress += speed;

            // Despawn packet once it arrives at its target node
            if packet.progress >= 1.0 {
                commands.entity(packet_entity).despawn();
            }
        } else {
            // If the wire is severed/despawned, the packet is dropped
            commands.entity(packet_entity).despawn();
        }
    }
}

/// System: Dijkstra Shortest Path calculations to update OSPF routing tables.
///
/// Runs dynamically to find optimal next-hops and compounded latencies.
pub fn update_routing_tables(
    mut nodes: Query<(Entity, &NetworkNode, &mut RoutingTable)>,
    links: Query<&NetworkLink>,
) {
    // 1. Build adjacency list of active links
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
        // Link cost based on speed (Fiber is fastest, Copper slowest)
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

    // 2. Compute Dijkstra shortest path from every node to every other node
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

        // 3. Reconstruct the next-hop entity and path costs
        let mut routes = HashMap::new();
        let mut route_costs = HashMap::new();
        for &dest in &node_entities {
            if src == dest {
                continue;
            }

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
                if let Ok((_, dest_node, _)) = nodes.get(dest) {
                    routes.insert(dest_node.ip, next_hop);
                    if let Some(&cost) = dist.get(&dest) {
                        if cost != u32::MAX {
                            route_costs.insert(dest_node.ip, cost);
                        }
                    }
                }
            }
        }

        // Save paths into the node's RoutingTable component
        if let Ok((_, _, mut node_routing)) = nodes.get_mut(src) {
            node_routing.routes = routes;
            node_routing.route_costs = route_costs;
        }
    }
}

/// System: Calculates city control percentages and distributes bandwidth accordingly.
///
/// Dominance is a function of:
///   - Latency (Path distance cost from the starting Main DC).
///   - Connectivity (Total capacity of links wired into the city).
fn update_city_dominance(
    mut game_resources: ResMut<GameResources>,
    mut cities: Query<(Entity, &NetworkNode, &mut CityDominance)>,
    nodes: Query<(Entity, &NetworkNode, &RoutingTable)>,
    links: Query<&NetworkLink>,
) {
    let mut balance_adjustments_player = 0.0;
    let mut balance_adjustments_ai1 = 0.0;
    let mut balance_adjustments_ai2 = 0.0;
    let mut balance_adjustments_ai3 = 0.0;

    for (city_entity, city_node, mut dominance) in cities.iter_mut() {
        let city_ip = city_node.ip;

        // 1. LATENCY COMPONENT: Find shortest path cost to each player's starting DC
        let mut min_cost_player = u32::MAX;
        let mut min_cost_ai1 = u32::MAX;
        let mut min_cost_ai2 = u32::MAX;
        let mut min_cost_ai3 = u32::MAX;

        for (_, node, routing) in nodes.iter() {
            if node.node_type == NodeType::DataCenter {
                if node.owner == Owner::Player && node.ip == 10 {
                    if let Some(&cost) = routing.route_costs.get(&city_ip) {
                        min_cost_player = cost;
                    }
                }
                if node.owner == Owner::AI1 && node.ip == 100 {
                    if let Some(&cost) = routing.route_costs.get(&city_ip) {
                        min_cost_ai1 = cost;
                    }
                }
                if node.owner == Owner::AI2 && node.ip == 200 {
                    if let Some(&cost) = routing.route_costs.get(&city_ip) {
                        min_cost_ai2 = cost;
                    }
                }
                if node.owner == Owner::AI3 && node.ip == 300 {
                    if let Some(&cost) = routing.route_costs.get(&city_ip) {
                        min_cost_ai3 = cost;
                    }
                }
            }
        }

        // Calculate latency factors (lower path cost yields higher factor)
        let latency_factor_player = if min_cost_player != u32::MAX { 10.0 / (min_cost_player as f32) } else { 0.0 };
        let latency_factor_ai1 = if min_cost_ai1 != u32::MAX { 10.0 / (min_cost_ai1 as f32) } else { 0.0 };
        let latency_factor_ai2 = if min_cost_ai2 != u32::MAX { 10.0 / (min_cost_ai2 as f32) } else { 0.0 };
        let latency_factor_ai3 = if min_cost_ai3 != u32::MAX { 10.0 / (min_cost_ai3 as f32) } else { 0.0 };

        // 2. THROUGHPUT COMPONENT: Accumulate the bandwidth limit of links active into the city
        let mut throughput_player = 0.0;
        let mut throughput_ai1 = 0.0;
        let mut throughput_ai2 = 0.0;
        let mut throughput_ai3 = 0.0;

        for link in links.iter() {
            if !link.is_active {
                continue;
            }

            let other_entity = if link.node_a == city_entity {
                Some(link.node_b)
            } else if link.node_b == city_entity {
                Some(link.node_a)
            } else {
                None
            };

            if let Some(other) = other_entity {
                if let Ok((_, other_node, _)) = nodes.get(other) {
                    let bw = link.link_type.bandwidth_limit() as f32;
                    match other_node.owner {
                        Owner::Player => throughput_player += bw,
                        Owner::AI1 => throughput_ai1 += bw,
                        Owner::AI2 => throughput_ai2 += bw,
                        Owner::AI3 => throughput_ai3 += bw,
                        Owner::Neutral => {}
                    }
                }
            }
        }

        // 3. DOMINANCE SCORE: Dominance = Throughput * Latency Factor
        dominance.player_dominance = throughput_player * latency_factor_player;
        dominance.ai1_dominance = throughput_ai1 * latency_factor_ai1;
        dominance.ai2_dominance = throughput_ai2 * latency_factor_ai2;
        dominance.ai3_dominance = throughput_ai3 * latency_factor_ai3;

        // 4. CONTROL PERCENTAGE: Normalized portion of dominance scores
        let total_dom = dominance.player_dominance + dominance.ai1_dominance + dominance.ai2_dominance + dominance.ai3_dominance;
        if total_dom > 0.0 {
            dominance.player_control_pct = dominance.player_dominance / total_dom;
            dominance.ai1_control_pct = dominance.ai1_dominance / total_dom;
            dominance.ai2_control_pct = dominance.ai2_dominance / total_dom;
            dominance.ai3_control_pct = dominance.ai3_dominance / total_dom;
        } else {
            dominance.player_control_pct = 0.0;
            dominance.ai1_control_pct = 0.0;
            dominance.ai2_control_pct = 0.0;
            dominance.ai3_control_pct = 0.0;
        }

        // 5. ACCUMULATE PAYOUT: Disburse bandwidth from this city according to control share
        let payout_tick = dominance.total_payout_rate / 60.0;
        balance_adjustments_player += dominance.player_control_pct * payout_tick;
        balance_adjustments_ai1 += dominance.ai1_control_pct * payout_tick;
        balance_adjustments_ai2 += dominance.ai2_control_pct * payout_tick;
        balance_adjustments_ai3 += dominance.ai3_control_pct * payout_tick;
    }

    // Apply adjustments to global resource singletons if the team is active
    if !game_resources.player_eliminated {
        game_resources.player_bandwidth += balance_adjustments_player;
    }
    if !game_resources.ai1_eliminated {
        game_resources.ai1_bandwidth += balance_adjustments_ai1;
    }
    if !game_resources.ai2_eliminated {
        game_resources.ai2_bandwidth += balance_adjustments_ai2;
    }
    if !game_resources.ai3_eliminated {
        game_resources.ai3_bandwidth += balance_adjustments_ai3;
    }
}

// -------------------------------------------------------------------------
// HELPERS (OSPF priority items & buyout formulas)
// -------------------------------------------------------------------------

/// Struct used by Dijkstra's min-heap priority queue
#[derive(Copy, Clone, Eq, PartialEq)]
struct DijkstraItem {
    node: Entity,
    cost: u32,
}

impl Ord for DijkstraItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order so the BinaryHeap behaves as a min-heap (lowest cost popped first)
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for DijkstraItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Calculates the overall map control share of a team across all cities.
pub fn get_map_control(team: Owner, cities: &Query<&CityDominance>) -> f32 {
    let mut total_pct = 0.0;
    let mut count = 0;
    for dom in cities.iter() {
        total_pct += match team {
            Owner::Player => dom.player_control_pct,
            Owner::AI1 => dom.ai1_control_pct,
            Owner::AI2 => dom.ai2_control_pct,
            Owner::AI3 => dom.ai3_control_pct,
            Owner::Neutral => 0.0,
        };
        count += 1;
    }
    if count > 0 {
        total_pct / (count as f32)
    } else {
        0.0
    }
}

/// Calculates the dynamic buyout cost of a team's Main Data Center.
///
/// Buying out a target gets cheaper as they control less map territory.
pub fn get_buyout_cost(team: Owner, cities: &Query<&CityDominance>) -> f32 {
    let control = get_map_control(team, cities);
    150.0 + control * 850.0
}

/// System: Resets node ownership to neutral once their team is bought out.
fn handle_eliminations(
    game_res: Res<GameResources>,
    mut nodes: Query<&mut NetworkNode>,
) {
    if game_res.player_eliminated {
        for mut node in nodes.iter_mut() {
            if node.owner == Owner::Player {
                node.owner = Owner::Neutral;
            }
        }
    }
    if game_res.ai1_eliminated {
        for mut node in nodes.iter_mut() {
            if node.owner == Owner::AI1 {
                node.owner = Owner::Neutral;
            }
        }
    }
    if game_res.ai2_eliminated {
        for mut node in nodes.iter_mut() {
            if node.owner == Owner::AI2 {
                node.owner = Owner::Neutral;
            }
        }
    }
    if game_res.ai3_eliminated {
        for mut node in nodes.iter_mut() {
            if node.owner == Owner::AI3 {
                node.owner = Owner::Neutral;
            }
        }
    }
}
