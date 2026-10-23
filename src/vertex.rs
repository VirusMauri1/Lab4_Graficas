use nalgebra_glm::Vec3;

/// Imprime un vértice de forma legible.
pub fn print_vertex(label: &str, v: &Vec3) {
    println!("{label}: ({:.4}, {:.4}, {:.4})", v.x, v.y, v.z);
}
