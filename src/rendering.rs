use bevy::prelude::*;
use bevy::math::primitives::Cuboid;
use crate::simulation::{NetworkNode, NetworkLink, Packet, Owner, NodeType, LinkType};
use crate::hex::{HexCoord, create_hex_prism_mesh, HexTile, HexTileType};

// --- Marker Components ---
#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct HexTileMeshMarker {
    pub coord: HexCoord,
}

#[derive(Component)]
pub struct NodeMeshMarker {
    pub node_entity: Entity,
}

#[derive(Component)]
pub struct LinkMeshMarker {
    pub link_entity: Entity,
}

#[derive(Component)]
pub struct PacketMeshMarker {
    pub packet_entity: Entity,
}

#[derive(Component)]
pub struct HoverHighlight;

#[derive(Component)]
pub struct SelectedHighlight;

// --- Camera State Resource ---
#[derive(Resource)]
pub struct CameraState {
    pub center: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            center: Vec3::new(0.0, 0.0, 0.0),
            distance: 12.0, // Start closer to the action
            yaw: 0.785398,  // 45 degrees
            pitch: -0.785398, // -45 degrees
        }
    }
}

// --- Materials Resource ---
#[derive(Resource)]
#[allow(dead_code)]
pub struct GameMaterials {
    pub grass_hex_mat: Handle<StandardMaterial>, // Golden sand
    pub water_hex_mat: Handle<StandardMaterial>, // Deep trench blue
    pub mountain_hex_mat: Handle<StandardMaterial>, // Coral pink
    pub dc_hex_mat: Handle<StandardMaterial>, // Bioluminescent cyan

    pub player_node_mat: Handle<StandardMaterial>,
    pub ai_node_mat: Handle<StandardMaterial>,
    pub neutral_node_mat: Handle<StandardMaterial>,
    pub player_node_glow_mat: Handle<StandardMaterial>,

    pub player_link_mat: Handle<StandardMaterial>,
    pub ai_link_mat: Handle<StandardMaterial>,
    pub neutral_link_mat: Handle<StandardMaterial>,

    pub player_packet_mat: Handle<StandardMaterial>,
    pub ai_packet_mat: Handle<StandardMaterial>,

    pub hover_highlight_mat: Handle<StandardMaterial>,
    pub selected_highlight_mat: Handle<StandardMaterial>,
}

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraState::default())
            // ClearColor: Vibrant underwater sea teal-cyan
            .insert_resource(ClearColor(Color::srgba(0.04, 0.28, 0.38, 1.0)))
            .add_systems(Startup, (setup_camera_and_lights, setup_materials).chain())
            .add_systems(Update, (
                camera_controls,
                sync_hex_tiles,
                sync_node_visuals,
                sync_link_visuals,
                sync_packet_visuals,
                update_highlights,
                update_link_preview,
            ));
    }
}

#[derive(Component)]
pub struct LinkPreviewMarker;

