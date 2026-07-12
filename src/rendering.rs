use bevy::prelude::*;
use bevy::math::primitives::Cuboid;
use crate::simulation::{NetworkNode, NetworkLink, Packet, Owner, NodeType, CityDominance, CitySize};
use crate::hex::{HexCoord, create_hex_prism_mesh, HexTile, HexTileType};

// =========================================================================
// PACKETCOMMAND RENDERING & 3D VISUALIZATION
// =========================================================================
// This file controls the visual representation of the game board. It is
// structured as a Bevy Plugin (`RenderingPlugin`) which runs visual sync systems.
//
// In ECS, the rendering layer behaves as an observer: it queries the simulation
// components (like `NetworkNode` and `NetworkLink`) and spawns/updates 3D meshes
// (PbrBundle) to reflect the backend state.

// --- Marker Components ---
// These are simple empty structs attached to entities to easily query or filter them in systems.

#[derive(Component)]
pub struct MainCamera;

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
pub struct HoverHighlightMarker;

#[derive(Component)]
pub struct SelectedHighlightMarker;

// --- Resources ---

/// Tracks the position and zoom level of the camera.
#[derive(Resource, Default)]
pub struct CameraState {
    pub radius: f32,
    pub azimuth: f32, // Horizontal rotation
    pub polar: f32,   // Vertical rotation
}

/// Stores handles to pre-loaded standard materials for our seaside pastel theme.
///
/// In Bevy, assets (like meshes, textures, or materials) are loaded once into
/// an `Assets<T>` catalog. Systems reference them using cheap, cloneable `Handle<T>` pointers,
/// which avoids duplicating materials in GPU memory.
#[derive(Resource, Clone)]
pub struct GameMaterials {
    // Terrain Tiles
    pub grass_hex_mat: Handle<StandardMaterial>,
    pub water_hex_mat: Handle<StandardMaterial>,
    pub mountain_hex_mat: Handle<StandardMaterial>,
    pub dc_hex_mat: Handle<StandardMaterial>,

    // Node Bases (Whitewashed Plaster/Stucco)
    pub player_node_mat: Handle<StandardMaterial>,
    pub ai1_node_mat: Handle<StandardMaterial>,
    pub ai2_node_mat: Handle<StandardMaterial>,
    pub ai3_node_mat: Handle<StandardMaterial>,
    pub neutral_node_mat: Handle<StandardMaterial>,

    // Node Tops (Pastel Accents)
    pub player_node_glow_mat: Handle<StandardMaterial>,
    pub ai1_node_glow_mat: Handle<StandardMaterial>,
    pub ai2_node_glow_mat: Handle<StandardMaterial>,
    pub ai3_node_glow_mat: Handle<StandardMaterial>,
    pub neutral_node_glow_mat: Handle<StandardMaterial>,

    // Wires (Painted paths)
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

    // Outlines
    pub hover_highlight_mat: Handle<StandardMaterial>,
    pub selected_highlight_mat: Handle<StandardMaterial>,
}

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    /// Configures window backgrounds, builds cameras/lights, and schedules visual updates.
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraState::default())
            // ClearColor: Sets the window background color (Happy Sea Mist Sky Blue)
            .insert_resource(ClearColor(Color::srgba(0.88, 0.94, 0.96, 1.0)))
            // Chains startup systems so camera coordinates register before materials load
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

// --- Systems Logic ---

