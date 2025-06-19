use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;
use std::f32::consts::PI;

pub struct ModelsPlugin;

impl Plugin for ModelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_custom_meshes);
    }
}

fn setup_custom_meshes(_meshes: ResMut<Assets<Mesh>>) {
    // Register custom meshes here if needed
}

pub fn create_detailed_aircraft_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    // Fuselage parameters
    let fuselage_length = 4.0;
    let fuselage_radius = 0.5;
    let nose_length = 1.5;
    let tail_length = 1.0;
    let segments: usize = 16;
    
    // Generate fuselage
    let total_length = fuselage_length + nose_length + tail_length;
    let mut vertex_offset: usize = 0;
    
    // Create fuselage sections
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let z = -total_length / 2.0 + t * total_length;
        
        // Vary radius based on position
        let radius = if z < -fuselage_length / 2.0 {
            // Nose section - taper to point
            let nose_t = (z + total_length / 2.0) / nose_length;
            fuselage_radius * (1.0 - nose_t * nose_t).max(0.1)
        } else if z > fuselage_length / 2.0 {
            // Tail section - slight taper
            let tail_t = (z - fuselage_length / 2.0) / tail_length;
            fuselage_radius * (1.0 - tail_t * 0.5)
        } else {
            // Main fuselage
            fuselage_radius
        };
        
        // Create ring of vertices
        for j in 0..segments {
            let angle = j as f32 * 2.0 * PI / segments as f32;
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            
            positions.push([x, y, z]);
            normals.push([angle.cos(), angle.sin(), 0.0]);
            uvs.push([j as f32 / segments as f32, t]);
        }
        
        // Create triangles between rings
        if i > 0 {
            for j in 0..segments {
                let current = vertex_offset + j;
                let next = vertex_offset + (j + 1) % segments;
                let prev_ring = current - segments;
                let prev_next = prev_ring + 1;
                
                if prev_next % segments == 0 {
                    indices.push(prev_ring as u32);
                    indices.push(current as u32);
                    indices.push(vertex_offset as u32);
                    
                    indices.push(prev_ring as u32);
                    indices.push(vertex_offset as u32);
                    indices.push((vertex_offset - segments) as u32);
                } else {
                    indices.push(prev_ring as u32);
                    indices.push(current as u32);
                    indices.push(next as u32);
                    
                    indices.push(prev_ring as u32);
                    indices.push(next as u32);
                    indices.push(prev_next as u32);
                }
            }
        }
        
        vertex_offset += segments;
    }
    
    // Add wing geometry
    let wing_span = 6.0;
    let wing_chord = 2.0;
    let wing_thickness = 0.15;
    let wing_sweep = 0.3;
    
    // Right wing
    add_wing_to_mesh(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        &mut vertex_offset,
        wing_span / 2.0,
        wing_chord,
        wing_thickness,
        wing_sweep,
        false,
    );
    
    // Left wing
    add_wing_to_mesh(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        &mut vertex_offset,
        wing_span / 2.0,
        wing_chord,
        wing_thickness,
        wing_sweep,
        true,
    );
    
    // Add tail surfaces
    add_tail_surfaces(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        &mut vertex_offset,
    );
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

fn add_wing_to_mesh(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    vertex_offset: &mut usize,
    span: f32,
    chord: f32,
    thickness: f32,
    sweep: f32,
    is_left: bool,
) {
    let direction = if is_left { -1.0 } else { 1.0 };
    let base_offset = *vertex_offset as u32;
    
    // Airfoil shape points (simplified NACA profile)
    let airfoil_points = vec![
        (0.0, 0.0),           // Leading edge
        (0.05, 0.03),         // Upper front
        (0.25, 0.05),         // Upper mid
        (0.7, 0.03),          // Upper rear
        (1.0, 0.0),           // Trailing edge
        (0.7, -0.02),         // Lower rear
        (0.25, -0.03),        // Lower mid
        (0.05, -0.02),        // Lower front
    ];
    
    // Wing sections
    for i in 0..5 {
        let t = i as f32 / 4.0;
        let x = direction * span * t;
        let z_offset = -sweep * t * chord;
        let taper = 1.0 - t * 0.4; // Taper to 60% at tip
        
        // Add vertices for this section
        for (chord_pos, y_profile) in &airfoil_points {
            let z = z_offset - chord_pos * chord * taper;
            let y = y_profile * thickness * taper;
            
            positions.push([x, y, z]);
            normals.push([0.0, y.signum(), 0.0]);
            uvs.push([t, *chord_pos]);
        }
        
        // Connect sections with triangles
        if i > 0 {
            let prev_section = base_offset + ((i - 1) * 8) as u32;
            let curr_section = base_offset + (i * 8) as u32;
            
            // Top surface
            for j in 0..4 {
                indices.push(prev_section + j);
                indices.push(curr_section + j);
                indices.push(curr_section + j + 1);
                
                indices.push(prev_section + j);
                indices.push(curr_section + j + 1);
                indices.push(prev_section + j + 1);
            }
            
            // Bottom surface
            for j in 4..7 {
                indices.push(prev_section + j);
                indices.push(curr_section + j + 1);
                indices.push(curr_section + j);
                
                indices.push(prev_section + j);
                indices.push(prev_section + j + 1);
                indices.push(curr_section + j + 1);
            }
            
            // Close the loop
            indices.push(prev_section + 7);
            indices.push(curr_section);
            indices.push(curr_section + 7);
            indices.push(prev_section + 7);
            indices.push(prev_section);
            indices.push(curr_section);
        }
    }
    
    *vertex_offset += 5 * 8;
}

