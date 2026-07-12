use bevy::prelude::*;
use bevy::render::mesh::{Mesh, Indices};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;

// =========================================================================
// PACKETCOMMAND HEXAGON MATHEMATICS
// =========================================================================
// This file defines the axial coordinate system (q, r) for pointy-topped 3D hexes,
// coordinate conversion helpers, and manually constructs a 3D hexagonal prism mesh.

/// ECS Component: Represents a single hex tile on the playing field.
#[derive(Component, Debug, Clone, Copy)]
pub struct HexTile {
    pub coord: HexCoord,
    pub tile_type: HexTileType,
}

/// The base terrain category of a hex tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexTileType {
    Grass,            // Sage Green hills
    Water,            // Sea Mist aquamarine water
    Mountain,         // Terracotta clay ridges
    DataCenterCenter, // Warm beach sand hub
}

/// A 2D axial coordinate mapping for a hexagonal grid.
///
/// In hex layouts, instead of using Cartesian (x, y) grid steps, we use axial axes (q, r):
///   - Axis `q` runs along diagonal columns.
///   - Axis `r` runs along horizontal rows.
///
/// The third axis `s` is defined implicitly as `-q -r`, satisfying `q + r + s = 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, serde::Serialize, serde::Deserialize)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    /// Creates a new hex coordinate pair.
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Converts axial hex coordinates (q, r) to 3D Cartesian coordinates (X, Y, Z).
    ///
    /// For a pointy-topped hex layout:
    ///   - Center-to-center horizontal spacing (X direction) is `radius * sqrt(3)`.
    ///   - Vertical spacing (Z direction) is `radius * 3/2`.
    pub fn to_world(&self, radius: f32) -> Vec3 {
        let x = radius * (1.7320508 * self.q as f32 + 0.8660254 * self.r as f32);
        let z = radius * 1.5 * self.r as f32;
        Vec3::new(x, 0.0, z)
    }

    /// Computes the distance (minimum number of steps) between two hex coordinates.
    #[allow(dead_code)]
    pub fn distance(&self, other: &Self) -> i32 {
        ((self.q - other.q).abs() 
            + (self.q + self.r - other.q - other.r).abs() 
            + (self.r - other.r).abs()) / 2
    }

    /// Checks if a coordinate is within a radius-3 board (for unit testing validation).
    pub fn is_on_board(&self) -> bool {
        self.q.abs() <= 2 && self.r.abs() <= 2 && (self.q + self.r).abs() <= 2
    }

    /// Converts 3D Cartesian world coordinates back to axial integer (q, r) coordinates.
    pub fn from_world(world: Vec3, radius: f32) -> Self {
        let q_float = (0.577350269 * world.x - 0.333333333 * world.z) / radius;
        let r_float = (0.666666667 * world.z) / radius;
        Self::round(q_float, r_float)
    }

    /// Rounds floating-point hex coordinates to the nearest integer axial coordinate.
    ///
    /// Since raw coordinates can result in values like (1.4, -0.6), we round them
    /// to integers while preserving the constraint `q + r + s = 0`.
    fn round(q: f32, r: f32) -> Self {
        let s = -q - r;
        let mut rq = q.round();
        let mut rr = r.round();
        let rs = s.round();

        let dq = (rq - q).abs();
        let dr = (rr - r).abs();
        let ds = (rs - s).abs();

        // Adjust the axis that deviated the most to maintain the constraint
        if dq > dr && dq > ds {
            rq = -rr - rs;
        } else if dr > ds {
            rr = -rq - rs;
        }

        Self {
            q: rq as i32,
            r: rr as i32,
        }
    }
}