/// System: Spawns the main 3D camera and configures ambient and directional lights.
fn setup_camera_and_lights(mut commands: Commands) {
    // Spawn 3D camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 12.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
    ));

    // Ambient light: Soft lighting applied to every mesh surface regardless of angle (warm sunlit gold)
    commands.insert_resource(AmbientLight {
        color: Color::srgba(1.0, 0.97, 0.92, 1.0),
        brightness: 800.0,
    });

    // Directional light: Models direct sunlight. Casts crisp shadows from mountains and spires.
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 9000.0,
            color: Color::srgba(1.0, 0.95, 0.88, 1.0),
            ..default()
        },
        transform: Transform::from_xyz(10.0, 20.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

/// System: Initializes materials into Bevy's asset database.
fn setup_materials(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Terrain Tiles
    let grass_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.65, 0.78, 0.65, 1.0), // Happy sage hill green
        perceptual_roughness: 0.9,
        ..default()
    });
    let water_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.55, 0.78, 0.85, 1.0), // Seaside aquamarine blue
        perceptual_roughness: 0.3,
        ..default()
    });
    let mountain_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.88, 0.76, 0.68, 1.0), // Warm terracotta mountain clay
        perceptual_roughness: 0.8,
        ..default()
    });
    let dc_hex_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.92, 0.86, 0.75, 1.0), // Soft beach sand
        perceptual_roughness: 0.5,
        ..default()
    });

    // Node bases (Whitewashed stucco plaster towers)
    let player_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.35, 0.55, 0.75, 1.0),
        perceptual_roughness: 0.5,
        ..default()
    });
    let ai1_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.65, 0.35, 0.28, 1.0),
        perceptual_roughness: 0.5,
        ..default()
    });
    let ai2_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.75, 0.65, 0.3, 1.0),
        perceptual_roughness: 0.5,
        ..default()
    });
    let ai3_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.55, 0.48, 0.68, 1.0),
        perceptual_roughness: 0.5,
        ..default()
    });
    let neutral_node_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.83, 0.78, 1.0),
        perceptual_roughness: 0.7,
        ..default()
    });

    // Spire tops (Accent pastel lighthouses & mission tiles)
    let player_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.55, 0.75, 0.9, 1.0),
        perceptual_roughness: 0.6,
        ..default()
    });
    let ai1_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.55, 0.45, 1.0),
        perceptual_roughness: 0.6,
        ..default()
    });
    let ai2_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.88, 0.52, 1.0),
        perceptual_roughness: 0.6,
        ..default()
    });
    let ai3_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.75, 0.68, 0.85, 1.0),
        perceptual_roughness: 0.6,
        ..default()
    });
    let neutral_node_glow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.93, 0.9, 1.0),
        perceptual_roughness: 0.8,
        ..default()
    });

    // Links (Symmetric painted paths / cable routes)
    let player_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.72, 0.9, 0.8),
        perceptual_roughness: 0.7,
        ..default()
    });
    let ai1_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.45, 0.35, 0.8),
        perceptual_roughness: 0.7,
        ..default()
    });
    let ai2_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.85, 0.45, 0.8),
        perceptual_roughness: 0.7,
        ..default()
    });
    let ai3_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.72, 0.65, 0.85, 0.8),
        perceptual_roughness: 0.7,
        ..default()
    });
    let neutral_link_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.78, 0.75, 0.7, 0.4),
        perceptual_roughness: 0.9,
        ..default()
    });

    // Packets
    let player_packet_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.6, 0.8, 0.95, 1.0),
        ..default()
    });
    let ai1_packet_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.9, 0.65, 0.58, 1.0),
        ..default()
    });
    let ai2_packet_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.98, 0.92, 0.68, 1.0),
        ..default()
    });
    let ai3_packet_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.8, 0.92, 1.0),
        ..default()
    });

    let hover_highlight_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.95, 0.6, 0.5),
        ..default()
    });
    let selected_highlight_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.85, 0.7, 0.6),
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
            transform: Transform::from_xyz(0.0, -100.0, 0.0), // Hide far below ground
            ..default()
        },
        HoverHighlightMarker,
    ));

    // Spawn selected highlight outline (initially hidden)
    commands.spawn((
        PbrBundle {
            mesh: highlight_mesh_handle,
            material: selected_highlight_mat,
            transform: Transform::from_xyz(0.0, -100.0, 0.0),
            ..default()
        },
        SelectedHighlightMarker,
    ));
}

/// System: Handles keyboard controls to orbit and zoom the camera.
fn camera_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut state: ResMut<CameraState>,
) {
    if state.radius == 0.0 {
        // Initialize position on first run
        state.radius = 18.0;
        state.azimuth = 0.8;
        state.polar = 1.0;
    }

    let speed = 0.03;
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        state.azimuth -= speed;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        state.azimuth += speed;
    }
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        state.polar = (state.polar - speed).max(0.1);
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        state.polar = (state.polar + speed).min(std::f32::consts::FRAC_PI_2 - 0.05);
    }
    if keyboard_input.pressed(KeyCode::KeyQ) {
        state.radius = (state.radius + 0.3).min(40.0);
    }
    if keyboard_input.pressed(KeyCode::KeyE) {
        state.radius = (state.radius - 0.3).max(5.0);
    }

    // Convert spherical coordinates to 3D Cartesian coordinates (X, Y, Z)
    let x = state.radius * state.polar.sin() * state.azimuth.sin();
    let z = state.radius * state.polar.sin() * state.azimuth.cos();
    let y = state.radius * state.polar.cos();

    if let Ok(mut transform) = camera_query.get_single_mut() {
        transform.translation = Vec3::new(x, y, z);
        *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
    }
}