fn add_tail_surfaces(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    vertex_offset: &mut usize,
) {
    let base_offset = *vertex_offset as u32;
    
    // Vertical stabilizer
    let vert_height = 1.5;
    let vert_chord = 1.0;
    let vert_sweep = 0.4;
    let vert_z_pos = 2.0;
    
    // Vertical stabilizer vertices
    positions.push([0.0, 0.0, vert_z_pos]); // Base front
    positions.push([0.0, 0.0, vert_z_pos + vert_chord]); // Base rear
    positions.push([0.0, vert_height, vert_z_pos + vert_sweep]); // Top front
    positions.push([0.0, vert_height, vert_z_pos + vert_chord]); // Top rear
    
    for _ in 0..4 {
        normals.push([1.0, 0.0, 0.0]);
        uvs.push([0.0, 0.0]);
    }
    
    // Vertical stabilizer triangles
    indices.push(base_offset);
    indices.push(base_offset + 2);
    indices.push(base_offset + 3);
    indices.push(base_offset);
    indices.push(base_offset + 3);
    indices.push(base_offset + 1);
    
    *vertex_offset += 4;
    
    // Horizontal stabilizer
    let horiz_span = 3.0;
    let horiz_chord = 0.8;
    let horiz_z_pos = 2.2;
    let horiz_y_pos = 0.3;
    
    let horiz_base = *vertex_offset as u32;
    
    // Right side
    positions.push([0.0, horiz_y_pos, horiz_z_pos]);
    positions.push([horiz_span / 2.0, horiz_y_pos, horiz_z_pos + 0.2]);
    positions.push([horiz_span / 2.0, horiz_y_pos, horiz_z_pos + horiz_chord]);
    positions.push([0.0, horiz_y_pos, horiz_z_pos + horiz_chord]);
    
    // Left side
    positions.push([-horiz_span / 2.0, horiz_y_pos, horiz_z_pos + 0.2]);
    positions.push([-horiz_span / 2.0, horiz_y_pos, horiz_z_pos + horiz_chord]);
    
    for _ in 0..6 {
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.0, 0.0]);
    }
    
    // Right side triangles
    indices.push(horiz_base);
    indices.push(horiz_base + 1);
    indices.push(horiz_base + 2);
    indices.push(horiz_base);
    indices.push(horiz_base + 2);
    indices.push(horiz_base + 3);
    
    // Left side triangles
    indices.push(horiz_base);
    indices.push(horiz_base + 4);
    indices.push(horiz_base);
    indices.push(horiz_base + 5);
    indices.push(horiz_base + 4);
    indices.push(horiz_base + 3);
    indices.push(horiz_base + 5);
    indices.push(horiz_base + 3);
    
    *vertex_offset += 6;
}

