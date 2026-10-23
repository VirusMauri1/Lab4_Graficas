use nalgebra_glm::Vec3;
use std::fs;

#[derive(Debug, Clone)]
pub struct Face {
    pub vertex_indices: Vec<[i32; 3]>,
    pub object: String,
}

pub fn load_obj(path: &str, out_vertices: &mut Vec<Vec3>, out_faces: &mut Vec<Face>) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        eprintln!("No se pudo leer {path}");
        return false;
    };

    let mut object = String::from("default");

    for (number, line) in content.lines().enumerate() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("o") => object = parts.collect::<Vec<_>>().join(" "),
            Some("v") => {
                let coords: Vec<f32> = parts.take(3).filter_map(|p| p.parse().ok()).collect();
                if coords.len() != 3 {
                    eprintln!("Vértice inválido en la línea {}", number + 1);
                    return false;
                }
                out_vertices.push(Vec3::new(coords[0], coords[1], coords[2]));
            }
            Some("f") => {
                let Some(indices) = parts.map(parse_index).collect::<Option<Vec<_>>>() else {
                    eprintln!("Cara inválida en la línea {}", number + 1);
                    return false;
                };
                if indices.len() < 3 {
                    eprintln!("Cara con menos de 3 vértices en la línea {}", number + 1);
                    return false;
                }
                for k in 1..indices.len() - 1 {
                    out_faces.push(Face {
                        vertex_indices: vec![indices[0], indices[k], indices[k + 1]],
                        object: object.clone(),
                    });
                }
            }
            _ => {} // vt, vn, s, mtllib, usemtl... se ignoran por ahora
        }
    }

    let vertex_count = out_vertices.len() as i32;
    let valid = out_faces
        .iter()
        .flat_map(|face| face.vertex_indices.iter())
        .all(|idx| idx[0] >= 0 && idx[0] < vertex_count);
    if !valid {
        eprintln!("Hay caras que apuntan a vértices que no existen");
    }
    valid
}

fn parse_index(token: &str) -> Option<[i32; 3]> {
    let mut result = [-1; 3];
    for (i, part) in token.split('/').take(3).enumerate() {
        if !part.is_empty() {
            result[i] = part.parse::<i32>().ok()? - 1;
        }
    }
    Some(result)
}

/// Convierte las caras en un solo arreglo de vértices
pub fn setup_vertex_array(vertices: &[Vec3], faces: &[Face]) -> Vec<Vec3> {
    let mut vertex_array = Vec::with_capacity(faces.len() * 3);

    for face in faces {
        for vertex_indices in &face.vertex_indices {
            let vertex_position = vertices[vertex_indices[0] as usize];
            vertex_array.push(vertex_position);
        }
    }

    vertex_array
}
