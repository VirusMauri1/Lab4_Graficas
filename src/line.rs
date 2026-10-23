use crate::framebuffer::Framebuffer;
use nalgebra_glm::Vec3;

pub fn line(framebuffer: &mut Framebuffer, start: Vec3, end: Vec3) {
    let (mut x1, mut y1) = (start.x.round() as i32, start.y.round() as i32);
    let (x2, y2) = (end.x.round() as i32, end.y.round() as i32);

    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;

    let steps = dx.max(-dy).max(1) as f32;
    let mut step = 0;

    loop {
        let t = step as f32 / steps;
        let z = start.z + (end.z - start.z) * t;
        framebuffer.point(&Vec3::new(x1 as f32, y1 as f32, z));

        if x1 == x2 && y1 == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x1 += sx;
        }
        if e2 <= dx {
            err += dx;
            y1 += sy;
        }
        step += 1;
    }
}
