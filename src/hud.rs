use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::simulation::{NetworkNode, NetworkLink, Owner, NodeType, LinkType, PacketType, GameResources, RoutingTable, FirewallRule, FirewallAction, Packet};
use crate::hex::{HexCoord, HexTile, HexTileType};
use crate::rendering::MainCamera;

// --- Player Controls Resource ---
#[derive(Resource, Default)]
pub struct PlayerControls {
    pub hovered_hex: Option<HexCoord>,
    pub selected_hex: Option<HexCoord>,
    pub selected_node: Option<Entity>,
    pub link_start_node: Option<Entity>,
    pub active_tool: SelectedTool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectedTool {
    #[default]
    Inspect,
    BuildCopperLink,
    BuildFiberLink,
    UpgradeCpu,
    DeployWorm,
    DeployDdos,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<PlayerControls>()
            .add_systems(Update, (draw_hud, handle_mouse_picking));
    }
}

// --- Egui HUD System ---
fn draw_hud(
    mut contexts: EguiContexts,
    mut game_resources: ResMut<GameResources>,
    mut player_controls: ResMut<PlayerControls>,
    mut nodes: Query<(Entity, &mut NetworkNode, &RoutingTable)>,
    tiles: Query<&HexTile>,
    mut commands: Commands,
    mut ip_sequence: Local<u32>,
) {
    let ctx = contexts.ctx_mut();

    // Undersea dark glassmorphism theme using color theory (Complementary Teal & Coral Gold)
    let mut visuals = egui::Visuals::dark();
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_premultiplied(8, 28, 40, 240);
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0, 180, 220));

    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_premultiplied(15, 48, 68, 200);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(180, 240, 255));
    visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_premultiplied(0, 180, 220, 255);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5_f32, egui::Color32::WHITE);
    visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);

    visuals.widgets.active.bg_fill = egui::Color32::from_rgba_premultiplied(255, 184, 92, 255);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0_f32, egui::Color32::BLACK);
    visuals.widgets.active.rounding = egui::Rounding::same(6.0);

    ctx.set_visuals(visuals);

    // Bottom Guidance Panel for interactive instructions
    egui::TopBottomPanel::bottom("guidance_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let guide_text = match player_controls.active_tool {
                SelectedTool::Inspect => "🔍 INSPECT: Hover over hexes to see coordinates. Click a node to view its buffer, health, and rules.",
                SelectedTool::BuildCopperLink => "🔌 LINK: Click one of your nodes (teal), then click an adjacent node to lay a Copper Cable.",
                SelectedTool::BuildFiberLink => "⚡ FIBER: Click one of your nodes (teal), then click an adjacent node to lay a high-speed Fiber Cable.",
                SelectedTool::UpgradeCpu => "🚀 UPGRADE: Click one of your owned nodes to increase its CPU packet processing speed.",
                SelectedTool::DeployWorm => "🐛 WORM: Click your source node, then click an adjacent target node to infect and seize it.",
                SelectedTool::DeployDdos => "💥 DDoS: Click your source node, then click an adjacent target node to flood its buffer queue.",
            };
            ui.colored_label(egui::Color32::from_rgb(255, 215, 100), guide_text);
        });
    });

    // 1. Top Panel: Stats
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("PacketCommand //").color(egui::Color32::from_rgb(0, 240, 255)).strong());
            ui.separator();
            ui.label(format!("Cycle: {}", game_resources.game_tick));
            ui.separator();
            
            // Bandwidth
            ui.label(egui::RichText::new(format!("Player BW: {:.1} Gbps", game_resources.player_bandwidth))
                .color(egui::Color32::from_rgb(0, 255, 180)).strong());
            ui.separator();
            ui.label(egui::RichText::new(format!("AI BW: {:.1} Gbps", game_resources.ai_bandwidth))
                .color(egui::Color32::from_rgb(255, 90, 140)).strong());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Vibe: Undersea Ocean Reef").color(egui::Color32::from_rgb(0, 200, 255)));
            });
        });
    });

    // 2. Left Panel: Construction & Operations
    egui::SidePanel::left("tools_panel").width_range(200.0..=240.0).show(ctx, |ui| {
        ui.heading("Construct & Route");
        ui.separator();

        ui.label("Cursor Mode:");
        if ui.selectable_label(player_controls.active_tool == SelectedTool::Inspect, "🔍 Inspect Hexes").clicked() {
            player_controls.active_tool = SelectedTool::Inspect;
        }

        ui.separator();
        ui.label("Catan Build Costs:");

        let copper_cost = LinkType::Copper.cost();
        let fiber_cost = LinkType::Fiber.cost();

        if ui.selectable_label(
            player_controls.active_tool == SelectedTool::BuildCopperLink,
            format!("🔌 Lay Copper ({} BW)", copper_cost)
        ).clicked() {
            player_controls.active_tool = SelectedTool::BuildCopperLink;
        }

        if ui.selectable_label(
            player_controls.active_tool == SelectedTool::BuildFiberLink,
            format!("⚡ Lay Fiber ({} BW)", fiber_cost)
        ).clicked() {
            player_controls.active_tool = SelectedTool::BuildFiberLink;
        }

        if ui.selectable_label(
            player_controls.active_tool == SelectedTool::UpgradeCpu,
            "🚀 Upgrade Node CPU (100 BW)"
        ).clicked() {
            player_controls.active_tool = SelectedTool::UpgradeCpu;
        }

        ui.separator();
        ui.label("Network Attack Vector:");
        if ui.selectable_label(player_controls.active_tool == SelectedTool::DeployWorm, "🐛 Worm Hacking (30 BW)").clicked() {
            player_controls.active_tool = SelectedTool::DeployWorm;
        }
        if ui.selectable_label(player_controls.active_tool == SelectedTool::DeployDdos, "💥 DDoS Overload (20 BW)").clicked() {
            player_controls.active_tool = SelectedTool::DeployDdos;
        }

        // 3. Purchase Nodes on selected empty hex
        if let Some(coord) = player_controls.selected_hex {
            // Check if hex is empty
            let is_empty = !nodes.iter().any(|(_, node, _)| node.coord == coord);
            
            // Check if tile is buyable (e.g. not water)
            let tile = tiles.iter().find(|t| t.coord == coord);
            let is_land = tile.map_or(false, |t| t.tile_type != HexTileType::Water);

            if is_empty && is_land {
                ui.separator();
                ui.label(format!("Empty Land at ({}, {})", coord.q, coord.r));
                
                if ui.button("🏗️ Buy Router (60 BW)").clicked() {
                    if game_resources.player_bandwidth >= 60.0 {
                        game_resources.player_bandwidth -= 60.0;
                        if *ip_sequence == 0 {
                            *ip_sequence = 30; // player IP sequences start at 30
                        }
                        *ip_sequence += 1;
                        
                        let world_pos = coord.to_world(1.0);
                        commands.spawn((
                            NetworkNode {
                                ip: *ip_sequence,
                                coord,
                                node_type: NodeType::Router,
                                owner: Owner::Player,
                                buffer: std::collections::VecDeque::new(),
                                max_buffer_size: 15,
                                cpu_processing_rate: 3,
                                firewall_rules: Vec::new(),
                                health: 100.0,
                            },
                            RoutingTable::default(),
                            Transform::from_translation(world_pos),
                        ));
                    }
                }

                if ui.button("🏡 Buy Client Node (100 BW)").clicked() {
                    if game_resources.player_bandwidth >= 100.0 {
                        game_resources.player_bandwidth -= 100.0;
                        if *ip_sequence == 0 {
                            *ip_sequence = 30;
                        }
                        *ip_sequence += 1;

                        let world_pos = coord.to_world(1.0);
                        commands.spawn((
                            NetworkNode {
                                ip: *ip_sequence,
                                coord,
                                node_type: NodeType::Client,
                                owner: Owner::Player,
                                buffer: std::collections::VecDeque::new(),
                                max_buffer_size: 15,
                                cpu_processing_rate: 2,
                                firewall_rules: Vec::new(),
                                health: 100.0,
                            },
                            RoutingTable::default(),
                            Transform::from_translation(world_pos),
                        ));
                    }
                }
            }
        }

        if player_controls.link_start_node.is_some() {
            ui.separator();
            ui.colored_label(egui::Color32::from_rgb(255, 200, 0), "Link Source Active. Click next node.");
            if ui.button("Cancel Active Link").clicked() {
                player_controls.link_start_node = None;
            }
        }
    });

    // 3. Right Panel: Inspector & Firewall Rules
    if let Some(selected_entity) = player_controls.selected_node {
        egui::SidePanel::right("inspector_panel").width_range(260.0..=300.0).show(ctx, |ui| {
            if let Ok((_, mut node, routing_table)) = nodes.get_mut(selected_entity) {
                ui.heading(format!("{:?} Node", node.node_type));
                ui.separator();

                ui.label(format!("Hex Position: ({}, {})", node.coord.q, node.coord.r));
                ui.label(format!("IP Address: 10.0.0.{}", node.ip));
                
                let owner_color = match node.owner {
                    Owner::Player => egui::Color32::from_rgb(0, 240, 255),
                    Owner::AI => egui::Color32::from_rgb(255, 90, 140),
                    Owner::Neutral => egui::Color32::GRAY,
                };
                ui.label(egui::RichText::new(format!("Owner: {:?}", node.owner)).color(owner_color).strong());
                
                ui.add(egui::ProgressBar::new(node.health / 100.0)
                    .text(format!("Health: {:.1}%", node.health)));

                ui.label(format!("Processing Speed: {} packets/tick", node.cpu_processing_rate));
                ui.label(format!("Buffer Congestion: {}/{} packets", node.buffer.len(), node.max_buffer_size));

                ui.separator();
                ui.heading("Firewall Settings");
                
                if node.firewall_rules.is_empty() {
                    ui.label("No active packet drop rules.");
                } else {
                    let mut rules_to_remove = Vec::new();
                    for (i, rule) in node.firewall_rules.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let type_str = rule.packet_type.map_or("ANY".to_string(), |t| format!("{:?}", t));
                            ui.label(format!("Drop type = {}", type_str));
                            if ui.button("x").clicked() {
                                rules_to_remove.push(i);
                            }
                        });
                    }

                    for i in rules_to_remove.into_iter().rev() {
                        node.firewall_rules.remove(i);
                    }
                }

                if node.owner == Owner::Player {
                    ui.separator();
                    ui.label("Add Filter Rule:");
                    ui.horizontal(|ui| {
                        if ui.button("Drop DDoS").clicked() {
                            node.firewall_rules.push(FirewallRule {
                                src_ip: None,
                                packet_type: Some(PacketType::Ddos),
                                action: FirewallAction::Drop,
                            });
                        }
                        if ui.button("Drop Worms").clicked() {
                            node.firewall_rules.push(FirewallRule {
                                src_ip: None,
                                packet_type: Some(PacketType::Worm),
                                action: FirewallAction::Drop,
                            });
                        }
                    });
                }

                ui.separator();
                ui.heading("OSPF Route Map");
                if routing_table.routes.is_empty() {
                    ui.label("Isolated node.");
                } else {
                    for (dest_ip, _) in &routing_table.routes {
                        ui.label(format!("Route to 10.0.0.{} -> Path Active", dest_ip));
                    }
                }
            } else {
                player_controls.selected_node = None;
            }
        });
    }
}