/// Generates a 3D pointy-topped hexagonal prism mesh structure.
///
/// In 3D rendering, a Mesh consists of:
///   - Positions: 3D vertices [x, y, z] in local coordinate space.
///   - Normals: 3D direction vectors showing which way each vertex faces (used for lighting/shading).
///   - UVs: 2D texture coordinates [u, v] mapping flat textures onto faces.
///   - Indices: Triangles specified by three vertex index positions (Winding order controls face culling).
pub fn create_hex_prism_mesh(radius: f32, height: f32) -> Mesh {
    let half_h = height / 2.0;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // 1. Generate local perimeter coordinates for a pointy-topped hexagon (6 vertices)
    let mut hex_vertices = Vec::new();
    for i in 0..6 {
        // Pointy-topped hexes start offsets at 30 degrees (FRAC_PI_6) and increment by 60 degrees (PI / 3)
        let angle = std::f32::consts::FRAC_PI_6 + (i as f32 * std::f32::consts::PI / 3.0);
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        hex_vertices.push(Vec3::new(x, 0.0, z));
    }

    // 2. BUILD THE TOP FACE (Triangle Fan)
    let top_center_idx = positions.len() as u32;
    positions.push([0.0, half_h, 0.0]); // Center vertex
    normals.push([0.0, 1.0, 0.0]);      // Facing straight up
    uvs.push([0.5, 0.5]);

    let top_start_idx = positions.len() as u32;
    for v in &hex_vertices {
        positions.push([v.x, half_h, v.z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([(v.x / radius + 1.0) * 0.5, (v.z / radius + 1.0) * 0.5]);
    }

    // Stitch triangles from center to perimeter vertices (clockwise winding order)
    for i in 0..6 {
        indices.push(top_center_idx);
        indices.push(top_start_idx + i);
        indices.push(top_start_idx + (i + 1) % 6);
    }

    // 3. BUILD THE BOTTOM FACE (Triangle Fan, counter-clockwise winding order)
    let bot_center_idx = positions.len() as u32;
    positions.push([0.0, -half_h, 0.0]); // Center vertex
    normals.push([0.0, -1.0, 0.0]);      // Facing straight down
    uvs.push([0.5, 0.5]);

    let bot_start_idx = positions.len() as u32;
    for v in &hex_vertices {
        positions.push([v.x, -half_h, v.z]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push([(v.x / radius + 1.0) * 0.5, (v.z / radius + 1.0) * 0.5]);
    }

    for i in 0..6 {
        indices.push(bot_center_idx);
        indices.push(bot_start_idx + (i + 1) % 6);
        indices.push(bot_start_idx + i);
    }

    // 4. BUILD THE SIDE FACES (Flat-shaded rectangular strips)
    for i in 0..6 {
        let i_next = (i + 1) % 6;
        let v1_top = hex_vertices[i] + Vec3::new(0.0, half_h, 0.0);
        let v2_top = hex_vertices[i_next] + Vec3::new(0.0, half_h, 0.0);
        let v1_bot = hex_vertices[i] - Vec3::new(0.0, half_h, 0.0);
        let v2_bot = hex_vertices[i_next] - Vec3::new(0.0, half_h, 0.0);

        // Compute normal vector for this flat side face using cross product
        let edge_top = v2_top - v1_top;
        let edge_side = v1_bot - v1_top;
        let n = edge_top.cross(edge_side).normalize();

        let start_idx = positions.len() as u32;
        positions.push([v1_top.x, v1_top.y, v1_top.z]);
        positions.push([v2_top.x, v2_top.y, v2_top.z]);
        positions.push([v2_bot.x, v2_bot.y, v2_bot.z]);
        positions.push([v1_bot.x, v1_bot.y, v1_bot.z]);

        for _ in 0..4 {
            normals.push([n.x, n.y, n.z]);
        }

        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 0.0]);
        uvs.push([1.0, 1.0]);
        uvs.push([0.0, 1.0]);

        // Triangle 1
        indices.push(start_idx);
        indices.push(start_idx + 1);
        indices.push(start_idx + 2);
        // Triangle 2
        indices.push(start_idx);
        indices.push(start_idx + 2);
        // Winding order matters!
        indices.push(start_idx + 3);
    }

    // Insert attributes into Bevy's Mesh resource
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

// -------------------------------------------------------------------------
// UNIT TESTS (Verify coordinate calculations)
// -------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_distance() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        assert_eq!(a.distance(&b), 1);

        let c = HexCoord::new(2, -2);
        assert_eq!(a.distance(&c), 2);

        let d = HexCoord::new(-1, -1);
        assert_eq!(a.distance(&d), 2);
    }

    #[test]
    fn test_hex_to_world() {
        let a = HexCoord::new(0, 0);
        let pos = a.to_world(1.0);
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.z, 0.0);
    }

    #[test]
    fn test_hex_from_world() {
        let radius = 1.0;
        let coord = HexCoord::new(1, -1);
        let pos = coord.to_world(radius);
        let rounded = HexCoord::from_world(pos, radius);
        assert_eq!(rounded, coord);
    }
}
