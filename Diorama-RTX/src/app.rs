use std::time::Instant;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::vec3;

use crate::agency;
use crate::camera::OrbitCamera;
use crate::config::{
    FB_HEIGHT, FB_WIDTH, MAX_DT, ORBIT_PITCH_SPEED, ORBIT_SCROLL_ZOOM_FACTOR, ORBIT_YAW_SPEED,
    ORBIT_ZOOM_SPEED, PLAYER_FOV_DEGREES, PLAYER_LOOK_PITCH_SPEED, PLAYER_LOOK_YAW_SPEED,
    PLAYER_RUN_SPEED, PLAYER_WALK_SPEED, WINDOW_SCALE, WINDOW_TITLE,
};
use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::renderer::{self, RenderParams};
use crate::scene::Scene;

#[derive(PartialEq, Eq, Clone, Copy)]
enum CameraMode {
    Orbit,
    FirstPerson,
}

pub struct App {
    window: Window,
    framebuffer: Framebuffer,
    scene: Scene,
    camera_mode: CameraMode,
    orbit_camera: OrbitCamera,
    player: Player,
    render_params: RenderParams,
    aspect: f32,
}

impl App {
    pub fn new() -> Self {
        let window = Window::new(
            WINDOW_TITLE,
            FB_WIDTH * WINDOW_SCALE,
            FB_HEIGHT * WINDOW_SCALE,
            WindowOptions::default(),
        )
        .expect("failed to create window");

        let framebuffer = Framebuffer::new(FB_WIDTH, FB_HEIGHT);
        let mut scene = Scene::new();
        agency::build(&mut scene);

        let aspect = FB_WIDTH as f32 / FB_HEIGHT as f32;
        let orbit_camera =
            OrbitCamera::new(vec3(0.0, 6.0, 12.0), 0.15, 0.35, 55.0, 55f32.to_radians());
        // Nace en la plaza, mirando hacia la entrada del edificio (+Z).
        let player = Player::new(vec3(0.0, -1.2, -3.0), 0.0);
        let render_params = orbit_camera.render_params(aspect);

        print_controls_help();

        Self {
            window,
            framebuffer,
            scene,
            camera_mode: CameraMode::Orbit,
            orbit_camera,
            player,
            render_params,
            aspect,
        }
    }

