/// Color RGB
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub fn to_hex(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
    }

    pub fn from_hex(hex: u32) -> Self {
        Color::new((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
    }

    pub fn scale(&self, k: f32) -> Color {
        let f = |c: u8| (c as f32 * k).clamp(0.0, 255.0) as u8;
        Color::new(f(self.r), f(self.g), f(self.b))
    }


    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let f = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
        Color::new(f(self.r, other.r), f(self.g, other.g), f(self.b, other.b))
    }
}