// --- Camera & Lighting Setup ---
fn setup_camera_and_lights(mut commands: Commands) {
    // Spawn 3D camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(8.0, 10.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
    ));

    // Deep cyan underwater ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgba(0.1, 0.5, 0.6, 1.0),
        brightness: 350.0,
    });

    // Intense golden sunbeams filtering down
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 6500.0,
            color: Color::srgba(1.0, 0.95, 0.85, 1.0),
            ..default()
        },
        transform: Transform::from_xyz(10.0, 20.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

// --- Initialize Materials ---
fn setup_materials(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let grass_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.88, 0.76, 0.52, 1.0), // Golden sea sand
        perceptual_roughness: 0.9,
        ..default()
    });
    let water_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.06, 0.18, 0.28, 1.0), // Deep blue ocean trench
        perceptual_roughness: 0.3,
        ..default()
    });
    let mountain_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.42, 0.55, 1.0), // Bright neon pink coral reef
        perceptual_roughness: 0.8,
        ..default()
    });
    let dc_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.85, 0.9, 1.0), // Intense bioluminescent cyan reef
        perceptual_roughness: 0.5,
        ..default()
    });

    let player_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.45, 0.55, 1.0), // Teal coral base
        perceptual_roughness: 0.5,
        ..default()
    });
    let ai_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.1, 0.3, 1.0), // Dark magenta coral base
        perceptual_roughness: 0.5,
        ..default()
    });
    let neutral_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.35, 0.4, 0.42, 1.0), // Slate grey rock
        perceptual_roughness: 0.8,
        ..default()
    });
    let player_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 0.9, 1.0),
        emissive: Color::srgba(0.0, 1.0, 0.9, 1.0).into(),
        ..default()
    });

    let player_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.9, 1.0, 0.6),
        emissive: Color::srgba(0.0, 0.7, 1.0, 1.0).into(),
        ..default()
    });
    let ai_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.25, 0.6, 0.6),
        emissive: Color::srgba(1.0, 0.15, 0.5, 1.0).into(),
        ..default()
    });
    let neutral_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.35, 0.4, 0.3),
        ..default()
    });

    let player_packet_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.4, 0.95, 1.0, 1.0), // Glowing cyan water bubble
        emissive: Color::srgba(0.4, 0.95, 1.0, 1.0).into(),
        ..default()
    });
    let ai_packet_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.4, 0.7, 1.0), // Glowing pink algae bubble
        emissive: Color::srgba(1.0, 0.4, 0.7, 1.0).into(),
        ..default()
    });

    let hover_highlight_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.95, 0.2, 0.3), // Soft glowing yellow highlight ring
        emissive: Color::srgba(1.0, 0.95, 0.2, 1.0).into(),
        ..default()
    });
    let selected_highlight_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 0.4, 0.4), // Soft glowing neon emerald ring
        emissive: Color::srgba(0.0, 1.0, 0.4, 1.0).into(),
        ..default()
    });

    let game_materials = GameMaterials {
        grass_hex_mat,
        water_hex_mat,
        mountain_hex_mat,
        dc_hex_mat,
        player_node_mat,
        ai_node_mat,
        neutral_node_mat,
        player_node_glow_mat,
        player_link_mat,
        ai_link_mat,
        neutral_link_mat,
        player_packet_mat,
        ai_packet_mat,
        hover_highlight_mat: hover_highlight_mat.clone(),
        selected_highlight_mat: selected_highlight_mat.clone(),
    };

    commands.insert_resource(game_materials);

    // Create selection outline mesh (slightly larger hex border outline)
    let highlight_mesh = create_hex_prism_mesh(1.01, 0.05);
    let highlight_mesh_handle = meshes.add(highlight_mesh);

    // Spawn hover highlight outline (initially hidden)
    commands.spawn((
        PbrBundle {
            mesh: highlight_mesh_handle.clone(),
            material: hover_highlight_mat,
            transform: Transform::from_xyz(0.0, -10.0, 0.0),
            visibility: Visibility::Hidden,
            ..default()
        },
        HoverHighlight,
    ));

    // Spawn selected highlight outline (initially hidden)
    commands.spawn((
        PbrBundle {
            mesh: highlight_mesh_handle,
            material: selected_highlight_mat,
            transform: Transform::from_xyz(0.0, -10.0, 0.0),
            visibility: Visibility::Hidden,
            ..default()
        },
        SelectedHighlight,
    ));

    // Spawn link preview entity (initially hidden)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(1.0, 0.04, 0.04))), // placeholder size, scaled dynamically
            material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, 0.6), // semi-transparent white
                emissive: Color::srgba(1.0, 1.0, 1.0, 1.0).into(),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -10.0, 0.0),
            visibility: Visibility::Hidden,
            ..default()
        },
        LinkPreviewMarker,
    ));
}

