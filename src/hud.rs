use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::simulation::{GameResources, NetworkNode, NetworkLink, RoutingTable, NodeType, LinkType, Owner, CityDominance, GameConfig};
use crate::hex::HexCoord;
use crate::rendering::MainCamera;

// =========================================================================
// PACKETCOMMAND INTERFACE & MOUSE INPUTS (HUD)
// =========================================================================
// This file implements the graphical overlays, dashboard statistics, donut charts,
// and user input click behaviors (tool selections and node building).
//
// It integrates Bevy with `egui` (via the `bevy_egui` library).

/// Selected toolbar action for laying wires or buying routers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum SelectedTool {
    #[default]
    Inspect,
    BuildRouter,
    LayWire,
}

/// ECS Resource: Tracks player selection states, hovering, and tool modes.
#[derive(Resource, Default, Debug, Clone, Reflect)]
pub struct PlayerControls {
    pub selected_tool: SelectedTool,
    pub selected_node: Option<Entity>,
    pub selected_node_coord: Option<HexCoord>,
    pub hovered_hex: Option<HexCoord>,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        // Register dependencies and local resources
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.init_resource::<PlayerControls>()
            .register_type::<SelectedTool>()
            .register_type::<PlayerControls>()
            .add_systems(Update, (draw_hud, handle_mouse_picking).after(bevy_egui::EguiSet::InitContexts));
    }
}

// -------------------------------------------------------------------------
// HUD PANEL & DRAWING SYSTEMS
// -------------------------------------------------------------------------

