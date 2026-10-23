use crate::framebuffer::Framebuffer;
use crate::line::line;
use nalgebra_glm::Vec3;

/// Contorno del triángulo usando la función de línea.
pub fn triangle(framebuffer: &mut Framebuffer, a: &Vec3, b: &Vec3, c: &Vec3) {
    line(framebuffer, *a, *b);
    line(framebuffer, *b, *c);
    line(framebuffer, *c, *a);
}

/// Triángulo relleno con coordenadas baricéntricas e interpolación de z.
pub fn fill_triangle(framebuffer: &mut Framebuffer, a: &Vec3, b: &Vec3, c: &Vec3) {
    let area = edge(a, b, c.x, c.y);
    if area.abs() < 1e-6 {
        return; // triángulo degenerado (visto de canto)
    }

    let max_x = (framebuffer.width - 1) as f32;
    let max_y = (framebuffer.height - 1) as f32;
    let min_px = a.x.min(b.x).min(c.x).floor().clamp(0.0, max_x) as usize;
    let max_px = a.x.max(b.x).max(c.x).ceil().clamp(0.0, max_x) as usize;
    let min_py = a.y.min(b.y).min(c.y).floor().clamp(0.0, max_y) as usize;
    let max_py = a.y.max(b.y).max(c.y).ceil().clamp(0.0, max_y) as usize;

    for y in min_py..=max_py {
        for x in min_px..=max_px {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = edge(b, c, px, py) / area;
            let w1 = edge(c, a, px, py) / area;
            let w2 = edge(a, b, px, py) / area;

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let z = w0 * a.z + w1 * b.z + w2 * c.z;
                framebuffer.point(&Vec3::new(x as f32, y as f32, z));
            }
        }
    }
}

#[inline]
fn edge(a: &Vec3, b: &Vec3, px: f32, py: f32) -> f32 {
    (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)
}
