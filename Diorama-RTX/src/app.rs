use std::time::Instant;

use minifb::{Key, KeyRepeat, MouseButton, Window, WindowOptions};
use nalgebra_glm::{Vec3, vec3};

use crate::agency::{self, METAL, PREVIEW_INVALID, PREVIEW_VALID, STONE, WOOD};
use crate::building::{self, BuildState, PieceKind};
use crate::camera::OrbitCamera;
use crate::collision;
use crate::config::{
    FB_HEIGHT, FB_WIDTH, IDLE_REFINE_DELAY, INTERACTIVE_FB_HEIGHT, INTERACTIVE_FB_WIDTH,
    INTERACTIVE_TRACE_DEPTH, JUMP_SPEED, MAX_DT, MAX_TRACE_DEPTH, ORBIT_PITCH_SPEED,
    ORBIT_SCROLL_ZOOM_FACTOR, ORBIT_YAW_SPEED, ORBIT_ZOOM_SPEED, PLAYER_FOV_DEGREES,
    PLAYER_LOOK_PITCH_SPEED, PLAYER_LOOK_YAW_SPEED, PLAYER_RUN_SPEED, PLAYER_WALK_SPEED,
    WINDOW_SCALE, WINDOW_TITLE,
};
use crate::framebuffer::Framebuffer;
use crate::hud;
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
    /// Framebuffer de calidad completa (`FB_WIDTH`x`FB_HEIGHT`, `MAX_TRACE_DEPTH`).
    framebuffer: Framebuffer,
    /// Framebuffer reducido para cuando la cámara está en movimiento (Fase 13).
    interactive_framebuffer: Framebuffer,
    /// Cuál de los dos framebuffers es el más reciente y debe mostrarse.
    showing_interactive: bool,
    /// Imagen final que se envía a la ventana: el render (escalado a
    /// `FB_WIDTH`x`FB_HEIGHT` si es el interactivo) más el HUD encima.
    display: Vec<u32>,
    /// Hay que recomponer `display` (nuevo render o cambio en el HUD).
    display_dirty: bool,
    /// Panel de controles abierto/cerrado con `M`.
    show_controls: bool,
    /// Hubo un cambio reciente que todavía no se refinó a calidad completa.
    needs_quality_refine: bool,
    last_change_at: Instant,
    scene: Scene,
    camera_mode: CameraMode,
    orbit_camera: OrbitCamera,
    player: Player,
    build_state: BuildState,
    left_mouse_was_down: bool,
    right_mouse_was_down: bool,
    stats: RenderStats,
}

/// Medición simple de tiempos (sección 13: "medir tiempos primero"). Junta
/// cuadros interactivos y de calidad en una ventana de ~1s y los imprime por
/// consola, para poder comparar antes/después de optimizar.
struct RenderStats {
    window_start: Instant,
    interactive_frames: u32,
    interactive_total_secs: f32,
    quality_frames: u32,
    quality_total_secs: f32,
}

impl RenderStats {
    fn new() -> Self {
        Self {
            window_start: Instant::now(),
            interactive_frames: 0,
            interactive_total_secs: 0.0,
            quality_frames: 0,
            quality_total_secs: 0.0,
        }
    }

    fn record_interactive(&mut self, elapsed_secs: f32) {
        self.interactive_frames += 1;
        self.interactive_total_secs += elapsed_secs;
    }

    fn record_quality(&mut self, elapsed_secs: f32) {
        self.quality_frames += 1;
        self.quality_total_secs += elapsed_secs;
    }