/// System: Draws the egui overlay dashboard windows.
fn draw_hud(
    mut contexts: EguiContexts,
    mut game_resources: ResMut<GameResources>,
    mut player_controls: ResMut<PlayerControls>,
    mut nodes: Query<(Entity, &mut NetworkNode, &RoutingTable, Option<&CityDominance>)>,
    cities_query: Query<&CityDominance>,
    config: Res<GameConfig>,
) {
    let ctx = contexts.ctx_mut();

    // Light beachside glassmorphism theme (Cornflower, Lemon, Terracotta, Sage)
    let mut visuals = egui::Visuals::light();
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_premultiplied(240, 245, 248, 240);
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(100, 160, 190));

    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_premultiplied(225, 235, 242, 200);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(70, 110, 145));
    visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_premultiplied(253, 242, 200, 255);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(175, 135, 45));
    visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);

    visuals.widgets.active.bg_fill = egui::Color32::from_rgba_premultiplied(238, 185, 165, 255);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(145, 75, 55));
    visuals.widgets.active.rounding = egui::Rounding::same(6.0);

    ctx.set_visuals(visuals);

    // -------------------------------------------------------------------------
    // PANEL A: PLAYER INVENTORY & STATS (Top Screen)
    // -------------------------------------------------------------------------
    egui::TopBottomPanel::top("stats_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading(egui::RichText::new("📡 PacketCommand").color(egui::Color32::from_rgb(70, 110, 145)).strong());
            ui.separator();

            // Display Bandwidth balances and elimination tags
            let p_lbl = if game_resources.player_eliminated { "💀 PLAYER (OFFLINE)".to_string() } else { format!("👤 Player: {:.1} BW", game_resources.player_bandwidth) };
            ui.label(egui::RichText::new(p_lbl).color(egui::Color32::from_rgb(40, 100, 150)).strong());
            ui.separator();

            let ai1_lbl = if game_resources.ai1_eliminated { "💀 AI 1 (OFFLINE)".to_string() } else { format!("🤖 AI 1: {:.1} BW", game_resources.ai1_bandwidth) };
            ui.label(egui::RichText::new(ai1_lbl).color(egui::Color32::from_rgb(150, 60, 50)).strong());
            ui.separator();

            let ai2_lbl = if game_resources.ai2_eliminated { "💀 AI 2 (OFFLINE)".to_string() } else { format!("🤖 AI 2: {:.1} BW", game_resources.ai2_bandwidth) };
            ui.label(egui::RichText::new(ai2_lbl).color(egui::Color32::from_rgb(140, 120, 30)).strong());
            ui.separator();

            let ai3_lbl = if game_resources.ai3_eliminated { "💀 AI 3 (OFFLINE)".to_string() } else { format!("🤖 AI 3: {:.1} BW", game_resources.ai3_bandwidth) };
            ui.label(egui::RichText::new(ai3_lbl).color(egui::Color32::from_rgb(110, 70, 130)).strong());
            ui.separator();

            // Dynamic global timer cycle display
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let seconds = game_resources.game_tick as f32 / 60.0;
                ui.label(format!("Time: {:.1}s", seconds));
            });
        });
    });

    // -------------------------------------------------------------------------
    // PANEL B: TOOLBAR SELECTOR (Left Panel)
    // -------------------------------------------------------------------------
    egui::SidePanel::left("toolbar").width_range(160.0..=220.0).show(ctx, |ui| {
        ui.heading("Operations Toolbar");
        ui.separator();

        ui.selectable_value(&mut player_controls.selected_tool, SelectedTool::Inspect, "🔍 Inspect Node");
        ui.selectable_value(&mut player_controls.selected_tool, SelectedTool::BuildRouter, &format!("📟 Buy Router ({:.0} BW)", config.router_placement_cost));
        ui.selectable_value(&mut player_controls.selected_tool, SelectedTool::LayWire, &format!("🔗 Lay Wire ({:.0} BW)", config.copper_link_cost));

        ui.separator();
        ui.heading("Quick Guide");
        ui.label("1. Select 'Buy Router' and click a green Sage tile adjacent to your active network.");
        ui.label("2. Select 'Lay Wire', click your DC, then drag/click to connect adjacent routers.");
        ui.label("3. Select 'Inspect Node' and click a router to upgrade it to a Data Center.");
        ui.label("4. Capture neutral cities to extract passive bandwidth trickle.");
    });

    // -------------------------------------------------------------------------
    // PANEL C: SELECTION INSPECTOR (Right Panel)
    // -------------------------------------------------------------------------
    if let Some(selected_entity) = player_controls.selected_node {
        egui::SidePanel::right("inspector").width_range(280.0..=360.0).show(ctx, |ui| {
            if let Ok((_, mut node, routing_table, city_dom)) = nodes.get_mut(selected_entity) {
                ui.heading("Selected Network Device");
                ui.separator();

                ui.label(format!("Coordinate: {:?}", node.coord));
                ui.label(format!("Type: {:?}", node.node_type));
                ui.label(format!("IP Address: 10.0.0.{}", node.ip));
                ui.label(format!("Occupying Owner: {:?}", node.owner));

                // If selected node is a City, display a colorful territory division donut chart
                if let Some(dom) = city_dom {
                    ui.separator();
                    ui.heading("City Network Shares");
                    
                    let player_pct = dom.player_control_pct;
                    let ai1_pct = dom.ai1_control_pct;
                    let ai2_pct = dom.ai2_control_pct;
                    let ai3_pct = dom.ai3_control_pct;

                    // egui Painter API for manual vector graphics rendering
                    let chart_size = 120.0;
                    let (_, rect) = ui.allocate_space(egui::vec2(chart_size, chart_size));
                    let painter = ui.painter_at(rect);
                    let center = rect.center();
                    let radius = chart_size * 0.45;

                    ui.painter_at(rect).image(
                        egui::TextureId::default(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::TRANSPARENT
                    );

                    ui.horizontal(|_| {
                        // Drawing divisions as convex polygons
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
                                draw_sector(&painter, center, radius, current_angle, current_angle + player_sweep, egui::Color32::from_rgb(85, 140, 190));
                                current_angle += player_sweep;
                            }
                            if ai1_pct > 0.0 {
                                draw_sector(&painter, center, radius, current_angle, current_angle + ai1_sweep, egui::Color32::from_rgb(205, 110, 95));
                                current_angle += ai1_sweep;
                            }
                            if ai2_pct > 0.0 {
                                draw_sector(&painter, center, radius, current_angle, current_angle + ai2_sweep, egui::Color32::from_rgb(220, 200, 100));
                                current_angle += ai2_sweep;
                            }
                            if ai3_pct > 0.0 {
                                draw_sector(&painter, center, radius, current_angle, current_angle + ai3_sweep, egui::Color32::from_rgb(170, 150, 200));
                            }
                        } else {
                            painter.circle_filled(center, radius, egui::Color32::from_gray(210));
                        }
                        
                        // Donut inner hole: draws background color over polygon center
                        painter.circle_filled(center, radius * 0.5, egui::Color32::from_rgb(240, 245, 248));
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
                        if ui.button(format!("⚡ Upgrade to Data Center ({:.0} BW)", config.router_upgrade_cost)).clicked() {
                            if game_resources.player_bandwidth >= config.router_upgrade_cost {
                                game_resources.player_bandwidth -= config.router_upgrade_cost;
                                node.node_type = NodeType::DataCenter;
                            }
                        }
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(200, 80, 70), "⚠️ Cannot Upgrade: Lay wire back to Main Data Center first!");
                    }
                }

                if node.node_type == NodeType::DataCenter && node.owner != Owner::Player && node.owner != Owner::Neutral {
                    let target_owner = node.owner;
                    let cost = crate::simulation::get_buyout_cost(target_owner, &cities_query, &config);
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(200, 140, 40), "🏢 Target Main Data Center");
                    ui.label(format!("Buyout Cost: {:.1} BW", cost));
                    
                    let tick = game_resources.game_tick;
                    if tick < crate::simulation::BUYOUT_LOCK_TICKS {
                        let remaining_secs = ((crate::simulation::BUYOUT_LOCK_TICKS - tick) as f32 / 60.0).max(0.0);
                        ui.colored_label(egui::Color32::from_rgb(200, 90, 40), format!("🔒 Locked for first 5 mins ({:.1}s remaining)", remaining_secs));
                    } else {
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
                            ui.colored_label(egui::Color32::from_rgb(220, 70, 70), "⚠️ Cannot Afford Buyout");
                        }
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

    // -------------------------------------------------------------------------
    // PANEL D: VICTORY / DEFEAT SCREENS
    // -------------------------------------------------------------------------
    if game_resources.player_eliminated {
        egui::Window::new("💀 GAME OVER")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(egui::RichText::new("DEFEAT").color(egui::Color32::from_rgb(200, 60, 50)).strong().size(28.0));
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
                    ui.heading(egui::RichText::new("VICTORY").color(egui::Color32::from_rgb(60, 160, 80)).strong().size(28.0));
                    ui.label("You have successfully bought out all competing networks!");
                });
            });
    }
}