pub fn create_terrain_chunk(
    size: f32,
    resolution: u32,
    height_fn: impl Fn(f32, f32) -> f32,
) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    let step = size / resolution as f32;
    
    // Generate vertices
    for i in 0..=resolution {
        for j in 0..=resolution {
            let x = -size / 2.0 + i as f32 * step;
            let z = -size / 2.0 + j as f32 * step;
            let y = height_fn(x, z);
            
            positions.push([x, y, z]);
            uvs.push([i as f32 / resolution as f32, j as f32 / resolution as f32]);
            
            // Calculate normal using neighboring points
            let dx = height_fn(x + step, z) - height_fn(x - step, z);
            let dz = height_fn(x, z + step) - height_fn(x, z - step);
            let normal = Vec3::new(-dx, 2.0 * step, -dz).normalize();
            normals.push(normal.to_array());
        }
    }
    
    // Generate indices
    for i in 0..resolution {
        for j in 0..resolution {
            let idx = i * (resolution + 1) + j;
            
            // First triangle
            indices.push(idx);
            indices.push(idx + resolution + 1);
            indices.push(idx + 1);
            
            // Second triangle
            indices.push(idx + 1);
            indices.push(idx + resolution + 1);
            indices.push(idx + resolution + 2);
        }
    }
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

pub fn create_tree_mesh(seed: u32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    // Use seed for variation
    let height_variation = ((seed % 10) as f32 / 10.0) * 10.0;
    let trunk_height = 15.0 + height_variation;
    let trunk_radius = 1.0 + ((seed % 5) as f32 / 10.0);
    
    // Create trunk with tapering
    let trunk_segments: usize = 8;
    let height_segments: usize = 6;
    let mut vertex_offset: usize = 0;
    
    for h in 0..=height_segments {
        let height_t = h as f32 / height_segments as f32;
        let y = height_t * trunk_height;
        let radius = trunk_radius * (1.0 - height_t * 0.3); // Taper by 30%
        
        for i in 0..trunk_segments {
            let angle = i as f32 * 2.0 * PI / trunk_segments as f32;
            let x = radius * angle.cos();
            let z = radius * angle.sin();
            
            positions.push([x, y, z]);
            normals.push([angle.cos(), 0.0, angle.sin()]);
            uvs.push([i as f32 / trunk_segments as f32, height_t]);
        }
        
        if h > 0 {
            for i in 0..trunk_segments {
                let current = vertex_offset + i;
                let next = vertex_offset + (i + 1) % trunk_segments;
                let prev_ring = current - trunk_segments;
                let prev_next = prev_ring + 1;
                
                if (i + 1) % trunk_segments == 0 {
                    indices.push(prev_ring as u32);
                    indices.push(current as u32);
                    indices.push(vertex_offset as u32);
                    
                    indices.push(prev_ring as u32);
                    indices.push(vertex_offset as u32);
                    indices.push((vertex_offset - trunk_segments) as u32);
                } else {
                    indices.push(prev_ring as u32);
                    indices.push(current as u32);
                    indices.push(next as u32);
                    
                    indices.push(prev_ring as u32);
                    indices.push(next as u32);
                    indices.push(prev_next as u32);
                }
            }
        }
        
        vertex_offset += trunk_segments;
    }
    
    // Add branches
    let num_branches = 3 + (seed % 3) as usize;
    for b in 0..num_branches {
        let branch_height = trunk_height * (0.5 + (b as f32 / num_branches as f32) * 0.4);
        let branch_angle = b as f32 * 2.0 * PI / num_branches as f32 + (seed as f32 * 0.1);
        let branch_length = 3.0 + ((seed + b as u32) % 5) as f32;
        
        add_branch_to_tree(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            &mut vertex_offset,
            branch_height,
            branch_angle,
            branch_length,
        );
    }
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

fn add_branch_to_tree(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    vertex_offset: &mut usize,
    height: f32,
    angle: f32,
    length: f32,
) {
    let base_offset = *vertex_offset as u32;
    let segments: usize = 4;
    
    // Branch direction
    let dir_x = angle.cos();
    let dir_z = angle.sin();
    
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let radius = 0.3 * (1.0 - t * 0.7); // Taper from 0.3 to 0.09
        
        // Branch curves upward
        let bend = t * t * 0.3;
        let x = dir_x * length * t;
        let y = height + bend * length;
        let z = dir_z * length * t;
        
        // Create a simple square cross-section for branches
        positions.push([x - radius, y, z]);
        positions.push([x + radius, y, z]);
        positions.push([x, y - radius, z]);
        positions.push([x, y + radius, z]);
        
        for _ in 0..4 {
            normals.push([dir_x, bend, dir_z]);
            uvs.push([t, 0.0]);
        }
        
        if i > 0 {
            let prev = base_offset + ((i - 1) * 4) as u32;
            let curr = base_offset + (i * 4) as u32;
            
            // Connect the segments
            for j in 0..4 {
                let next_j = (j + 1) % 4;
                indices.push(prev + j);
                indices.push(curr + j);
                indices.push(curr + next_j);
                
                indices.push(prev + j);
                indices.push(curr + next_j);
                indices.push(prev + next_j);
            }
        }
    }
    
    *vertex_offset += (segments + 1) * 4;
}

