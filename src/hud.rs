use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::simulation::{NetworkNode, NetworkLink, Owner, NodeType, LinkType, GameResources, RoutingTable, CityDominance};
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
    mut nodes: Query<(Entity, &mut NetworkNode, &RoutingTable, Option<&CityDominance>)>,
    tiles: Query<&HexTile>,
    cities_query: Query<&CityDominance>,
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
                SelectedTool::Inspect => "🔍 INSPECT: Hover over hexes to see coordinates. Click a node to inspect it.",
                SelectedTool::BuildCopperLink => "🔌 LINK: Click one of your nodes (teal), then click an adjacent node to lay a Copper Cable.",
                SelectedTool::BuildFiberLink => "⚡ FIBER: Click one of your nodes (teal), then click an adjacent node to lay a high-speed Fiber Cable.",
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
            
            // Bandwidth balances
            if game_resources.player_eliminated {
                ui.label(egui::RichText::new("Player: ELIMINATED").color(egui::Color32::GRAY));
            } else {
                ui.label(egui::RichText::new(format!("Player: {:.1} Gbps", game_resources.player_bandwidth))
                    .color(egui::Color32::from_rgb(0, 240, 255)).strong());
            }
            ui.separator();
            if game_resources.ai1_eliminated {
                ui.label(egui::RichText::new("AI 1: ELIMINATED").color(egui::Color32::GRAY));
            } else {
                ui.label(egui::RichText::new(format!("AI 1: {:.1} Gbps", game_resources.ai1_bandwidth))
                    .color(egui::Color32::from_rgb(255, 90, 140)).strong());
            }
            ui.separator();
            if game_resources.ai2_eliminated {
                ui.label(egui::RichText::new("AI 2: ELIMINATED").color(egui::Color32::GRAY));
            } else {
                ui.label(egui::RichText::new(format!("AI 2: {:.1} Gbps", game_resources.ai2_bandwidth))
                    .color(egui::Color32::from_rgb(100, 255, 100)).strong());
            }
            ui.separator();
            if game_resources.ai3_eliminated {
                ui.label(egui::RichText::new("AI 3: ELIMINATED").color(egui::Color32::GRAY));
            } else {
                ui.label(egui::RichText::new(format!("AI 3: {:.1} Gbps", game_resources.ai3_bandwidth))
                    .color(egui::Color32::from_rgb(220, 100, 220)).strong());
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Vibe: Cyber Grid").color(egui::Color32::from_rgb(0, 240, 255)));
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

        // 3. Purchase Nodes on selected empty hex
        if let Some(coord) = player_controls.selected_hex {
            // Check if hex is empty
            let is_empty = !nodes.iter().any(|(_, node, _, _)| node.coord == coord);
            
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

    // 3. Right Panel: Inspector
    if let Some(selected_entity) = player_controls.selected_node {
        egui::SidePanel::right("inspector_panel").width_range(260.0..=300.0).show(ctx, |ui| {
            if let Ok((_, mut node, routing_table, city_dom)) = nodes.get_mut(selected_entity) {
                ui.heading(format!("{:?} Node", node.node_type));
                ui.separator();

                ui.label(format!("Hex Position: ({}, {})", node.coord.q, node.coord.r));
                ui.label(format!("IP Address: 10.0.0.{}", node.ip));
                
                let owner_color = match node.owner {
                    Owner::Player => egui::Color32::from_rgb(0, 240, 255),
                    Owner::AI1 => egui::Color32::from_rgb(255, 90, 140),
                    Owner::AI2 => egui::Color32::from_rgb(100, 255, 100),
                    Owner::AI3 => egui::Color32::from_rgb(220, 100, 220),
                    Owner::Neutral => egui::Color32::GRAY,
                };
                ui.label(egui::RichText::new(format!("Owner: {:?}", node.owner)).color(owner_color).strong());
                
                if let Some(dom) = city_dom {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(255, 215, 0), "City Dominance & Control");
                    ui.label(format!("Yield: {:.1} BW/sec", dom.total_payout_rate));
                    ui.label(format!("Player Share: {:.2} BW/sec", dom.player_control_pct * dom.total_payout_rate));
                    ui.label(format!("AI 1 Share: {:.2} BW/sec", dom.ai1_control_pct * dom.total_payout_rate));
                    ui.label(format!("AI 2 Share: {:.2} BW/sec", dom.ai2_control_pct * dom.total_payout_rate));
                    ui.label(format!("AI 3 Share: {:.2} BW/sec", dom.ai3_control_pct * dom.total_payout_rate));
                    
                    let player_pct = dom.player_control_pct;
                    let ai1_pct = dom.ai1_control_pct;
                    let ai2_pct = dom.ai2_control_pct;
                    let ai3_pct = dom.ai3_control_pct;

                    // Donut Chart
                    ui.vertical_centered(|ui| {
                        let size = egui::vec2(120.0, 120.0);
                        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
                        let center = rect.center();
                        let radius = 50.0;
                        
                        let painter = ui.painter();
                        
                        let draw_sector = |painter: &egui::Painter, center: egui::Pos2, radius: f32, start_angle: f32, end_angle: f32, color: egui::Color32| {
                            let num_points = 24;
                            let mut points = vec![center];
                            for i in 0..=num_points {
                                let t = i as f32 / num_points as f32;
                                let angle = start_angle + t * (end_angle - start_angle);
                                let x = center.x + radius * angle.cos();
                                let y = center.y + radius * angle.sin();
                                points.push(egui::pos2(x, y));
                            }
                            painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
                        };
                        
                        let total_pct = player_pct + ai1_pct + ai2_pct + ai3_pct;
                        if total_pct > 0.0 {
                            let start_angle = -std::f32::consts::FRAC_PI_2;
                            let player_sweep = player_pct * std::f32::consts::TAU;
                            let ai1_sweep = ai1_pct * std::f32::consts::TAU;
                            let ai2_sweep = ai2_pct * std::f32::consts::TAU;
                            let ai3_sweep = ai3_pct * std::f32::consts::TAU;
                            
                            let mut current_angle = start_angle;
                            if player_pct > 0.0 {
                                draw_sector(painter, center, radius, current_angle, current_angle + player_sweep, egui::Color32::from_rgb(0, 240, 255));
                                current_angle += player_sweep;
                            }
                            if ai1_pct > 0.0 {
                                draw_sector(painter, center, radius, current_angle, current_angle + ai1_sweep, egui::Color32::from_rgb(255, 90, 140));
                                current_angle += ai1_sweep;
                            }
                            if ai2_pct > 0.0 {
                                draw_sector(painter, center, radius, current_angle, current_angle + ai2_sweep, egui::Color32::from_rgb(100, 255, 100));
                                current_angle += ai2_sweep;
                            }
                            if ai3_pct > 0.0 {
                                draw_sector(painter, center, radius, current_angle, current_angle + ai3_sweep, egui::Color32::from_rgb(220, 100, 220));
                            }
                        } else {
                            painter.circle_filled(center, radius, egui::Color32::from_gray(60));
                        }
                        
                        // Donut inner hole
                        painter.circle_filled(center, radius * 0.5, egui::Color32::from_rgb(8, 28, 40));
                    });

                    ui.label(format!("Player Control: {:.1}%", player_pct * 100.0));
                    ui.label(format!("AI 1 Control: {:.1}%", ai1_pct * 100.0));
                    ui.label(format!("AI 2 Control: {:.1}%", ai2_pct * 100.0));
                    ui.label(format!("AI 3 Control: {:.1}%", ai3_pct * 100.0));
                    ui.separator();
                    ui.label(format!("Player Dominance: {:.1}", dom.player_dominance));
                    ui.label(format!("AI 1 Dominance: {:.1}", dom.ai1_dominance));
                    ui.label(format!("AI 2 Dominance: {:.1}", dom.ai2_dominance));
                    ui.label(format!("AI 3 Dominance: {:.1}", dom.ai3_dominance));
                }

                if node.node_type == NodeType::Router && node.owner == Owner::Player {
                    ui.separator();
                    let is_connected = routing_table.route_costs.contains_key(&10) || node.ip == 10;
                    if is_connected {
                        if ui.button("⚡ Upgrade to Data Center (120 BW)").clicked() {
                            if game_resources.player_bandwidth >= 120.0 {
                                game_resources.player_bandwidth -= 120.0;
                                node.node_type = NodeType::DataCenter;
                            }
                        }
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(255, 90, 90), "⚠️ Cannot Upgrade: Lay wire back to Main Data Center first!");
                    }
                }

                if node.node_type == NodeType::DataCenter && node.owner != Owner::Player && node.owner != Owner::Neutral {
                    let target_owner = node.owner;
                    let cost = crate::simulation::get_buyout_cost(target_owner, &cities_query);
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(255, 215, 0), "🏢 Target Main Data Center");
                    ui.label(format!("Buyout Cost: {:.1} BW", cost));
                    
                    let can_afford = game_resources.player_bandwidth >= cost;
                    if can_afford {
                        if ui.button(format!("🛒 Buy Out {:?}", target_owner)).clicked() {
                            game_resources.player_bandwidth -= cost;
                            match target_owner {
                                Owner::AI1 => game_resources.ai1_eliminated = true,
                                Owner::AI2 => game_resources.ai2_eliminated = true,
                                Owner::AI3 => game_resources.ai3_eliminated = true,
                                _ => {}
                            }
                        }
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(255, 90, 90), "⚠️ Cannot Afford Buyout");
                    }
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

    // Victory / Defeat Overlays
    if game_resources.player_eliminated {
        egui::Window::new("💀 GAME OVER")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(egui::RichText::new("DEFEAT").color(egui::Color32::RED).strong().size(28.0));
                    ui.label("Your Main Data Center has been bought out by the competition.");
                });
            });
    } else if game_resources.ai1_eliminated && game_resources.ai2_eliminated && game_resources.ai3_eliminated {
        egui::Window::new("🎉 VICTORY")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(egui::RichText::new("VICTORY").color(egui::Color32::GREEN).strong().size(28.0));
                    ui.label("You have successfully bought out all competing networks!");
                });
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
    nodes: Query<(Entity, &mut NetworkNode, &Transform)>,
    links: Query<(Entity, &NetworkLink)>,
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
                    clicked_node = Some((entity, node.owner));
                    break;
                }
            }

            // Process actions if a node is clicked
            if let Some((clicked_entity, clicked_owner)) = clicked_node {
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
                    // CPU upgrades and network attack logic removed in simplified version
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
