use std::f32::consts::TAU;

use nalgebra_glm::{Vec3, vec3};

use crate::renderer::RenderParams;

// Un poco menos de PI/2 para nunca dejar `forward` paralelo al `up` del mundo,
// lo cual produciría un producto cruzado nulo y una normalización con NaN.
const MIN_PITCH: f32 = -1.45;
const MAX_PITCH: f32 = 1.45;

const MIN_RADIUS: f32 = 3.0;
const MAX_RADIUS: f32 = 40.0;

pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub radius: f32,
    pub fov: f32,
}

impl OrbitCamera {
    pub fn new(target: Vec3, yaw: f32, pitch: f32, radius: f32, fov: f32) -> Self {
        Self {
            target,
            yaw: yaw.rem_euclid(TAU),
            pitch: pitch.clamp(MIN_PITCH, MAX_PITCH),
            radius: radius.clamp(MIN_RADIUS, MAX_RADIUS),
            fov,
        }
    }

    /// Orbita horizontalmente (yaw, 360° continuos) y verticalmente (pitch,
    /// limitado antes de los polos).
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw = (self.yaw + delta_yaw).rem_euclid(TAU);
        self.pitch = (self.pitch + delta_pitch).clamp(MIN_PITCH, MAX_PITCH);
    }

    /// Acerca (`delta_radius < 0`) o aleja (`delta_radius > 0`) la cámara,
    /// respetando el radio mínimo y máximo para no atravesar la escena.
    pub fn zoom(&mut self, delta_radius: f32) {
        self.radius = (self.radius + delta_radius).clamp(MIN_RADIUS, MAX_RADIUS);
    }

    pub fn eye(&self) -> Vec3 {
        let x = self.radius * self.pitch.cos() * self.yaw.sin();
        let y = self.radius * self.pitch.sin();
        let z = self.radius * self.pitch.cos() * self.yaw.cos();
        self.target + vec3(x, y, z)
    }

    pub fn render_params(&self, aspect: f32) -> RenderParams {
        RenderParams::look_at(
            self.eye(),
            self.target,
            vec3(0.0, 1.0, 0.0),
            self.fov,
            aspect,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_finite(v: Vec3) {
        assert!(v.x.is_finite() && v.y.is_finite() && v.z.is_finite());
    }

    #[test]
    fn full_horizontal_orbit_wraps_and_stays_finite() {
        let mut cam = OrbitCamera::new(Vec3::zeros(), 0.0, 0.2, 10.0, 60f32.to_radians());
        let steps = 36;
        for _ in 0..steps {
            cam.orbit(TAU / steps as f32, 0.0);
            assert_finite(cam.eye());
        }
        assert!(cam.yaw >= 0.0 && cam.yaw < TAU);
    }

    #[test]
    fn pitch_is_clamped_before_the_poles() {
        let mut cam = OrbitCamera::new(Vec3::zeros(), 0.0, 0.0, 10.0, 60f32.to_radians());
        cam.orbit(0.0, 100.0);
        assert!(cam.pitch <= MAX_PITCH);
        assert_finite(cam.eye());

        cam.orbit(0.0, -200.0);
        assert!(cam.pitch >= MIN_PITCH);
        assert_finite(cam.eye());
    }

    #[test]
    fn zoom_is_clamped_to_min_and_max_radius() {
        let mut cam = OrbitCamera::new(Vec3::zeros(), 0.0, 0.0, 10.0, 60f32.to_radians());
        cam.zoom(-1000.0);
        assert_eq!(cam.radius, MIN_RADIUS);

        cam.zoom(1000.0);
        assert_eq!(cam.radius, MAX_RADIUS);
    }

    #[test]
    fn render_params_look_at_target_has_valid_basis() {
        let cam = OrbitCamera::new(vec3(1.0, 2.0, 3.0), 0.7, 0.3, 8.0, 60f32.to_radians());
        let params = cam.render_params(16.0 / 9.0);
        assert_finite(params.eye);
        assert_finite(params.forward);
        assert_finite(params.right);
        assert_finite(params.up);
        assert!((params.forward.norm() - 1.0).abs() < 1e-4);
    }
}