fn handle_mouse_picking(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut player_controls: ResMut<PlayerControls>,
    mut game_resources: ResMut<GameResources>,
    mut nodes: Query<(Entity, &mut NetworkNode, &Transform)>,
    links: Query<(Entity, &NetworkLink)>,
    mut packet_id_seq: Local<u32>,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().wants_pointer_input() {
        player_controls.hovered_hex = None;
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    // 1. Raycast for Hover Tracking (Every Frame)
    let mut hovered_coord = None;
    if let Some(cursor_position) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            if ray.direction.y.abs() > 0.0001 {
                let t = -ray.origin.y / ray.direction.y;
                if t > 0.0 {
                    let intersection_point = ray.origin + ray.direction * t;
                    let coord = HexCoord::from_world(intersection_point, 1.0);
                    if coord.is_on_board() {
                        hovered_coord = Some(coord);
                    }
                }
            }
        }
    }
    player_controls.hovered_hex = hovered_coord;

    // 2. Click Handling
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(clicked_coord) = hovered_coord {
            player_controls.selected_hex = Some(clicked_coord);

            // Check if a node structure exists on this HexCoord
            let mut clicked_node = None;
            for (entity, node, _) in nodes.iter() {
                if node.coord == clicked_coord {
                    clicked_node = Some((entity, node.ip, node.owner));
                    break;
                }
            }

            // Process actions if a node is clicked
            if let Some((clicked_entity, clicked_ip, clicked_owner)) = clicked_node {
                match player_controls.active_tool {
                    SelectedTool::Inspect => {
                        player_controls.selected_node = Some(clicked_entity);
                    }
                    SelectedTool::BuildCopperLink | SelectedTool::BuildFiberLink => {
                        let link_type = if player_controls.active_tool == SelectedTool::BuildFiberLink {
                            LinkType::Fiber
                        } else {
                            LinkType::Copper
                        };

                        if let Some(start_entity) = player_controls.link_start_node {
                            if start_entity != clicked_entity {
                                // Check link limit
                                let mut link_exists = false;
                                for (_, link) in links.iter() {
                                    if (link.node_a == start_entity && link.node_b == clicked_entity)
                                        || (link.node_a == clicked_entity && link.node_b == start_entity)
                                    {
                                        link_exists = true;
                                        break;
                                    }
                                }

                                if !link_exists {
                                    let cost = link_type.cost();
                                    if game_resources.player_bandwidth >= cost {
                                        game_resources.player_bandwidth -= cost;
                                        commands.spawn(NetworkLink {
                                            node_a: start_entity,
                                            node_b: clicked_entity,
                                            link_type,
                                            is_active: true,
                                        });
                                    }
                                }
                                player_controls.link_start_node = None;
                            }
                        } else {
                            if clicked_owner == Owner::Player {
                                player_controls.link_start_node = Some(clicked_entity);
                            }
                        }
                    }
                    SelectedTool::UpgradeCpu => {
                        if clicked_owner == Owner::Player && game_resources.player_bandwidth >= 100.0 {
                            game_resources.player_bandwidth -= 100.0;
                            if let Ok((_, mut node, _)) = nodes.get_mut(clicked_entity) {
                                node.cpu_processing_rate += 1;
                            }
                        }
                    }
                    SelectedTool::DeployWorm => {
                        if game_resources.player_bandwidth >= 30.0 {
                            if let Some(start_entity) = player_controls.link_start_node {
                                let mut start_ip = 0;
                                if let Ok((_, start_node, _)) = nodes.get(start_entity) {
                                    start_ip = start_node.ip;
                                }

                                let adjacent_link = links.iter().find(|(_, link)| {
                                    (link.node_a == start_entity && link.node_b == clicked_entity)
                                        || (link.node_a == clicked_entity && link.node_b == start_entity)
                                });

                                if let Some((link_entity, _)) = adjacent_link {
                                    game_resources.player_bandwidth -= 30.0;
                                    *packet_id_seq += 1;
                                    commands.spawn(Packet {
                                        id: *packet_id_seq,
                                        src_ip: start_ip,
                                        dst_ip: clicked_ip,
                                        packet_type: PacketType::Worm,
                                        payload_size: 256,
                                        link: link_entity,
                                        progress: 0.0,
                                        from_node: start_entity,
                                        to_node: clicked_entity,
                                        spawn_tick: game_resources.game_tick,
                                    });
                                }
                            }
                        }
                        player_controls.link_start_node = None;
                    }
                    SelectedTool::DeployDdos => {
                        if game_resources.player_bandwidth >= 20.0 {
                            if let Some(start_entity) = player_controls.link_start_node {
                                let mut start_ip = 0;
                                if let Ok((_, start_node, _)) = nodes.get(start_entity) {
                                    start_ip = start_node.ip;
                                }

                                let adjacent_link = links.iter().find(|(_, link)| {
                                    (link.node_a == start_entity && link.node_b == clicked_entity)
                                        || (link.node_a == clicked_entity && link.node_b == start_entity)
                                });

                                if let Some((link_entity, _)) = adjacent_link {
                                    game_resources.player_bandwidth -= 20.0;
                                    *packet_id_seq += 1;
                                    commands.spawn(Packet {
                                        id: *packet_id_seq,
                                        src_ip: start_ip,
                                        dst_ip: clicked_ip,
                                        packet_type: PacketType::Ddos,
                                        payload_size: 64,
                                        link: link_entity,
                                        progress: 0.0,
                                        from_node: start_entity,
                                        to_node: clicked_entity,
                                        spawn_tick: game_resources.game_tick,
                                    });
                                }
                            }
                        }
                        player_controls.link_start_node = None;
                    }
                }
            } else {
                player_controls.selected_node = None;
            }
        } else {
            // Clicked out of bounds
            player_controls.selected_hex = None;
            player_controls.selected_node = None;
        }
    }
}