    /// Imprime y reinicia la ventana si ya pasó ~1 segundo y hubo actividad.
    fn maybe_report(&mut self, now: Instant) {
        let window_secs = now.duration_since(self.window_start).as_secs_f32();
        if window_secs < 1.0 {
            return;
        }
        if self.interactive_frames > 0 || self.quality_frames > 0 {
            let fps = self.interactive_frames as f32 / window_secs;
            let avg_interactive_ms = if self.interactive_frames > 0 {
                self.interactive_total_secs / self.interactive_frames as f32 * 1000.0
            } else {
                0.0
            };
            let avg_quality_ms = if self.quality_frames > 0 {
                self.quality_total_secs / self.quality_frames as f32 * 1000.0
            } else {
                0.0
            };
            println!(
                "[render] interactivo: {} cuadros ({:.1} fps, {:.1} ms/cuadro) | calidad: {} cuadros ({:.1} ms/cuadro)",
                self.interactive_frames,
                fps,
                avg_interactive_ms,
                self.quality_frames,
                avg_quality_ms
            );
        }
        *self = Self::new();
    }
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
        let interactive_framebuffer = Framebuffer::new(INTERACTIVE_FB_WIDTH, INTERACTIVE_FB_HEIGHT);
        let mut scene = Scene::new();
        agency::build(&mut scene);

        let orbit_camera =
            OrbitCamera::new(vec3(0.0, 6.0, 12.0), 0.15, 0.35, 55.0, 55f32.to_radians());
        // Nace en la plaza, mirando hacia la entrada del edificio (+Z).
        let player = Player::new(vec3(0.0, -1.2, -3.0), 0.0);

        print_controls_help();

        let mut app = Self {
            window,
            framebuffer,
            interactive_framebuffer,
            showing_interactive: false,
            display: vec![0; FB_WIDTH * FB_HEIGHT],
            display_dirty: true,
            show_controls: false,
            needs_quality_refine: false,
            last_change_at: Instant::now(),
            scene,
            camera_mode: CameraMode::Orbit,
            orbit_camera,
            player,
            build_state: BuildState::new(STONE),
            left_mouse_was_down: false,
            right_mouse_was_down: false,
            stats: RenderStats::new(),
        };

