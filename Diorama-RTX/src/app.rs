use std::time::Instant;

use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{vec2, vec3};

use crate::camera::OrbitCamera;
use crate::config::{
    FB_HEIGHT, FB_WIDTH, MAX_DT, ORBIT_PITCH_SPEED, ORBIT_SCROLL_ZOOM_FACTOR, ORBIT_YAW_SPEED,
    ORBIT_ZOOM_SPEED, WINDOW_SCALE, WINDOW_TITLE,
};
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::material::load_core_materials;
use crate::renderer::{self, RenderParams};
use crate::scene::Scene;

pub struct App {
    window: Window,
    framebuffer: Framebuffer,
    scene: Scene,
    camera: OrbitCamera,
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
        let scene = build_demo_scene();
        let aspect = FB_WIDTH as f32 / FB_HEIGHT as f32;
        let camera = OrbitCamera::new(vec3(1.4, 0.0, -1.0), 0.0, 0.3, 9.0, 60f32.to_radians());
        let render_params = camera.render_params(aspect);

        Self {
            window,
            framebuffer,
            scene,
            camera,
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

            if self.handle_orbit_input(dt) {
                self.render_params = self.camera.render_params(self.aspect);
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
            self.camera.orbit(delta_yaw, delta_pitch);
        }
        if delta_radius != 0.0 {
            self.camera.zoom(delta_radius);
        }
        changed
    }
}

/// Escena temporal de la Fase 2-4: los cinco materiales principales aplicados
/// a cubos a distintas profundidades, para validar intersección, cámara
/// orbital y muestreo de texturas. Se reemplaza por `agency::build` en la
/// Fase 8.
fn build_demo_scene() -> Scene {
    let mut scene = Scene::new();
    scene.materials = load_core_materials(&mut scene.textures);

    // Piedra, mármol, madera, vidrio, metal (índices 0..4 de load_core_materials).
    let mut stone = Cube::new(vec3(-3.0, -1.0, -1.0), vec3(-1.5, 0.5, 0.5), 0);
    stone.uv_scale = vec2(2.0, 2.0);
    let mut marble = Cube::new(vec3(-1.0, -1.0, -1.5), vec3(0.5, 0.5, 0.0), 1);
    marble.uv_scale = vec2(1.5, 1.5);
    let mut wood = Cube::new(vec3(0.8, -1.0, -2.0), vec3(2.3, 0.5, -0.5), 2);
    wood.uv_scale = vec2(2.0, 1.0);
    let mut glass_cube = Cube::new(vec3(2.6, -1.0, -1.0), vec3(4.1, 0.5, 0.5), 3);
    glass_cube.uv_scale = vec2(1.0, 1.0);
    let mut metal = Cube::new(vec3(4.4, -1.0, -1.5), vec3(5.9, 0.5, 0.0), 4);
    metal.uv_scale = vec2(2.0, 1.0);

    // Piso: reutiliza la piedra con un mosaico grande.
    let mut floor = Cube::new(vec3(-20.0, -1.5, -20.0), vec3(20.0, -1.0, 20.0), 0);
    floor.uv_scale = vec2(8.0, 8.0);

    scene.cubes.push(stone);
    scene.cubes.push(marble);
    scene.cubes.push(wood);
    scene.cubes.push(glass_cube);
    scene.cubes.push(metal);
    scene.cubes.push(floor);

    scene
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