    pub fn run(&mut self) {
        renderer::render(&self.scene, &self.render_params, &mut self.framebuffer);

        let mut last_frame = Instant::now();

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let now = Instant::now();
            let dt = (now - last_frame).as_secs_f32().min(MAX_DT);
            last_frame = now;

            if self.window.is_key_pressed(Key::Tab, KeyRepeat::No) {
                self.camera_mode = match self.camera_mode {
                    CameraMode::Orbit => CameraMode::FirstPerson,
                    CameraMode::FirstPerson => CameraMode::Orbit,
                };
            }
            if self.window.is_key_pressed(Key::F1, KeyRepeat::No) {
                print_controls_help();
            }

            let changed = match self.camera_mode {
                CameraMode::Orbit => self.handle_orbit_input(dt),
                CameraMode::FirstPerson => self.handle_first_person_input(dt),
            };

            if changed {
                self.render_params = match self.camera_mode {
                    CameraMode::Orbit => self.orbit_camera.render_params(self.aspect),
                    CameraMode::FirstPerson => self
                        .player
                        .render_params(PLAYER_FOV_DEGREES.to_radians(), self.aspect),
                };
                renderer::render(&self.scene, &self.render_params, &mut self.framebuffer);
            }

            self.window
                .update_with_buffer(
                    self.framebuffer.as_slice(),
                    self.framebuffer.width,
                    self.framebuffer.height,
                )
                .expect("failed to update window buffer");
        }
    }

    /// Lee teclado y rueda del mouse para orbitar/hacer zoom. Devuelve `true`
    /// si la cámara cambió, para no re-renderizar cuando nada se movió.
    fn handle_orbit_input(&mut self, dt: f32) -> bool {
        let mut delta_yaw = 0.0f32;
        let mut delta_pitch = 0.0f32;
        if self.window.is_key_down(Key::Left) {
            delta_yaw -= ORBIT_YAW_SPEED * dt;
        }
        if self.window.is_key_down(Key::Right) {
            delta_yaw += ORBIT_YAW_SPEED * dt;
        }
        if self.window.is_key_down(Key::Up) {
            delta_pitch += ORBIT_PITCH_SPEED * dt;
        }
        if self.window.is_key_down(Key::Down) {
            delta_pitch -= ORBIT_PITCH_SPEED * dt;
        }

        let mut delta_radius = 0.0f32;
        if self.window.is_key_down(Key::Q) {
            delta_radius -= ORBIT_ZOOM_SPEED * dt;
        }
        if self.window.is_key_down(Key::E) {
            delta_radius += ORBIT_ZOOM_SPEED * dt;
        }
        if let Some((_, scroll_y)) = self.window.get_scroll_wheel() {
            delta_radius -= scroll_y * ORBIT_SCROLL_ZOOM_FACTOR;
        }

        let changed = delta_yaw != 0.0 || delta_pitch != 0.0 || delta_radius != 0.0;
        if delta_yaw != 0.0 || delta_pitch != 0.0 {
            self.orbit_camera.orbit(delta_yaw, delta_pitch);
        }
        if delta_radius != 0.0 {
            self.orbit_camera.zoom(delta_radius);
        }
        changed
    }

    /// `WASD` mueve al jugador según hacia dónde mira; las flechas giran la
    /// mirada (yaw/pitch); `Shift` corre. Devuelve `true` si algo cambió.
    fn handle_first_person_input(&mut self, dt: f32) -> bool {
        let mut delta_yaw = 0.0f32;
        let mut delta_pitch = 0.0f32;
        if self.window.is_key_down(Key::Left) {
            delta_yaw -= PLAYER_LOOK_YAW_SPEED * dt;
        }
        if self.window.is_key_down(Key::Right) {
            delta_yaw += PLAYER_LOOK_YAW_SPEED * dt;
        }
        if self.window.is_key_down(Key::Up) {
            delta_pitch += PLAYER_LOOK_PITCH_SPEED * dt;
        }
        if self.window.is_key_down(Key::Down) {
            delta_pitch -= PLAYER_LOOK_PITCH_SPEED * dt;
        }
        if delta_yaw != 0.0 || delta_pitch != 0.0 {
            self.player.look(delta_yaw, delta_pitch);
        }

        let mut forward = 0.0f32;
        let mut strafe = 0.0f32;
        if self.window.is_key_down(Key::W) {
            forward += 1.0;
        }
        if self.window.is_key_down(Key::S) {
            forward -= 1.0;
        }
        if self.window.is_key_down(Key::D) {
            strafe += 1.0;
        }
        if self.window.is_key_down(Key::A) {
            strafe -= 1.0;
        }

        let moving = forward != 0.0 || strafe != 0.0;
        if moving {
            let running =
                self.window.is_key_down(Key::LeftShift) || self.window.is_key_down(Key::RightShift);
            let speed = if running {
                PLAYER_RUN_SPEED
            } else {
                PLAYER_WALK_SPEED
            };
            self.player.walk(forward, strafe, speed, dt);
        }

        moving || delta_yaw != 0.0 || delta_pitch != 0.0
    }
}

fn print_controls_help() {
    println!("=== Controles ===");
    println!("Tab: cambiar camara orbital / primera persona");
    println!("Orbital -> Flechas: orbitar | Q/E o rueda: acercar/alejar");
    println!("Primera persona -> WASD: moverse | Flechas: mirar | Shift: correr");
    println!("F1: mostrar esta ayuda | Escape: salir");
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