pub fn create_enemy_fighter_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    // Fighter has a sleeker, more aggressive design
    let fuselage_length = 3.0;
    let fuselage_radius = 0.4;
    let segments: usize = 12;
    
    // Fuselage
    let mut vertex_offset: usize = 0;
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let z = -fuselage_length / 2.0 + t * fuselage_length;
        
        // Sharp nose, wider middle
        let radius = if t < 0.3 {
            fuselage_radius * (t * 3.0).min(1.0)
        } else if t > 0.8 {
            fuselage_radius * (1.0 - (t - 0.8) * 3.0)
        } else {
            fuselage_radius
        };
        
        for j in 0..segments {
            let angle = j as f32 * 2.0 * PI / segments as f32;
            positions.push([radius * angle.cos(), radius * angle.sin(), z]);
            normals.push([angle.cos(), angle.sin(), 0.0]);
            uvs.push([j as f32 / segments as f32, t]);
        }
        
        if i > 0 {
            for j in 0..segments {
                let current = vertex_offset + j;
                let next = vertex_offset + (j + 1) % segments;
                let prev_ring = current - segments;
                let prev_next = prev_ring + 1;
                
                if (j + 1) % segments == 0 {
                    indices.extend_from_slice(&[
                        prev_ring as u32, current as u32, vertex_offset as u32,
                        prev_ring as u32, vertex_offset as u32, (vertex_offset - segments) as u32,
                    ]);
                } else {
                    indices.extend_from_slice(&[
                        prev_ring as u32, current as u32, next as u32,
                        prev_ring as u32, next as u32, prev_next as u32,
                    ]);
                }
            }
        }
        vertex_offset += segments;
    }
    
    // Delta wings
    add_delta_wings(&mut positions, &mut normals, &mut uvs, &mut indices, &mut vertex_offset);
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

pub fn create_enemy_bomber_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    // Bomber has a larger, bulkier design
    let fuselage_length = 5.0;
    let fuselage_radius = 0.8;
    let segments: usize = 16;
    
    // Bulky fuselage
    let mut vertex_offset: usize = 0;
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let z = -fuselage_length / 2.0 + t * fuselage_length;
        
        // Rounded nose and tail
        let radius = fuselage_radius * (1.0 - 4.0 * (t - 0.5).powi(2)).max(0.3);
        
        for j in 0..segments {
            let angle = j as f32 * 2.0 * PI / segments as f32;
            positions.push([radius * angle.cos(), radius * angle.sin(), z]);
            normals.push([angle.cos(), angle.sin(), 0.0]);
            uvs.push([j as f32 / segments as f32, t]);
        }
        
        if i > 0 {
            for j in 0..segments {
                let current = vertex_offset + j;
                let next = vertex_offset + (j + 1) % segments;
                let prev_ring = current - segments;
                let prev_next = prev_ring + 1;
                
                if (j + 1) % segments == 0 {
                    indices.extend_from_slice(&[
                        prev_ring as u32, current as u32, vertex_offset as u32,
                        prev_ring as u32, vertex_offset as u32, (vertex_offset - segments) as u32,
                    ]);
                } else {
                    indices.extend_from_slice(&[
                        prev_ring as u32, current as u32, next as u32,
                        prev_ring as u32, next as u32, prev_next as u32,
                    ]);
                }
            }
        }
        vertex_offset += segments;
    }
    
    // Straight wings for bomber
    add_bomber_wings(&mut positions, &mut normals, &mut uvs, &mut indices, &mut vertex_offset);
    
    // Engine pods
    add_engine_pods(&mut positions, &mut normals, &mut uvs, &mut indices, &mut vertex_offset);
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

pub fn create_enemy_ace_mesh() -> Mesh {
    // Similar to fighter but with more advanced features
    let mesh = create_enemy_fighter_mesh();
    
    // Add canards and other advanced features
    // (Implementation details would go here)
    
    mesh
}