// --- Orbital Camera Controls ---
fn camera_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_state: ResMut<CameraState>,
    mut query: Query<&mut Transform, With<MainCamera>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    // 1. Pan controls (relative to yaw rotation)
    let mut pan_dir = Vec3::ZERO;
    let forward = Vec3::new(camera_state.yaw.sin(), 0.0, camera_state.yaw.cos()).normalize_or_zero();
    let right = Vec3::new(camera_state.yaw.cos(), 0.0, -camera_state.yaw.sin()).normalize_or_zero();

    if keyboard_input.pressed(KeyCode::KeyW) {
        pan_dir -= forward;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        pan_dir += forward;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        pan_dir -= right;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        pan_dir += right;
    }

    camera_state.center += pan_dir.normalize_or_zero() * 10.0 * dt;

    // 2. Yaw (Rotation around Y)
    if keyboard_input.pressed(KeyCode::KeyQ) {
        camera_state.yaw += 1.2 * dt;
    }
    if keyboard_input.pressed(KeyCode::KeyE) {
        camera_state.yaw -= 1.2 * dt;
    }

    // 3. Zoom (Distance) and Pitch (Angle)
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        camera_state.distance = (camera_state.distance - 12.0 * dt).max(4.0);
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        camera_state.distance = (camera_state.distance + 12.0 * dt).min(28.0);
    }
    if keyboard_input.pressed(KeyCode::PageUp) {
        camera_state.pitch = (camera_state.pitch + 1.0 * dt).min(-0.2);
    }
    if keyboard_input.pressed(KeyCode::PageDown) {
        camera_state.pitch = (camera_state.pitch - 1.0 * dt).max(-1.4);
    }

    // 4. Recalculate camera transform
    if let Ok(mut transform) = query.get_single_mut() {
        let rotation = Quat::from_rotation_y(camera_state.yaw) * Quat::from_rotation_x(camera_state.pitch);
        let offset = rotation * Vec3::new(0.0, 0.0, camera_state.distance);
        transform.translation = camera_state.center + offset;
        let target_transform = Transform::from_translation(camera_state.center + offset)
            .looking_at(camera_state.center, Vec3::Y);
        transform.rotation = target_transform.rotation;
    }
}

// --- Sync Hex Tiles ---
fn sync_hex_tiles(
    mut commands: Commands,
    tiles: Query<(Entity, &HexTile), Changed<HexTile>>,
    mesh_query: Query<(Entity, &HexTileMeshMarker)>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
) {
    for (_tile_entity, tile) in tiles.iter() {
        for (mesh_entity, marker) in mesh_query.iter() {
            if marker.coord == tile.coord {
                commands.entity(mesh_entity).despawn_recursive();
            }
        }

        let pos = tile.coord.to_world(1.0);
        let mat = match tile.tile_type {
            HexTileType::Grass => materials.grass_hex_mat.clone(),
            HexTileType::Water => materials.water_hex_mat.clone(),
            HexTileType::Mountain => materials.mountain_hex_mat.clone(),
            HexTileType::DataCenterCenter => materials.dc_hex_mat.clone(),
        };

        let hex_mesh = create_hex_prism_mesh(0.96, 0.35); // thin tiles

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(hex_mesh),
                material: mat,
                transform: Transform::from_translation(pos - Vec3::new(0.0, 0.175, 0.0)),
                ..default()
            },
            HexTileMeshMarker { coord: tile.coord },
        ));
    }
}

