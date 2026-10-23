use crate::color::Color;
use nalgebra_glm::Vec3;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    pub zbuffer: Vec<f32>,
    pub id_buffer: Vec<i32>,
    background: Vec<u32>,
    current_color: Color,
    current_id: i32,
    pub depth_test: bool,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Framebuffer {
            width,
            height,
            buffer: vec![0; size],
            zbuffer: vec![f32::NEG_INFINITY; size],
            id_buffer: vec![-1; size],
            background: vec![0; size],
            current_color: Color::new(255, 255, 255),
            current_id: -1,
            depth_test: true,
        }
    }

    /// Imagen de fondo que se copia en cada clear().
    pub fn set_background(&mut self, background: Vec<u32>) {
        assert_eq!(background.len(), self.buffer.len());
        self.background = background;
    }

    pub fn clear(&mut self) {
        self.buffer.copy_from_slice(&self.background);
        self.zbuffer.fill(f32::NEG_INFINITY);
        self.id_buffer.fill(-1);
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn set_current_id(&mut self, id: i32) {
        self.current_id = id;
    }

    #[inline]
    pub fn point(&mut self, v: &Vec3) {
        let x = v.x.round();
        let y = v.y.round();
        if x < 0.0 || y < 0.0 || x >= self.width as f32 || y >= self.height as f32 {
            return;
        }
        let index = y as usize * self.width + x as usize;

        // Mayor z = más cerca de la cámara.
        if self.depth_test && v.z < self.zbuffer[index] {
            return;
        }
        self.zbuffer[index] = v.z;
        self.id_buffer[index] = self.current_id;
        self.buffer[index] = self.current_color.to_hex();
    }

    pub fn id_at(&self, x: usize, y: usize) -> i32 {
        if x >= self.width || y >= self.height {
            return -1;
        }
        self.id_buffer[y * self.width + x]
    }

    pub fn save_png(&self, path: &str) -> Result<(), image::ImageError> {
        let mut rgb = Vec::with_capacity(self.buffer.len() * 3);
        for &pixel in &self.buffer {
            let c = Color::from_hex(pixel);
            rgb.extend_from_slice(&[c.r, c.g, c.b]);
        }
        image::RgbImage::from_raw(self.width as u32, self.height as u32, rgb)
            .expect("el tamaño del buffer no coincide con la imagen")
            .save(path)
    }
}