fn add_delta_wings(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    vertex_offset: &mut usize,
) {
    let base_offset = *vertex_offset as u32;
    
    // Delta wing shape
    let wing_points = vec![
        // Right wing
        [0.0, 0.0, 0.5],      // Root front
        [3.0, 0.0, 1.0],      // Tip
        [0.0, 0.0, -1.5],     // Root back
        // Left wing
        [0.0, 0.0, 0.5],      // Root front
        [-3.0, 0.0, 1.0],     // Tip
        [0.0, 0.0, -1.5],     // Root back
    ];
    
    for point in &wing_points {
        positions.push(*point);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.0, 0.0]);
    }
    
    // Right wing triangle
    indices.extend_from_slice(&[base_offset, base_offset + 1, base_offset + 2]);
    // Left wing triangle
    indices.extend_from_slice(&[base_offset + 3, base_offset + 4, base_offset + 5]);
    
    *vertex_offset += 6;
}

fn add_bomber_wings(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    vertex_offset: &mut usize,
) {
    let base_offset = *vertex_offset as u32;
    
    // Straight wings with more area
    let wing_thickness = 0.2;
    
    // Right wing
    positions.extend_from_slice(&[
        [0.5, wing_thickness/2.0, 0.0],
        [4.0, wing_thickness/2.0, 0.5],
        [4.0, wing_thickness/2.0, -1.5],
        [0.5, wing_thickness/2.0, -1.0],
        [0.5, -wing_thickness/2.0, 0.0],
        [4.0, -wing_thickness/2.0, 0.5],
        [4.0, -wing_thickness/2.0, -1.5],
        [0.5, -wing_thickness/2.0, -1.0],
    ]);
    
    for _ in 0..8 {
        normals.push([0.0, 0.0, 0.0]); // Will be calculated
        uvs.push([0.0, 0.0]);
    }
    
    // Top face
    indices.extend_from_slice(&[
        base_offset, base_offset + 1, base_offset + 2,
        base_offset, base_offset + 2, base_offset + 3,
    ]);
    
    // Bottom face
    indices.extend_from_slice(&[
        base_offset + 4, base_offset + 6, base_offset + 5,
        base_offset + 4, base_offset + 7, base_offset + 6,
    ]);
    
    *vertex_offset += 8;
    
    // Mirror for left wing
    let left_offset = *vertex_offset as u32;
    for i in 0..8 {
        let mut pos = positions[base_offset as usize + i];
        pos[0] = -pos[0];
        positions.push(pos);
        normals.push([0.0, 0.0, 0.0]);
        uvs.push([0.0, 0.0]);
    }
    
    // Left wing faces
    indices.extend_from_slice(&[
        left_offset, left_offset + 2, left_offset + 1,
        left_offset, left_offset + 3, left_offset + 2,
        left_offset + 4, left_offset + 5, left_offset + 6,
        left_offset + 4, left_offset + 6, left_offset + 7,
    ]);
    
    *vertex_offset += 8;
}

fn add_engine_pods(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    vertex_offset: &mut usize,
) {
    // Add cylindrical engine pods under wings
    let pod_positions = vec![
        [2.0, -0.3, -0.5],   // Right engine
        [-2.0, -0.3, -0.5],  // Left engine
    ];
    
    for pod_pos in pod_positions {
        let base_offset = *vertex_offset as u32;
        let segments: usize = 8;
        let length = 2.0;
        let radius = 0.3;
        
        for i in 0..=2 {
            let z = pod_pos[2] - length / 2.0 + (i as f32 / 2.0) * length;
            for j in 0..segments {
                let angle = j as f32 * 2.0 * PI / segments as f32;
                positions.push([
                    pod_pos[0] + radius * angle.cos(),
                    pod_pos[1] + radius * angle.sin(),
                    z,
                ]);
                normals.push([angle.cos(), angle.sin(), 0.0]);
                uvs.push([j as f32 / segments as f32, i as f32 / 2.0]);
            }
            
            if i > 0 {
                for j in 0..segments {
                    let current = base_offset + (i * segments + j) as u32;
                    let next = base_offset + (i * segments + (j + 1) % segments) as u32;
                    let prev_ring = current - segments as u32;
                    let prev_next = prev_ring + 1;
                    
                    if (j + 1) % segments == 0 {
                        indices.extend_from_slice(&[
                            prev_ring, current, base_offset + (i * segments) as u32,
                            prev_ring, base_offset + (i * segments) as u32, base_offset + ((i - 1) * segments) as u32,
                        ]);
                    } else {
                        indices.extend_from_slice(&[
                            prev_ring, current, next,
                            prev_ring, next, prev_next,
                        ]);
                    }
                }
            }
        }
        
        *vertex_offset += 3 * segments;
    }
}

