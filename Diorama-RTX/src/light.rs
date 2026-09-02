use nalgebra_glm::{Vec3, normalize, vec3};

use crate::color::Color;

// Coeficientes de atenuación por distancia para luces puntuales (alcance ~20u).
const ATTEN_LINEAR: f32 = 0.22;
const ATTEN_QUADRATIC: f32 = 0.20;

pub enum Light {
    Directional {
        direction: Vec3,
        color: Color,
        intensity: f32,
    },
    Point {
        position: Vec3,
        color: Color,
        intensity: f32,
    },
}

impl Light {
    /// Evalúa la luz desde `point`: dirección normalizada hacia la luz,
    /// distancia hasta ella (infinita para direccionales) y radiancia ya
    /// atenuada por distancia.
    pub fn sample(&self, point: Vec3) -> (Vec3, f32, Color) {
        match self {
            Light::Directional {
                direction,
                color,
                intensity,
            } => {
                let light_dir = normalize(&(-*direction));
                (light_dir, f32::INFINITY, *color * *intensity)
            }
            Light::Point {
                position,
                color,
                intensity,
            } => {
                let to_light = *position - point;
                let distance = to_light.norm();
                let light_dir = if distance > 1e-6 {
                    to_light / distance
                } else {
                    vec3(0.0, 1.0, 0.0)
                };
                let attenuation =
                    1.0 / (1.0 + ATTEN_LINEAR * distance + ATTEN_QUADRATIC * distance * distance);
                (light_dir, distance, *color * (*intensity * attenuation))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directional_light_points_opposite_its_direction() {
        let light = Light::Directional {
            direction: vec3(0.0, -1.0, 0.0),
            color: Color::WHITE,
            intensity: 1.0,
        };
        let (light_dir, distance, _) = light.sample(vec3(0.0, 0.0, 0.0));
        assert!((light_dir - vec3(0.0, 1.0, 0.0)).norm() < 1e-6);
        assert!(distance.is_infinite());
    }

    #[test]
    fn point_light_direction_and_distance_are_correct() {
        let light = Light::Point {
            position: vec3(0.0, 5.0, 0.0),
            color: Color::WHITE,
            intensity: 10.0,
        };
        let (light_dir, distance, _) = light.sample(vec3(0.0, 0.0, 0.0));
        assert!((light_dir - vec3(0.0, 1.0, 0.0)).norm() < 1e-6);
        assert!((distance - 5.0).abs() < 1e-6);
    }

    #[test]
    fn point_light_attenuates_with_distance() {
        let light = Light::Point {
            position: vec3(0.0, 0.0, 0.0),
            color: Color::WHITE,
            intensity: 10.0,
        };
        let (_, _, near_radiance) = light.sample(vec3(0.0, 0.0, 1.0));
        let (_, _, far_radiance) = light.sample(vec3(0.0, 0.0, 10.0));
        assert!(near_radiance.r > far_radiance.r);
    }
}