/// System: Synchronizes hex tile models and paints them dynamically based on territory ownership.
fn sync_hex_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    tiles_query: Query<&HexTile>,
    nodes_query: Query<&NetworkNode>,
    city_query: Query<(&NetworkNode, &CityDominance)>,
    // Query filter checking for existing 3D tile models
    spawned_query: Query<(Entity, &Handle<StandardMaterial>, &Transform), With<HexTile>>,
) {
    // If tiles aren't spawned yet, instantiate their 3D models
    if spawned_query.iter().next().is_none() {
        let hex_mesh = create_hex_prism_mesh(0.95, 0.1);
        let mesh_handle = meshes.add(hex_mesh);

        for tile in tiles_query.iter() {
            let base_mat = match tile.tile_type {
                HexTileType::Grass => materials.grass_hex_mat.clone(),
                HexTileType::Water => materials.water_hex_mat.clone(),
                HexTileType::Mountain => materials.mountain_hex_mat.clone(),
                HexTileType::DataCenterCenter => materials.dc_hex_mat.clone(),
            };

            let transform = Transform::from_translation(tile.coord.to_world(0.0));
            commands.spawn((
                PbrBundle {
                    mesh: mesh_handle.clone(),
                    material: base_mat,
                    transform,
                    ..default()
                },
                tile.clone(), // Attach HexTile data component to mesh entity
            ));
        }
        return;
    }

    // Dynamic Territory Recoloring: paint tiles according to occupant ownership
    for (entity, current_mat_handle, transform) in spawned_query.iter() {
        let coord = HexCoord::from_world(transform.translation, 1.0);
        
        // Find if a node occupies this hex tile coord
        let mut occupant_owner = Owner::Neutral;
        for node in nodes_query.iter() {
            if node.coord == coord {
                occupant_owner = node.owner;
                
                // If it is a City, base recoloring on dominant team (>50% control)
                if node.node_type == NodeType::City {
                    for (c_node, dom) in city_query.iter() {
                        if c_node.coord == coord {
                            if dom.player_control_pct > 0.5 { occupant_owner = Owner::Player; }
                            else if dom.ai1_control_pct > 0.5 { occupant_owner = Owner::AI1; }
                            else if dom.ai2_control_pct > 0.5 { occupant_owner = Owner::AI2; }
                            else if dom.ai3_control_pct > 0.5 { occupant_owner = Owner::AI3; }
                        }
                    }
                }
            }
        }

        // Determine target pastel terrain color
        let target_mat = match occupant_owner {
            Owner::Player => materials.player_link_mat.clone(), // Cornflower blue wash
            Owner::AI1 => materials.ai1_link_mat.clone(),       // Terracotta wash
            Owner::AI2 => materials.ai2_link_mat.clone(),       // Lemon yellow wash
            Owner::AI3 => materials.ai3_link_mat.clone(),       // Lavender wash
            Owner::Neutral => {
                // Keep base terrain color
                materials.grass_hex_mat.clone()
            }
        };

        // Swap material handle if it changed to trigger GPU update
        if *current_mat_handle != target_mat {
            commands.entity(entity).insert(target_mat);
        }
    }
}

