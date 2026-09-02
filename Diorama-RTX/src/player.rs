use std::f32::consts::TAU;

use nalgebra_glm::{Vec3, cross, normalize, vec3};

use crate::config::PLAYER_EYE_HEIGHT;
use crate::renderer::RenderParams;

// Un poco menos de PI/2, igual que en `camera.rs`, para nunca mirar
// perfectamente hacia arriba o abajo (evita normalizar un vector nulo).
const MIN_PITCH: f32 = -1.45;
const MAX_PITCH: f32 = 1.45;

pub struct Player {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    // Reservados para la Fase 11 (gravedad, salto y colisiones).
    #[allow(dead_code)]
    pub velocity: Vec3,
    #[allow(dead_code)]
    pub grounded: bool,
}

impl Player {
    pub fn new(position: Vec3, yaw: f32) -> Self {
        Self {
            position,
            yaw: yaw.rem_euclid(TAU),
            pitch: 0.0,
            velocity: Vec3::zeros(),
            grounded: false,
        }
    }

    /// Gira la mirada. El pitch queda limitado antes de los polos.
    pub fn look(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw = (self.yaw + delta_yaw).rem_euclid(TAU);
        self.pitch = (self.pitch + delta_pitch).clamp(MIN_PITCH, MAX_PITCH);
    }

    /// Dirección horizontal hacia donde mira el jugador (yaw=0 -> +Z).
    fn forward_flat(&self) -> Vec3 {
        vec3(self.yaw.sin(), 0.0, self.yaw.cos())
    }

    fn right_flat(&self) -> Vec3 {
        normalize(&cross(&self.forward_flat(), &vec3(0.0, 1.0, 0.0)))
    }

    /// Mueve al jugador sobre el plano XZ según la mirada actual. `forward`
    /// y `strafe` esperan valores en `[-1, 1]`; la diagonal se normaliza para
    /// que W+A no sea más rápido que W solo.
    pub fn walk(&mut self, forward: f32, strafe: f32, speed: f32, dt: f32) {
        let mut direction = self.forward_flat() * forward + self.right_flat() * strafe;
        let len = direction.norm();
        if len > 1e-6 {
            direction /= len;
            self.position += direction * speed * dt;
        }
    }

    fn eye_position(&self) -> Vec3 {
        self.position + vec3(0.0, PLAYER_EYE_HEIGHT, 0.0)
    }

    /// Dirección de la mirada en 3D (incluye pitch), siempre unitaria.
    fn look_direction(&self) -> Vec3 {
        vec3(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
    }

    pub fn render_params(&self, fov: f32, aspect: f32) -> RenderParams {
        let eye = self.eye_position();
        RenderParams::look_at(
            eye,
            eye + self.look_direction(),
            vec3(0.0, 1.0, 0.0),
            fov,
            aspect,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_forward_moves_towards_where_the_player_looks() {
        let mut player = Player::new(Vec3::zeros(), 0.0);
        player.walk(1.0, 0.0, 4.0, 1.0);
        let expected = player.forward_flat() * 4.0;
        assert!((player.position - expected).norm() < 1e-5);
    }

    #[test]
    fn walk_after_turning_moves_along_new_forward() {
        let mut player = Player::new(Vec3::zeros(), 0.0);
        player.look(std::f32::consts::FRAC_PI_2, 0.0);
        player.walk(1.0, 0.0, 2.0, 1.0);
        let expected = player.forward_flat() * 2.0;
        assert!((player.position - expected).norm() < 1e-5);
        // Con yaw = 90°, mirar hacia adelante ya no coincide con +Z original.
        assert!(player.position.z.abs() < 1e-4);
    }

    #[test]
    fn diagonal_movement_is_not_faster_than_straight() {
        let mut forward_only = Player::new(Vec3::zeros(), 0.0);
        forward_only.walk(1.0, 0.0, 4.0, 1.0);

        let mut diagonal = Player::new(Vec3::zeros(), 0.0);
        diagonal.walk(1.0, 1.0, 4.0, 1.0);

        assert!((forward_only.position.norm() - diagonal.position.norm()).abs() < 1e-4);
    }

    #[test]
    fn pitch_is_clamped_before_the_poles() {
        let mut player = Player::new(Vec3::zeros(), 0.0);
        player.look(0.0, 100.0);
        assert!(player.pitch <= MAX_PITCH);
        player.look(0.0, -200.0);
        assert!(player.pitch >= MIN_PITCH);
    }

    #[test]
    fn render_params_forward_matches_look_direction() {
        let player = Player::new(vec3(1.0, 0.0, 2.0), 0.3);
        let params = player.render_params(60f32.to_radians(), 16.0 / 9.0);
        let expected = player.look_direction();
        assert!((params.forward - expected).norm() < 1e-5);
    }
}
