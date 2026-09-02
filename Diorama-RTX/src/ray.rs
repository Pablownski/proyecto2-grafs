// Se usa completamente a partir de Fase 2 (generación de rayos e intersecciones).
#![allow(dead_code)]

use nalgebra_glm::Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_glm::vec3;

    #[test]
    fn at_zero_returns_origin() {
        let ray = Ray::new(vec3(1.0, 2.0, 3.0), vec3(0.0, 0.0, 1.0));
        assert_eq!(ray.at(0.0), vec3(1.0, 2.0, 3.0));
    }

    #[test]
    fn at_positive_t_moves_along_direction() {
        let ray = Ray::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0));
        assert_eq!(ray.at(5.0), vec3(5.0, 0.0, 0.0));
    }

    #[test]
    fn at_negative_t_moves_backwards() {
        let ray = Ray::new(vec3(2.0, 2.0, 2.0), vec3(0.0, 1.0, 0.0));
        assert_eq!(ray.at(-2.0), vec3(2.0, 0.0, 2.0));
    }

    #[test]
    fn at_scales_non_unit_direction() {
        let ray = Ray::new(vec3(0.0, 0.0, 0.0), vec3(2.0, -1.0, 0.5));
        assert_eq!(ray.at(2.0), vec3(4.0, -2.0, 1.0));
    }
}