/// System: Spawns and morphs node visuals (Mission bell towers, plaster spires).
fn sync_node_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    nodes_query: Query<(Entity, &NetworkNode, &Transform, Option<&CityDominance>)>,
    mesh_query: Query<(Entity, &NodeMeshMarker)>,
) {
    let mut active_nodes = std::collections::HashSet::new();
    for (node_entity, _, _, _) in nodes_query.iter() {
        active_nodes.insert(node_entity);
    }

    // Clean up mesh structures for despawned/bought out nodes
    for (entity, marker) in mesh_query.iter() {
        if !active_nodes.contains(&marker.node_entity) {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Re-spawn or morph materials to match owners
    for (node_entity, node, transform, city_dom) in nodes_query.iter() {
        let has_mesh = mesh_query.iter().any(|(_, marker)| marker.node_entity == node_entity);
        if has_mesh {
            // Re-spawn meshes only if owner changes to keep spires synchronized
            continue;
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
                    // Small Mission bell-tower (bell structure)
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
                    // Giant Cluster structures representing Main Mission Hubs
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.4, 1.2, 0.4))),
                        material: base_mat.clone(),
                        transform: Transform::from_xyz(-0.2, 0.6, 0.0),
                        ..default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.35, 1.5, 0.35))),
                        material: base_mat.clone(),
                        transform: Transform::from_xyz(0.2, 0.75, 0.1),
                        ..default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.3, 0.8, 0.3))),
                        material: glow_mat.clone(),
                        transform: Transform::from_xyz(0.0, 1.0, -0.2),
                        ..default()
                    });
                }
                NodeType::City => {
                    // Quaint white-washed tower blocks representing towns/cities
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.6, 0.7, 0.6))),
                        material: base_mat.clone(),
                        transform: Transform::from_xyz(0.0, 0.35, 0.0),
                        ..default()
                    });
                    parent.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.4, 1.1, 0.4))),
                        material: glow_mat.clone(),
                        transform: Transform::from_xyz(0.0, 0.55, 0.0),
                        ..default()
                    });
                }
            }
        });
    }
}

/// System: Renders wire link cables between connected nodes.
fn sync_link_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    links_query: Query<(Entity, &NetworkLink)>,
    nodes_query: Query<(&NetworkNode, &Transform)>,
    mesh_query: Query<(Entity, &LinkMeshMarker)>,
) {
    let mut active_links = std::collections::HashSet::new();
    for (entity, _) in links_query.iter() {
        active_links.insert(entity);
    }

    // Despawn severed visual wires
    for (entity, marker) in mesh_query.iter() {
        if !active_links.contains(&marker.link_entity) {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Spawn 3D tube wires for active connections
    for (link_entity, link) in links_query.iter() {
        let has_mesh = mesh_query.iter().any(|(_, marker)| marker.link_entity == link_entity);
        if has_mesh {
            continue;
        }

        if let (Ok((node_a, transform_a)), Ok((node_b, transform_b))) = 
            (nodes_query.get(link.node_a), nodes_query.get(link.node_b)) {
            
            let pos_a = transform_a.translation + Vec3::Y * 0.15;
            let pos_b = transform_b.translation + Vec3::Y * 0.15;
            let dir = pos_b - pos_a;
            let dist = dir.length();
            let midpoint = pos_a + dir * 0.5;

            // Align wire tube along the axis between coordinates
            let rotation = Quat::from_rotation_arc(Vec3::Y, dir.normalize());

            // Color wires by owner's color to show link occupancy
            let wire_mat = match node_a.owner {
                Owner::Player => materials.player_link_mat.clone(),
                Owner::AI1 => materials.ai1_link_mat.clone(),
                Owner::AI2 => materials.ai2_link_mat.clone(),
                Owner::AI3 => materials.ai3_link_mat.clone(),
                Owner::Neutral => {
                    // Match owner of node_b if node_a is neutral
                    match node_b.owner {
                        Owner::Player => materials.player_link_mat.clone(),
                        Owner::AI1 => materials.ai1_link_mat.clone(),
                        Owner::AI2 => materials.ai2_link_mat.clone(),
                        Owner::AI3 => materials.ai3_link_mat.clone(),
                        _ => materials.neutral_link_mat.clone(),
                    }
                }
            };

            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(Cuboid::new(0.08, dist, 0.08))),
                    material: wire_mat,
                    transform: Transform::from_translation(midpoint).with_rotation(rotation),
                    ..default()
                },
                LinkMeshMarker { link_entity },
            ));
        }
    }
}

