use bevy::prelude::*;
use bevy::math::primitives::Cuboid;
use crate::simulation::{NetworkNode, NetworkLink, Packet, Owner, NodeType, CityDominance, CitySize};
use crate::hex::{HexCoord, create_hex_prism_mesh, HexTile};

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
    pub grass_hex_mat: Handle<StandardMaterial>, // Cyber Grid base
    pub water_hex_mat: Handle<StandardMaterial>,
    pub mountain_hex_mat: Handle<StandardMaterial>,
    pub dc_hex_mat: Handle<StandardMaterial>,

    // Player/AI Node bases
    pub player_node_mat: Handle<StandardMaterial>,
    pub ai1_node_mat: Handle<StandardMaterial>,
    pub ai2_node_mat: Handle<StandardMaterial>,
    pub ai3_node_mat: Handle<StandardMaterial>,
    pub neutral_node_mat: Handle<StandardMaterial>,

    // Glow overlays
    pub player_node_glow_mat: Handle<StandardMaterial>,
    pub ai1_node_glow_mat: Handle<StandardMaterial>,
    pub ai2_node_glow_mat: Handle<StandardMaterial>,
    pub ai3_node_glow_mat: Handle<StandardMaterial>,
    pub neutral_node_glow_mat: Handle<StandardMaterial>,

    // Links
    pub player_link_mat: Handle<StandardMaterial>,
    pub ai1_link_mat: Handle<StandardMaterial>,
    pub ai2_link_mat: Handle<StandardMaterial>,
    pub ai3_link_mat: Handle<StandardMaterial>,
    pub neutral_link_mat: Handle<StandardMaterial>,

    // Packets
    pub player_packet_mat: Handle<StandardMaterial>,
    pub ai1_packet_mat: Handle<StandardMaterial>,
    pub ai2_packet_mat: Handle<StandardMaterial>,
    pub ai3_packet_mat: Handle<StandardMaterial>,

    // Highlights
    pub hover_highlight_mat: Handle<StandardMaterial>,
    pub selected_highlight_mat: Handle<StandardMaterial>,
}

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraState::default())
            // ClearColor: Dark Cyber Space Black
            .insert_resource(ClearColor(Color::srgba(0.01, 0.02, 0.04, 1.0)))
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
            transform: Transform::from_xyz(10.0, 12.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
    ));

    // Dark cyber-space ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgba(0.05, 0.08, 0.15, 1.0),
        brightness: 200.0,
    });

    // Directional cyber sunbeams
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 3500.0,
            color: Color::srgba(0.7, 0.85, 1.0, 1.0),
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
    // Tile Bases: Sleek Cyber Metal/Grid base
    let grass_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.08, 0.1, 0.15, 1.0), // Dark obsidian blue
        perceptual_roughness: 0.9,
        ..default()
    });
    let water_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.03, 0.05, 0.08, 1.0), // Even darker blue
        perceptual_roughness: 0.4,
        ..default()
    });
    let mountain_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.12, 0.15, 0.22, 1.0), // Elevated plateaus
        perceptual_roughness: 0.8,
        ..default()
    });
    let dc_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.05, 0.2, 0.3, 1.0),
        perceptual_roughness: 0.5,
        ..default()
    });

    // Player and AI Node Bases (Neon Cyber Spire Bases)
    let player_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.3, 0.4, 1.0),
        perceptual_roughness: 0.5,
        ..default()
    });
    let ai1_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.4, 0.05, 0.1, 1.0),
        perceptual_roughness: 0.5,
        ..default()
    });
    let ai2_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.05, 0.4, 0.05, 1.0),
        perceptual_roughness: 0.5,
        ..default()
    });
    let ai3_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.35, 0.05, 0.35, 1.0),
        perceptual_roughness: 0.5,
        ..default()
    });
    let neutral_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.22, 0.25, 1.0), // Neutral slate metal
        perceptual_roughness: 0.7,
        ..default()
    });

    // Neon Glow overlays
    let player_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.9, 1.0, 1.0),
        emissive: Color::srgba(0.0, 0.9, 1.0, 1.0).into(),
        ..default()
    });
    let ai1_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.1, 0.3, 1.0),
        emissive: Color::srgba(1.0, 0.1, 0.3, 1.0).into(),
        ..default()
    });
    let ai2_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 1.0, 0.2, 1.0),
        emissive: Color::srgba(0.2, 1.0, 0.2, 1.0).into(),
        ..default()
    });
    let ai3_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.9, 0.2, 0.9, 1.0),
        emissive: Color::srgba(0.9, 0.2, 0.9, 1.0).into(),
        ..default()
    });
    let neutral_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.55, 0.6, 1.0),
        emissive: Color::srgba(0.2, 0.22, 0.25, 1.0).into(),
        ..default()
    });

    // Links (Glowing circuits)
    let player_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.8, 1.0, 0.6),
        emissive: Color::srgba(0.0, 0.6, 0.8, 1.0).into(),
        ..default()
    });
    let ai1_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.1, 0.3, 0.6),
        emissive: Color::srgba(0.8, 0.05, 0.2, 1.0).into(),
        ..default()
    });
    let ai2_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 1.0, 0.2, 0.6),
        emissive: Color::srgba(0.1, 0.8, 0.1, 1.0).into(),
        ..default()
    });
    let ai3_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.9, 0.2, 0.9, 0.6),
        emissive: Color::srgba(0.7, 0.1, 0.7, 1.0).into(),
        ..default()
    });
    let neutral_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.25, 0.3, 0.3),
        ..default()
    });

    // Packets
    let player_packet_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 1.0, 1.0),
        emissive: Color::srgba(0.0, 1.0, 1.0, 1.0).into(),
        ..default()
    });
    let ai1_packet_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.1, 0.3, 1.0),
        emissive: Color::srgba(1.0, 0.1, 0.3, 1.0).into(),
        ..default()
    });
    let ai2_packet_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 1.0, 0.2, 1.0),
        emissive: Color::srgba(0.2, 1.0, 0.2, 1.0).into(),
        ..default()
    });
    let ai3_packet_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.9, 0.2, 0.9, 1.0),
        emissive: Color::srgba(0.9, 0.2, 0.9, 1.0).into(),
        ..default()
    });

    let hover_highlight_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.9, 0.0, 0.4), // Glowing yellow highlight ring
        emissive: Color::srgba(1.0, 0.9, 0.0, 1.0).into(),
        ..default()
    });
    let selected_highlight_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 0.5, 0.5), // Glowing cyan-green emerald ring
        emissive: Color::srgba(0.0, 1.0, 0.5, 1.0).into(),
        ..default()
    });

    let game_materials = GameMaterials {
        grass_hex_mat,
        water_hex_mat,
        mountain_hex_mat,
        dc_hex_mat,
        player_node_mat,
        ai1_node_mat,
        ai2_node_mat,
        ai3_node_mat,
        neutral_node_mat,
        player_node_glow_mat,
        ai1_node_glow_mat,
        ai2_node_glow_mat,
        ai3_node_glow_mat,
        neutral_node_glow_mat,
        player_link_mat,
        ai1_link_mat,
        ai2_link_mat,
        ai3_link_mat,
        neutral_link_mat,
        player_packet_mat,
        ai1_packet_mat,
        ai2_packet_mat,
        ai3_packet_mat,
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
    tiles: Query<&HexTile>,
    nodes: Query<&NetworkNode>,
    cities: Query<(&NetworkNode, &CityDominance)>,
    mut mesh_query: Query<(Entity, &mut Handle<StandardMaterial>, &HexTileMeshMarker)>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
) {
    // 1. Spawn missing tile meshes on startup
    for tile in tiles.iter() {
        let exists = mesh_query.iter().any(|(_, _, marker)| marker.coord == tile.coord);
        if !exists {
            let pos = tile.coord.to_world(1.0);
            let hex_mesh = create_hex_prism_mesh(0.96, 0.35); // thin tiles
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(hex_mesh),
                    material: materials.grass_hex_mat.clone(),
                    transform: Transform::from_translation(pos - Vec3::new(0.0, 0.175, 0.0)),
                    ..default()
                },
                HexTileMeshMarker { coord: tile.coord },
            ));
        }
    }

    // 2. Dynamically update tile colors based on occupying nodes / dominance
    for (_, mut mat_handle, marker) in mesh_query.iter_mut() {
        let coord = marker.coord;
        let mut tile_owner = Owner::Neutral;
        
        for node in nodes.iter() {
            if node.coord == coord {
                if node.node_type == NodeType::City {
                    if let Some((_, dom)) = cities.iter().find(|(n, _)| n.coord == coord) {
                        if dom.player_control_pct > 0.5 {
                            tile_owner = Owner::Player;
                        } else if dom.ai1_control_pct > 0.5 {
                            tile_owner = Owner::AI1;
                        } else if dom.ai2_control_pct > 0.5 {
                            tile_owner = Owner::AI2;
                        } else if dom.ai3_control_pct > 0.5 {
                            tile_owner = Owner::AI3;
                        }
                    }
                } else {
                    tile_owner = node.owner;
                }
                break;
            }
        }

        let new_mat = match tile_owner {
            Owner::Player => materials.player_link_mat.clone(),
            Owner::AI1 => materials.ai1_link_mat.clone(),
            Owner::AI2 => materials.ai2_link_mat.clone(),
            Owner::AI3 => materials.ai3_link_mat.clone(),
            Owner::Neutral => materials.grass_hex_mat.clone(),
        };

        if *mat_handle != new_mat {
            *mat_handle = new_mat;
        }
    }
}