// --- Sync Node Visuals (Bioluminescent Coral structures) ---
fn sync_node_visuals(
    mut commands: Commands,
    nodes: Query<(Entity, &NetworkNode, &Transform), Changed<NetworkNode>>,
    mesh_query: Query<(Entity, &NodeMeshMarker)>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
) {
    for (node_entity, node, transform) in nodes.iter() {
        for (mesh_entity, marker) in mesh_query.iter() {
            if marker.node_entity == node_entity {
                commands.entity(mesh_entity).despawn_recursive();
            }
        }

        let base_mat = match node.owner {
            Owner::Player => materials.player_node_mat.clone(),
            Owner::AI => materials.ai_node_mat.clone(),
            Owner::Neutral => materials.neutral_node_mat.clone(),
        };

        let glow_mat = if node.owner == Owner::Player {
            materials.player_node_glow_mat.clone()
        } else {
            materials.ai_packet_mat.clone()
        };

        commands.spawn((
            SpatialBundle::from_transform(*transform),
            NodeMeshMarker { node_entity },
        )).with_children(|parent| {
            match node.node_type {
                NodeType::Client => {
                    // Sea Anemone: Stacked disks with glowing central bulbs
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.5, 0.25, 0.5))),
                        material: base_mat.clone(),
                        transform: Transform::from_xyz(0.0, 0.125, 0.0),
                        ..default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.3, 0.3, 0.3))),
                        material: glow_mat.clone(),
                        transform: Transform::from_xyz(0.0, 0.35, 0.0),
                        ..default()
                    });
                }
                NodeType::Router | NodeType::Firewall => {
                    // Spiral Coral Spire: thin offset segments
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.25, 0.8, 0.25))),
                        material: base_mat.clone(),
                        transform: Transform::from_xyz(0.0, 0.4, 0.0),
                        ..default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.2, 0.4, 0.2))),
                        material: glow_mat.clone(),
                        transform: Transform::from_xyz(0.0, 0.8, 0.0),
                        ..default()
                    });
                }
                NodeType::DataCenter | NodeType::Ixp => {
                    // Giant bioluminescent crystal clusters
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.4, 1.2, 0.4))),
                        material: base_mat.clone(),
                        transform: Transform::from_xyz(-0.2, 0.6, 0.0),
                        ..default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.35, 1.5, 0.35))),
                        material: base_mat.clone(),
                        transform: Transform::from_xyz(0.2, 0.75, 0.0),
                        ..default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.2, 0.4, 0.2))),
                        material: glow_mat.clone(),
                        transform: Transform::from_xyz(0.2, 1.45, 0.0),
                        ..default()
                    });
                }
            }
        });
    }
}

// --- Sync Link Visuals ---
fn sync_link_visuals(
    mut commands: Commands,
    links: Query<(Entity, &NetworkLink), Changed<NetworkLink>>,
    nodes: Query<&Transform, With<NetworkNode>>,
    mesh_query: Query<(Entity, &LinkMeshMarker)>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
) {
    for (link_entity, link) in links.iter() {
        for (mesh_entity, marker) in mesh_query.iter() {
            if marker.link_entity == link_entity {
                commands.entity(mesh_entity).despawn_recursive();
            }
        }

        if let (Ok(trans_a), Ok(trans_b)) = (nodes.get(link.node_a), nodes.get(link.node_b)) {
            let pos_a = trans_a.translation;
            let pos_b = trans_b.translation;
            let distance = pos_a.distance(pos_b);
            let midpoint = pos_a.lerp(pos_b, 0.5);

            let direction = (pos_b - pos_a).normalize();
            let rotation = Quat::from_rotation_arc(Vec3::X, direction);

            let mat = match link.link_type {
                LinkType::Fiber => materials.player_link_mat.clone(),
                LinkType::Wireless => materials.ai_link_mat.clone(),
                LinkType::Copper => materials.neutral_link_mat.clone(),
            };

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(Cuboid::new(distance, 0.06, 0.06))),
                    material: mat,
                    transform: Transform::from_translation(midpoint + Vec3::new(0.0, 0.04, 0.0))
                        .with_rotation(rotation),
                    ..default()
                },
                LinkMeshMarker { link_entity },
            ));
        }
    }
}

// --- Sync Packet Visuals with bubble bobbing ---
fn sync_packet_visuals(
    mut commands: Commands,
    packets: Query<(Entity, &Packet)>,
    nodes: Query<&Transform, With<NetworkNode>>,
    mut mesh_query: Query<(Entity, &mut Transform, &PacketMeshMarker), Without<NetworkNode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    time: Res<Time>,
) {
    let mut updated_entities = bevy::utils::HashSet::default();
    let time_sec = time.elapsed_seconds();
    let bobbing = (time_sec * 5.0).sin() * 0.06; // smooth sea wave floating bobbing effect
    
    // 1. Move and bob existing packets
    for (mesh_entity, mut mesh_transform, marker) in mesh_query.iter_mut() {
        if let Ok((_, packet)) = packets.get(marker.packet_entity) {
            if let (Ok(trans_a), Ok(trans_b)) = (nodes.get(packet.from_node), nodes.get(packet.to_node)) {
                let pos_a = trans_a.translation;
                let pos_b = trans_b.translation;
                let current_pos = pos_a.lerp(pos_b, packet.progress);
                
                mesh_transform.translation = current_pos + Vec3::new(0.0, 0.25 + bobbing, 0.0);
                updated_entities.insert(marker.packet_entity);
            }
        } else {
            commands.entity(mesh_entity).despawn_recursive();
        }
    }

    // 2. Spawn new bobbing packets
    for (packet_entity, packet) in packets.iter() {
        if updated_entities.contains(&packet_entity) {
            continue;
        }

        if let (Ok(trans_a), Ok(trans_b)) = (nodes.get(packet.from_node), nodes.get(packet.to_node)) {
            let pos_a = trans_a.translation;
            let pos_b = trans_b.translation;
            let current_pos = pos_a.lerp(pos_b, packet.progress);

            let mat = if packet.src_ip < 256 {
                materials.player_packet_mat.clone()
            } else {
                materials.ai_packet_mat.clone()
            };

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(Cuboid::new(0.18, 0.18, 0.18))),
                    material: mat,
                    transform: Transform::from_translation(current_pos + Vec3::new(0.0, 0.25 + bobbing, 0.0)),
                    ..default()
                },
                PacketMeshMarker { packet_entity },
            ));
        }
    }
}