/// System: Draws traveling packet boxes along links.
fn sync_packet_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    packets_query: Query<(Entity, &Packet)>,
    nodes_query: Query<&Transform, With<NetworkNode>>,
    mesh_query: Query<(Entity, &PacketMeshMarker)>,
) {
    let mut active_packets = std::collections::HashSet::new();
    for (entity, _) in packets_query.iter() {
        active_packets.insert(entity);
    }

    // Despawn visual packets once they reach target or get dropped
    for (entity, marker) in mesh_query.iter() {
        if !active_packets.contains(&marker.packet_entity) {
            commands.entity(entity).despawn();
        }
    }

    // Update positions of active packets
    for (packet_entity, packet) in packets_query.iter() {
        if let (Ok(transform_a), Ok(transform_b)) = 
            (nodes_query.get(packet.from_node), nodes_query.get(packet.to_node)) {
            
            let pos_a = transform_a.translation + Vec3::Y * 0.2;
            let pos_b = transform_b.translation + Vec3::Y * 0.2;
            
            // Linear interpolation (lerp) progress between nodes
            let current_pos = pos_a.lerp(pos_b, packet.progress);

            let has_mesh = mesh_query.iter().any(|(_, marker)| marker.packet_entity == packet_entity);
            if has_mesh {
                // Find existing packet entity and update its physical translation
                for (mesh_entity, marker) in mesh_query.iter() {
                    if marker.packet_entity == packet_entity {
                        commands.entity(mesh_entity).insert(Transform::from_translation(current_pos));
                    }
                }
            } else {
                // Determine pastel color based on sender IP
                let packet_mat = if packet.src_ip < 100 {
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
                        mesh: meshes.add(Mesh::from(Cuboid::new(0.12, 0.12, 0.12))),
                        material: packet_mat,
                        transform: Transform::from_translation(current_pos),
                        ..default()
                    },
                    PacketMeshMarker { packet_entity },
                ));
            }
        }
    }
}

/// System: Updates the visual outline rings showing hovered or selected hex coordinates.
fn update_highlights(
    player_controls: Res<crate::hud::PlayerControls>,
    mut hover_query: Query<&mut Transform, (With<HoverHighlightMarker>, Without<SelectedHighlightMarker>)>,
    mut selected_query: Query<&mut Transform, (With<SelectedHighlightMarker>, Without<HoverHighlightMarker>)>,
) {
    // Positioning hover outline
    if let Ok(mut transform) = hover_query.get_single_mut() {
        if let Some(coord) = player_controls.hovered_hex {
            transform.translation = coord.to_world(0.06); // Hover slightly above ground
        } else {
            transform.translation = Vec3::new(0.0, -100.0, 0.0); // Hide
        }
    }

    // Positioning selection outline
    if let Ok(mut transform) = selected_query.get_single_mut() {
        if let Some(coord) = player_controls.selected_node_coord {
            transform.translation = coord.to_world(0.07);
        } else {
            transform.translation = Vec3::new(0.0, -100.0, 0.0);
        }
    }
}

/// System: Spawns a translucent preview cable when laying wires.
fn update_link_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GameMaterials>,
    player_controls: Res<crate::hud::PlayerControls>,
    nodes_query: Query<(&NetworkNode, &Transform)>,
    preview_query: Query<Entity, With<LinkPreviewMarker>>,
) {
    // Despawn previous frame's preview line
    for entity in preview_query.iter() {
        commands.entity(entity).despawn();
    }

    // Draw preview line only if wirelaying tool is active
    if player_controls.selected_tool == crate::hud::SelectedTool::LayWire {
        if let (Some(selected_entity), Some(hovered_coord)) = 
            (player_controls.selected_node, player_controls.hovered_hex) {
            
            if let Ok((selected_node, selected_transform)) = nodes_query.get(selected_entity) {
                // Enforce laying adjacent to selected node (distance limit = 1 hex)
                if selected_node.coord.distance(&hovered_coord) == 1 {
                    let pos_a = selected_transform.translation + Vec3::Y * 0.15;
                    let pos_b = hovered_coord.to_world(0.15);
                    let dir = pos_b - pos_a;
                    let dist = dir.length();
                    let midpoint = pos_a + dir * 0.5;
                    let rotation = Quat::from_rotation_arc(Vec3::Y, dir.normalize());

                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(Cuboid::new(0.04, dist, 0.04))),
                            material: materials.selected_highlight_mat.clone(), // Translucent green outline
                            transform: Transform::from_translation(midpoint).with_rotation(rotation),
                            ..default()
                        },
                        LinkPreviewMarker,
                    ));
                }
            }
        }
    }
}
