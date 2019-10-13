use cef_sys::cef_color_t;

#[derive(Debug, Clone, Copy)]
pub struct Color(u32);

impl Color {
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        let rgba = u32::from_be_bytes([r, g, b, a]);
        Color(rgba)
    }

    pub fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Color {
        let argb = u32::from_be_bytes([a, r, g, b]);
        Color(argb)
    }

    pub fn to_color(&self) -> cef_color_t {
        self.0
    }
}