// --- Position Selection Outlines System ---
fn update_highlights(
    player_controls: Res<crate::hud::PlayerControls>,
    mut hover_query: Query<&mut Transform, (With<HoverHighlight>, Without<SelectedHighlight>)>,
    mut select_query: Query<&mut Transform, (With<SelectedHighlight>, Without<HoverHighlight>)>,
    mut hover_visibility: Query<&mut Visibility, (With<HoverHighlight>, Without<SelectedHighlight>)>,
    mut select_visibility: Query<&mut Visibility, (With<SelectedHighlight>, Without<HoverHighlight>)>,
) {
    // 1. Update Hover Highlight
    if let Some(hovered_coord) = player_controls.hovered_hex {
        if let Ok(mut transform) = hover_query.get_single_mut() {
            transform.translation = hovered_coord.to_world(1.0) + Vec3::new(0.0, 0.02, 0.0);
        }
        if let Ok(mut vis) = hover_visibility.get_single_mut() {
            *vis = Visibility::Inherited;
        }
    } else {
        if let Ok(mut vis) = hover_visibility.get_single_mut() {
            *vis = Visibility::Hidden;
        }
    }

    // 2. Update Selected Highlight
    if let Some(selected_coord) = player_controls.selected_hex {
        if let Ok(mut transform) = select_query.get_single_mut() {
            transform.translation = selected_coord.to_world(1.0) + Vec3::new(0.0, 0.03, 0.0);
        }
        if let Ok(mut vis) = select_visibility.get_single_mut() {
            *vis = Visibility::Inherited;
        }
    } else {
        if let Ok(mut vis) = select_visibility.get_single_mut() {
            *vis = Visibility::Hidden;
        }
    }
}

fn update_link_preview(
    player_controls: Res<crate::hud::PlayerControls>,
    nodes: Query<&Transform, With<NetworkNode>>,
    mut preview_query: Query<(&mut Transform, &mut Visibility), (With<LinkPreviewMarker>, Without<NetworkNode>)>,
) {
    let Ok((mut trans, mut vis)) = preview_query.get_single_mut() else { return; };

    if let (Some(start_entity), Some(hovered_coord)) = (player_controls.link_start_node, player_controls.hovered_hex) {
        if let Ok(start_trans) = nodes.get(start_entity) {
            let pos_a = start_trans.translation;
            let pos_b = hovered_coord.to_world(1.0);
            
            let distance = pos_a.distance(pos_b);
            let midpoint = pos_a.lerp(pos_b, 0.5);
            let direction = (pos_b - pos_a).normalize();
            
            if distance > 0.01 {
                let rotation = Quat::from_rotation_arc(Vec3::X, direction);
                trans.translation = midpoint + Vec3::new(0.0, 0.04, 0.0);
                trans.rotation = rotation;
                trans.scale = Vec3::new(distance, 1.0, 1.0);
                *vis = Visibility::Inherited;
            } else {
                *vis = Visibility::Hidden;
            }
        } else {
            *vis = Visibility::Hidden;
        }
    } else {
        *vis = Visibility::Hidden;
    }
}
