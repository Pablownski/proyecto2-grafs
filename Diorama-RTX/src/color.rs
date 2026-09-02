// Se usa completamente a partir de Fase 2 (shading) y Fase 4 (materiales).
#![allow(dead_code)]

use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };

    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// Multiplicación componente a componente (no confundir con `Mul<f32>`, que escala).
    pub fn mul_color(self, other: Color) -> Color {
        Color::new(self.r * other.r, self.g * other.g, self.b * other.b)
    }

    pub fn lerp(self, other: Color, t: f32) -> Color {
        self + (other - self) * t
    }

    pub fn clamp(self) -> Color {
        Color::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
        )
    }

    /// Corrección gamma simple (gamma = 2.2 típico) antes de convertir a u32.
    pub fn gamma_corrected(self, gamma: f32) -> Color {
        let inv_gamma = 1.0 / gamma;
        Color::new(
            self.r.max(0.0).powf(inv_gamma),
            self.g.max(0.0).powf(inv_gamma),
            self.b.max(0.0).powf(inv_gamma),
        )
    }

    /// Convierte a `0xRRGGBB`, aplicando clamp primero para evitar desbordes.
    pub fn to_u32(self) -> u32 {
        let c = self.clamp();
        let r = (c.r * 255.0).round() as u32;
        let g = (c.g * 255.0).round() as u32;
        let b = (c.b * 255.0).round() as u32;
        (r << 16) | (g << 8) | b
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Color {
        Color::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl std::ops::Sub for Color {
    type Output = Color;

    fn sub(self, rhs: Color) -> Color {
        Color::new(self.r - rhs.r, self.g - rhs.g, self.b - rhs.b)
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color::new(self.r * scalar, self.g * scalar, self.b * scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sums_components() {
        let a = Color::new(0.1, 0.2, 0.3);
        let b = Color::new(0.4, 0.4, 0.4);
        let c = a + b;
        assert!((c.r - 0.5).abs() < 1e-6);
        assert!((c.g - 0.6).abs() < 1e-6);
        assert!((c.b - 0.7).abs() < 1e-6);
    }

    #[test]
    fn mul_scalar_scales_components() {
        let c = Color::new(0.2, 0.4, 0.6) * 2.0;
        assert!((c.r - 0.4).abs() < 1e-6);
        assert!((c.g - 0.8).abs() < 1e-6);
        assert!((c.b - 1.2).abs() < 1e-6);
    }

    #[test]
    fn mul_color_multiplies_componentwise() {
        let a = Color::new(0.5, 1.0, 0.0);
        let b = Color::new(0.5, 0.5, 0.5);
        let c = a.mul_color(b);
        assert!((c.r - 0.25).abs() < 1e-6);
        assert!((c.g - 0.5).abs() < 1e-6);
        assert!((c.b - 0.0).abs() < 1e-6);
    }

    #[test]
    fn lerp_interpolates_linearly() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = a.lerp(b, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-6);
        assert!((mid.g - 0.5).abs() < 1e-6);
        assert!((mid.b - 0.5).abs() < 1e-6);
    }

    #[test]
    fn clamp_restricts_to_unit_range() {
        let c = Color::new(-0.5, 0.5, 1.5).clamp();
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.5);
        assert_eq!(c.b, 1.0);
    }

    #[test]
    fn to_u32_never_overflows_out_of_range_input() {
        let over = Color::new(2.0, -2.0, 2.0).to_u32();
        assert_eq!(over, 0x00FF00FF);
    }

    #[test]
    fn to_u32_matches_expected_channels() {
        let c = Color::new(1.0, 0.0, 0.5).to_u32();
        let r = (c >> 16) & 0xFF;
        let g = (c >> 8) & 0xFF;
        let b = c & 0xFF;
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 128);
    }
}