// --- Sync Node Visuals (Bioluminescent Coral structures) ---
fn sync_node_visuals(
    mut commands: Commands,
    nodes: Query<(Entity, &NetworkNode, &Transform, Option<&CityDominance>), Or<(Changed<NetworkNode>, Changed<CityDominance>)>>,
    mesh_query: Query<(Entity, &NodeMeshMarker)>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
) {
    for (node_entity, node, transform, city_dom) in nodes.iter() {
        for (mesh_entity, marker) in mesh_query.iter() {
            if marker.node_entity == node_entity {
                commands.entity(mesh_entity).despawn_recursive();
            }
        }

        let base_mat = match node.owner {
            Owner::Player => materials.player_node_mat.clone(),
            Owner::AI1 => materials.ai1_node_mat.clone(),
            Owner::AI2 => materials.ai2_node_mat.clone(),
            Owner::AI3 => materials.ai3_node_mat.clone(),
            Owner::Neutral => materials.neutral_node_mat.clone(),
        };

        let mut glow_mat = materials.neutral_node_glow_mat.clone();
        if node.node_type == NodeType::City {
            if let Some(dom) = city_dom {
                if dom.player_control_pct > 0.5 {
                    glow_mat = materials.player_node_glow_mat.clone();
                } else if dom.ai1_control_pct > 0.5 {
                    glow_mat = materials.ai1_node_glow_mat.clone();
                } else if dom.ai2_control_pct > 0.5 {
                    glow_mat = materials.ai2_node_glow_mat.clone();
                } else if dom.ai3_control_pct > 0.5 {
                    glow_mat = materials.ai3_node_glow_mat.clone();
                }
            }
        } else {
            glow_mat = match node.owner {
                Owner::Player => materials.player_node_glow_mat.clone(),
                Owner::AI1 => materials.ai1_node_glow_mat.clone(),
                Owner::AI2 => materials.ai2_node_glow_mat.clone(),
                Owner::AI3 => materials.ai3_node_glow_mat.clone(),
                Owner::Neutral => materials.neutral_node_glow_mat.clone(),
            };
        }

        let scale = if let Some(dom) = city_dom {
            match dom.size {
                CitySize::Small => Vec3::splat(0.7),
                CitySize::Medium => Vec3::splat(1.0),
                CitySize::Large => Vec3::splat(1.4),
            }
        } else {
            Vec3::ONE
        };

        let mut target_transform = *transform;
        target_transform.scale = scale;

        commands.spawn((
            SpatialBundle::from_transform(target_transform),
            NodeMeshMarker { node_entity },
        )).with_children(|parent| {
            match node.node_type {
                NodeType::Router => {
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
                NodeType::DataCenter => {
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
                NodeType::City => {
                    // Futuristic tower cluster
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.4, 1.4, 0.4))),
                        material: base_mat.clone(),
                        transform: Transform::from_xyz(0.0, 0.7, 0.0),
                        ..default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.2, 0.8, 0.2))),
                        material: glow_mat.clone(),
                        transform: Transform::from_xyz(-0.25, 0.4, -0.25),
                        ..default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.2, 0.8, 0.2))),
                        material: glow_mat.clone(),
                        transform: Transform::from_xyz(0.25, 0.4, 0.25),
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
    nodes_query: Query<&NetworkNode>,
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

            let mut link_owner = Owner::Neutral;
            if let Ok(node_a_comp) = nodes_query.get(link.node_a) {
                if node_a_comp.owner != Owner::Neutral {
                    link_owner = node_a_comp.owner;
                }
            }
            if link_owner == Owner::Neutral {
                if let Ok(node_b_comp) = nodes_query.get(link.node_b) {
                    link_owner = node_b_comp.owner;
                }
            }

            let mat = match link_owner {
                Owner::Player => materials.player_link_mat.clone(),
                Owner::AI1 => materials.ai1_link_mat.clone(),
                Owner::AI2 => materials.ai2_link_mat.clone(),
                Owner::AI3 => materials.ai3_link_mat.clone(),
                Owner::Neutral => materials.neutral_link_mat.clone(),
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

// --- Sync Packet Visuals ---
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
    let bobbing = (time_sec * 5.0).sin() * 0.06;

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

    for (packet_entity, packet) in packets.iter() {
        if updated_entities.contains(&packet_entity) {
            continue;
        }

        if let (Ok(trans_a), Ok(trans_b)) = (nodes.get(packet.from_node), nodes.get(packet.to_node)) {
            let pos_a = trans_a.translation;
            let pos_b = trans_b.translation;
            let current_pos = pos_a.lerp(pos_b, packet.progress);

            let mat = if packet.src_ip < 100 {
                materials.player_packet_mat.clone()
            } else if packet.src_ip < 200 {
                materials.ai1_packet_mat.clone()
            } else if packet.src_ip < 300 {
                materials.ai2_packet_mat.clone()
            } else {
                materials.ai3_packet_mat.clone()
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
