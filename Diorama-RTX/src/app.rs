use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::vec3;

use crate::color::Color;
use crate::config::{FB_HEIGHT, FB_WIDTH, WINDOW_SCALE, WINDOW_TITLE};
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::renderer::{self, RenderParams};
use crate::scene::Scene;

pub struct App {
    window: Window,
    framebuffer: Framebuffer,
    scene: Scene,
    render_params: RenderParams,
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
        let render_params = RenderParams::look_at(
            vec3(0.0, 2.0, 6.0),
            vec3(0.0, 0.5, 0.0),
            vec3(0.0, 1.0, 0.0),
            60f32.to_radians(),
            FB_WIDTH as f32 / FB_HEIGHT as f32,
        );

        Self {
            window,
            framebuffer,
            scene,
            render_params,
        }
    }

    pub fn run(&mut self) {
        renderer::render(&self.scene, &self.render_params, &mut self.framebuffer);

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.window
                .update_with_buffer(
                    self.framebuffer.as_slice(),
                    self.framebuffer.width,
                    self.framebuffer.height,
                )
                .expect("failed to update window buffer");
        }
    }
}

/// Escena temporal de la Fase 2: varios cubos de colores planos a distintas
/// profundidades, para validar la intersección rayo-cubo. Se reemplaza por
/// `agency::build` en la Fase 8.
fn build_demo_scene() -> Scene {
    let mut scene = Scene::new();

    scene.flat_colors = vec![
        Color::new(0.85, 0.25, 0.25), // rojo, cubo cercano
        Color::new(0.25, 0.65, 0.35), // verde, cubo medio
        Color::new(0.30, 0.45, 0.85), // azul, cubo lejano
        Color::new(0.55, 0.55, 0.55), // gris, piso
    ];

    scene
        .cubes
        .push(Cube::new(vec3(-2.5, -1.0, -1.0), vec3(-1.0, 0.5, 0.5), 0));
    scene
        .cubes
        .push(Cube::new(vec3(-0.5, -1.0, -3.0), vec3(1.0, 1.0, -1.5), 1));
    scene
        .cubes
        .push(Cube::new(vec3(1.5, -1.0, -6.0), vec3(3.5, 1.5, -4.0), 2));
    scene.cubes.push(Cube::new(
        vec3(-20.0, -1.5, -20.0),
        vec3(20.0, -1.0, 20.0),
        3,
    ));

    scene
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