pub fn create_volumetric_cloud_mesh(seed: u32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    // Create cloud using metaballs
    let num_spheres = 5 + (seed % 4) as usize;
    let mut vertex_offset: usize = 0;
    
    for i in 0..num_spheres {
        let offset_x = ((seed + i as u32) % 7) as f32 - 3.5;
        let offset_y = ((seed + i as u32 * 2) % 5) as f32 - 2.5;
        let offset_z = ((seed + i as u32 * 3) % 7) as f32 - 3.5;
        let radius = 5.0 + ((seed + i as u32) % 4) as f32;
        
        // Create low-poly sphere
        let rings: usize = 8;
        let sectors: usize = 12;
        
        for r in 0..=rings {
            let phi = PI * r as f32 / rings as f32;
            let y = radius * phi.cos();
            let ring_radius = radius * phi.sin();
            
            for s in 0..=sectors {
                let theta = 2.0 * PI * s as f32 / sectors as f32;
                let x = ring_radius * theta.cos() + offset_x;
                let z = ring_radius * theta.sin() + offset_z;
                
                positions.push([x, y + offset_y, z]);
                let normal = Vec3::new(x - offset_x, y, z - offset_z).normalize();
                normals.push(normal.to_array());
                uvs.push([s as f32 / sectors as f32, r as f32 / rings as f32]);
            }
        }
        
        // Generate indices for this sphere
        let sphere_base = vertex_offset as u32;
        for r in 0..rings {
            for s in 0..sectors {
                let current = sphere_base + (r * (sectors + 1) + s) as u32;
                let next = current + (sectors + 1) as u32;
                
                indices.push(current);
                indices.push(next);
                indices.push(current + 1);
                
                indices.push(current + 1);
                indices.push(next);
                indices.push(next + 1);
            }
        }
        
        vertex_offset += (rings + 1) * (sectors + 1);
    }
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}

pub fn create_detailed_balloon_mesh(_target_type: &crate::targets::TargetType) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    // Balloon parameters
    let radius = 1.5;
    let height_stretch = 1.2;
    let segments: usize = 16;
    let rings: usize = 12;
    
    // Generate balloon body with more realistic shape
    for r in 0..=rings {
        let phi = PI * r as f32 / rings as f32;
        let y = radius * height_stretch * phi.cos();
        
        // Vary radius to create teardrop shape
        let shape_factor = if r < rings / 2 {
            // Upper half - round
            phi.sin()
        } else {
            // Lower half - taper to point
            let t = (r - rings / 2) as f32 / (rings / 2) as f32;
            (1.0 - t * 0.8) * phi.sin()
        };
        
        let ring_radius = radius * shape_factor;
        
        for s in 0..=segments {
            let theta = 2.0 * PI * s as f32 / segments as f32;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();
            
            positions.push([x, y, z]);
            
            // Calculate normal
            let normal = Vec3::new(x, y / height_stretch, z).normalize();
            normals.push(normal.to_array());
            uvs.push([s as f32 / segments as f32, r as f32 / rings as f32]);
        }
    }
    
    // Generate indices
    for r in 0..rings {
        for s in 0..segments {
            let current = r * (segments + 1) + s;
            let next = current + segments + 1;
            
            indices.push(current as u32);
            indices.push(next as u32);
            indices.push((current + 1) as u32);
            
            indices.push((current + 1) as u32);
            indices.push(next as u32);
            indices.push((next + 1) as u32);
        }
    }
    
    // Add knot at bottom
    let knot_base = positions.len() as u32;
    let knot_y = -radius * height_stretch - 0.2;
    let knot_radius = 0.1;
    
    for i in 0..segments {
        let angle = i as f32 * 2.0 * PI / segments as f32;
        positions.push([
            knot_radius * angle.cos(),
            knot_y,
            knot_radius * angle.sin(),
        ]);
        normals.push([angle.cos(), 0.0, angle.sin()]);
        uvs.push([i as f32 / segments as f32, 1.0]);
    }
    
    // Connect knot to balloon
    let bottom_ring_start = (rings * (segments + 1)) as u32;
    for i in 0..segments {
        let balloon_idx = bottom_ring_start + i as u32;
        let knot_idx = knot_base + i as u32;
        let next_knot = knot_base + ((i + 1) % segments) as u32;
        
        indices.push(balloon_idx);
        indices.push(knot_idx);
        indices.push(next_knot);
    }
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}
