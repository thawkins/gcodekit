//! STL file import and processing.
//!
//! This module provides functionality for loading STL files (both ASCII and binary)
//! and converting them into Mesh structures for 3D machining operations.

use crate::cam::types::*;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Load an STL file and return a Mesh
pub fn load_stl(path: &Path) -> Result<Mesh, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    // Check if binary or ASCII
    if buffer.len() < 6 {
        return Err("File too small".into());
    }

    // ASCII STL files start with "solid "
    if buffer.starts_with(b"solid ") {
        load_ascii_stl(&buffer)
    } else {
        load_binary_stl(&buffer)
    }
}

/// Load ASCII STL format
fn load_ascii_stl(data: &[u8]) -> Result<Mesh, Box<dyn std::error::Error>> {
    let content = String::from_utf8(data.to_vec())?;
    let mut triangles = Vec::new();
    let mut bounds = BoundingBox::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("facet normal") {
            // Parse normal
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let normal = Point3D {
                    x: parts[2].parse()?,
                    y: parts[3].parse()?,
                    z: parts[4].parse()?,
                };

                // Skip to outer loop
                i += 1;
                while i < lines.len() && !lines[i].trim().starts_with("outer loop") {
                    i += 1;
                }
                i += 1; // Skip outer loop

                // Parse 3 vertices
                let mut vertices = Vec::new();
                for _ in 0..3 {
                    if i >= lines.len() {
                        break;
                    }
                    let vertex_line = lines[i].trim();
                    if vertex_line.starts_with("vertex") {
                        let parts: Vec<&str> = vertex_line.split_whitespace().collect();
                        if parts.len() >= 4 {
                            let vertex = Point3D {
                                x: parts[1].parse()?,
                                y: parts[2].parse()?,
                                z: parts[3].parse()?,
                            };
                            vertices.push(vertex);
                            bounds.expand(&vertex);
                        }
                    }
                    i += 1;
                }

                if vertices.len() == 3 {
                    triangles.push(Triangle {
                        vertices: [vertices[0], vertices[1], vertices[2]],
                        normal,
                    });
                }

                // Skip to endfacet
                while i < lines.len() && !lines[i].trim().starts_with("endfacet") {
                    i += 1;
                }
            }
        }
        i += 1;
    }

    Ok(Mesh { triangles, bounds })
}

/// Load binary STL format
fn load_binary_stl(data: &[u8]) -> Result<Mesh, Box<dyn std::error::Error>> {
    if data.len() < 84 {
        return Err("Binary STL file too small".into());
    }

    // Skip 80-byte header
    let mut offset = 80;

    // Read number of triangles (u32)
    let num_triangles = u32::from_le_bytes(data[offset..offset + 4].try_into()?);
    offset += 4;

    let mut triangles = Vec::new();
    let mut bounds = BoundingBox::new();

    for _ in 0..num_triangles {
        if offset + 50 > data.len() {
            break;
        }

        // Read normal (3 floats)
        let normal = Point3D {
            x: f32::from_le_bytes(data[offset..offset + 4].try_into()?),
            y: f32::from_le_bytes(data[offset + 4..offset + 8].try_into()?),
            z: f32::from_le_bytes(data[offset + 8..offset + 12].try_into()?),
        };
        offset += 12;

        // Read 3 vertices (9 floats)
        let mut vertices = [Point3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 3];
        for vertex in &mut vertices {
            *vertex = Point3D {
                x: f32::from_le_bytes(data[offset..offset + 4].try_into()?),
                y: f32::from_le_bytes(data[offset + 4..offset + 8].try_into()?),
                z: f32::from_le_bytes(data[offset + 8..offset + 12].try_into()?),
            };
            bounds.expand(vertex);
            offset += 12;
        }

        triangles.push(Triangle { vertices, normal });

        // Skip attribute byte count (u16)
        offset += 2;
    }

    Ok(Mesh { triangles, bounds })
}