// -------------------------------------------------------------------------
// MOUSE INTERACTION & WORLD RAYCASTING
// -------------------------------------------------------------------------

/// System: Converts 2D cursor window coordinates to 3D world coordinates.
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
    mut ip_sequence: Local<u32>,
    config: Res<GameConfig>,
) {
    // If the user's cursor is hovering over an egui panel, bypass game board raycasting
    if contexts.ctx_mut().wants_pointer_input() {
        player_controls.hovered_hex = None;
        return;
    }

    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();

    // Raycast: Project vector from camera lens through cursor coordinate into 3D space plane Y=0
    let ray_opt = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor));

    let mut plane_intersection = None;
    if let Some(ray) = ray_opt {
        // Intersect ray with flat horizontal plane Y = 0
        if ray.direction.y.abs() > 0.001 {
            let t = -ray.origin.y / ray.direction.y;
            if t >= 0.0 {
                plane_intersection = Some(ray.origin + ray.direction * t);
            }
        }
    }

    if let Some(pos) = plane_intersection {
        // Convert physical position coordinate into axial grid index (q, r)
        let hex_coord = HexCoord::from_world(pos, 1.0);
        player_controls.hovered_hex = Some(hex_coord);

        // Click actions: Triggered when mouse button is pressed
        if mouse_button_input.just_pressed(MouseButton::Left) {
            
            // Check if there is an existing network device on clicked coordinate
            let mut clicked_node = None;
            for (entity, node, _) in nodes.iter() {
                if node.coord == hex_coord {
                    clicked_node = Some((entity, node.ip));
                    break;
                }
            }

            match player_controls.selected_tool {
                SelectedTool::Inspect => {
                    // Selection inspector mode: store entity and display sidebar
                    if let Some((entity, _)) = clicked_node {
                        player_controls.selected_node = Some(entity);
                        player_controls.selected_node_coord = Some(hex_coord);
                    } else {
                        // Clicked empty ground: clear selection
                        player_controls.selected_node = None;
                        player_controls.selected_node_coord = None;
                    }
                }
                SelectedTool::BuildRouter => {
                    if clicked_node.is_none() {
                        // Enforce placement rule
                        let cost = config.router_placement_cost;
                        let is_adjacent_to_player = nodes.iter().any(|(_, n, _)| {
                            n.owner == Owner::Player && n.coord.distance(&hex_coord) == 1
                        });

                        if is_adjacent_to_player && game_resources.player_bandwidth >= cost {
                            game_resources.player_bandwidth -= cost;
                            
                            // Generate unique IP
                            *ip_sequence += 1;
                            let new_ip = 30 + *ip_sequence;

                            // Spawn the Router Node
                            let new_router = commands.spawn((
                                NetworkNode {
                                    ip: new_ip,
                                    coord: hex_coord,
                                    node_type: NodeType::Router,
                                    owner: Owner::Player,
                                },
                                RoutingTable::default(),
                                Transform::from_translation(hex_coord.to_world(1.0)),
                            )).id();

                            // Automatically wire to the closest adjacent player node
                            let mut closest_player_node = None;
                            let mut min_dist = 999;
                            for (entity, node, _) in nodes.iter() {
                                if node.owner == Owner::Player {
                                    let d = node.coord.distance(&hex_coord);
                                    if d < min_dist {
                                        min_dist = d;
                                        closest_player_node = Some(entity);
                                    }
                                }
                            }

                            if let Some(src_entity) = closest_player_node {
                                commands.spawn(NetworkLink {
                                    node_a: src_entity,
                                    node_b: new_router,
                                    link_type: LinkType::Copper,
                                    is_active: true,
                                });
                            }
                        }
                    }
                }
                SelectedTool::LayWire => {
                    if let Some((clicked_entity, _)) = clicked_node {
                        if let Some(source_entity) = player_controls.selected_node {
                            // Enforce building rules
                            let cost = config.copper_link_cost;
                            if source_entity != clicked_entity {
                                if let Ok((_, src_node, _)) = nodes.get(source_entity) {
                                    if src_node.coord.distance(&hex_coord) == 1 {
                                        
                                        // Ensure link doesn't already exist
                                        let mut link_exists = false;
                                        for (_, link) in links.iter() {
                                            if (link.node_a == source_entity && link.node_b == clicked_entity)
                                                || (link.node_a == clicked_entity && link.node_b == source_entity)
                                            {
                                                link_exists = true;
                                                break;
                                            }
                                        }

                                        if !link_exists && game_resources.player_bandwidth >= cost {
                                            game_resources.player_bandwidth -= cost;
                                            
                                            // Construct the connection wire
                                            commands.spawn(NetworkLink {
                                                node_a: source_entity,
                                                node_b: clicked_entity,
                                                link_type: LinkType::Copper,
                                                is_active: true,
                                            });

                                            // Re-route path coordinates immediately
                                            player_controls.selected_node = None;
                                            player_controls.selected_node_coord = None;
                                        }
                                    }
                                }
                            }
                        } else {
                            // First click: select the source node to start laying wire from
                            player_controls.selected_node = Some(clicked_entity);
                            player_controls.selected_node_coord = Some(hex_coord);
                        }
                    } else {
                        // Clicked empty ground: clear laying wire source
                        player_controls.selected_node = None;
                        player_controls.selected_node_coord = None;
                    }
                }
            }
        }
    } else {
        player_controls.hovered_hex = None;
    }
}