        let params = app.current_render_params(FB_WIDTH, FB_HEIGHT);
        renderer::render(&app.scene, &params, &mut app.framebuffer, MAX_TRACE_DEPTH);
        app
    }

    fn current_render_params(&self, width: usize, height: usize) -> RenderParams {
        let aspect = width as f32 / height as f32;
        match self.camera_mode {
            CameraMode::Orbit => self.orbit_camera.render_params(aspect),
            CameraMode::FirstPerson => self
                .player
                .render_params(PLAYER_FOV_DEGREES.to_radians(), aspect),
        }
    }

    pub fn run(&mut self) {
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
            if self.window.is_key_pressed(Key::M, KeyRepeat::No) {
                self.show_controls = !self.show_controls;
                self.display_dirty = true;
            }

            let mut changed = match self.camera_mode {
                CameraMode::Orbit => self.handle_orbit_input(dt),
                CameraMode::FirstPerson => self.handle_first_person_input(dt),
            };

            if self.camera_mode == CameraMode::FirstPerson {
                changed |= self.handle_build_input();
            } else if !self.scene.preview_cubes.is_empty() {
                self.scene.preview_cubes.clear();
                changed = true;
            }

            // Calidad adaptativa (sección 13.2): mientras algo cambia se
            // renderiza a resolución/profundidad reducidas; al quedar
            // quieto un instante corto se refina a calidad completa una
            // sola vez. Si nada cambió y ya está refinado, no se vuelve a
            // trazar: se reutiliza el framebuffer ya calculado.
            if changed {
                self.last_change_at = now;
                self.needs_quality_refine = true;
                let params =
                    self.current_render_params(INTERACTIVE_FB_WIDTH, INTERACTIVE_FB_HEIGHT);
                let render_start = Instant::now();
                renderer::render(
                    &self.scene,
                    &params,
                    &mut self.interactive_framebuffer,
                    INTERACTIVE_TRACE_DEPTH,
                );
                self.stats
                    .record_interactive(render_start.elapsed().as_secs_f32());
                self.showing_interactive = true;
                self.display_dirty = true;
            } else if self.needs_quality_refine
                && now.duration_since(self.last_change_at).as_secs_f32() >= IDLE_REFINE_DELAY
            {
                let params = self.current_render_params(FB_WIDTH, FB_HEIGHT);
                let render_start = Instant::now();
                renderer::render(&self.scene, &params, &mut self.framebuffer, MAX_TRACE_DEPTH);
                self.stats
                    .record_quality(render_start.elapsed().as_secs_f32());
                self.showing_interactive = false;
                self.needs_quality_refine = false;
                self.display_dirty = true;
            }
            self.stats.maybe_report(now);

            if self.display_dirty {
                self.compose_display();
                self.display_dirty = false;
            }
            self.window
                .update_with_buffer(&self.display, FB_WIDTH, FB_HEIGHT)
                .expect("failed to update window buffer");
        }
    }

    /// Copia el último render a `display` (escalando por vecino más cercano
    /// si es el framebuffer interactivo) y dibuja el HUD encima. Solo se
    /// llama cuando algo cambió, para no recomponer en cada cuadro quieto.
    fn compose_display(&mut self) {
        let source = if self.showing_interactive {
            &self.interactive_framebuffer
        } else {
            &self.framebuffer
        };
        let pixels = source.as_slice();
        for y in 0..FB_HEIGHT {
            let sy = y * source.height / FB_HEIGHT;
            let src_row = &pixels[sy * source.width..(sy + 1) * source.width];
            let dst_row = &mut self.display[y * FB_WIDTH..(y + 1) * FB_WIDTH];
            for (x, dst) in dst_row.iter_mut().enumerate() {
                *dst = src_row[x * source.width / FB_WIDTH];
            }
        }

        if self.show_controls {
            hud::draw_controls_panel(&mut self.display, FB_WIDTH, FB_HEIGHT);
        }
        hud::draw_hint(&mut self.display, FB_WIDTH, FB_HEIGHT, self.show_controls);
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

    /// `WASD` mueve al jugador según hacia dónde mira (con colisión y
    /// gravedad); las flechas giran la mirada (yaw/pitch); `Shift` corre;
    /// `Espacio` salta. Devuelve `true` si algo cambió.
    fn handle_first_person_input(&mut self, dt: f32) -> bool {
        let mut delta_yaw = 0.0f32;
        let mut delta_pitch = 0.0f32;
        // Signo invertido a propósito respecto a `handle_orbit_input`: en
        // `Player::look_direction`/`forward_flat`, yaw creciente gira la
        // vista hacia +X, que es el lado izquierdo de la pantalla (el
        // vector "derecha" ya validado por el render es -X, el mismo que
        // usa `RenderParams::look_at`). Con el signo original, la flecha
        // derecha giraba la vista hacia la izquierda y viceversa.
        if self.window.is_key_down(Key::Left) {
            delta_yaw += PLAYER_LOOK_YAW_SPEED * dt;
        }
        if self.window.is_key_down(Key::Right) {
            delta_yaw -= PLAYER_LOOK_YAW_SPEED * dt;
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

        let horizontal_delta = if forward != 0.0 || strafe != 0.0 {
            let running =
                self.window.is_key_down(Key::LeftShift) || self.window.is_key_down(Key::RightShift);
            let speed = if running {
                PLAYER_RUN_SPEED
            } else {
                PLAYER_WALK_SPEED
            };
            self.player
                .horizontal_move_vector(forward, strafe, speed, dt)
        } else {
            Vec3::zeros()
        };

        if self.window.is_key_pressed(Key::Space, KeyRepeat::No) {
            self.player.jump(JUMP_SPEED);
        }

        let position_before = self.player.position;
        collision::move_player(&self.scene, &mut self.player, horizontal_delta, dt);
        let moved = (self.player.position - position_before).norm() > 1e-4;

        moved || delta_yaw != 0.0 || delta_pitch != 0.0
    }

    /// Sistema de construcción (Fase 12): `B` activa/desactiva, `1/2/3`
    /// eligen pieza, `R` rota 90°, `Z/X/C` eligen material, clic
    /// izquierdo/`Enter` coloca, clic derecho/`Delete` elimina.
    fn handle_build_input(&mut self) -> bool {
        let mut changed = false;

        if self.window.is_key_pressed(Key::B, KeyRepeat::No) {
            self.build_state.enabled = !self.build_state.enabled;
            if !self.build_state.enabled {
                self.scene.preview_cubes.clear();
            }
            changed = true;
        }

        if !self.build_state.enabled {
            return changed;
        }

        // Estas selecciones no necesitan marcar `changed` por separado: la
        // vista previa se recalcula y se vuelve a renderizar de todas formas
        // mientras la construcción está activa (ver `changed = true` abajo).
        if self.window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            self.build_state.kind = PieceKind::Wall;
        }
        if self.window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            self.build_state.kind = PieceKind::Floor;
        }
        if self.window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            self.build_state.kind = PieceKind::Ramp;
        }
        if self.window.is_key_pressed(Key::R, KeyRepeat::No) {
            self.build_state.rotate();
        }
        if self.window.is_key_pressed(Key::Z, KeyRepeat::No) {
            self.build_state.material = WOOD;
        }
        if self.window.is_key_pressed(Key::X, KeyRepeat::No) {
            self.build_state.material = STONE;
        }
        if self.window.is_key_pressed(Key::C, KeyRepeat::No) {
            self.build_state.material = METAL;
        }

        let preview = building::compute_preview(&self.scene, &self.player, &self.build_state);
        self.scene.preview_cubes.clear();
        if let Some(preview) = &preview {
            // Vuelve a pintar cada cubo con el material fantasma (verde/rojo)
            // en vez del material elegido, para no duplicar la geometría.
            let tint = if preview.valid {
                PREVIEW_VALID
            } else {
                PREVIEW_INVALID
            };
            for cube in &preview.cubes {
                let mut ghost = crate::cube::Cube::new(cube.min, cube.max, tint);
                ghost.uv_scale = cube.uv_scale;
                self.scene.preview_cubes.push(ghost);
            }
        }
        changed = true;

        let left_down = self.window.get_mouse_down(MouseButton::Left);
        let place_pressed = (left_down && !self.left_mouse_was_down)
            || self.window.is_key_pressed(Key::Enter, KeyRepeat::No);
        self.left_mouse_was_down = left_down;

        if place_pressed
            && let Some(preview) = &preview
            && preview.valid
        {
            self.scene
                .dynamic_cubes
                .extend(preview.cubes.iter().map(clone_cube));
            changed = true;
        }

        let right_down = self.window.get_mouse_down(MouseButton::Right);
        let delete_pressed = (right_down && !self.right_mouse_was_down)
            || self.window.is_key_pressed(Key::Delete, KeyRepeat::No);
        self.right_mouse_was_down = right_down;

        if delete_pressed
            && let Some(index) = building::find_targeted_piece(&self.scene, &self.player)
        {
            self.scene.dynamic_cubes.remove(index);
            changed = true;
        }

        changed
    }
}

fn clone_cube(cube: &crate::cube::Cube) -> crate::cube::Cube {
    let mut copy = crate::cube::Cube::new(cube.min, cube.max, cube.material_id);
    copy.uv_scale = cube.uv_scale;
    copy.build_piece = cube.build_piece;
    copy
}

fn print_controls_help() {
    println!("=== Controles ===");
    println!("Tab: cambiar camara orbital / primera persona");
    println!("Orbital -> Flechas: orbitar | Q/E o rueda: acercar/alejar");
    println!("Primera persona -> WASD: moverse | Flechas: mirar | Shift: correr | Espacio: saltar");
    println!("Construccion -> B: activar/desactivar | 1/2/3: pared/piso/rampa | R: rotar");
    println!(
        "  Z/X/C: madera/piedra/metal | Click izq o Enter: colocar | Click der o Delete: eliminar"
    );
    println!("M: mostrar/ocultar panel de controles | F1: mostrar esta ayuda | Escape: salir");
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
