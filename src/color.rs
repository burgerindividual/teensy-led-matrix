#[derive(Default, Eq, PartialEq, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn adjust_for_led(self) -> AdjustedColor {
        AdjustedColor {
            r: self.r,
            g: ((self.g as u16 * 29) / 50) as u8,
            b: ((self.b as u16 * 29) / 70) as u8,
        }
    }
}

#[derive(Default, Eq, PartialEq, Copy, Clone)]
pub struct AdjustedColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const BLACK: Color = Color::from_rgb(0, 0, 0);
