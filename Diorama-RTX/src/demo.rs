//! Video demo automático: recorre un guion fijo (cámara orbital, primera
//! persona, construcción y entrada a la agencia) usando la misma física,
//! colisiones y sistema de construcción del juego, renderiza cada cuadro a
//! calidad completa y lo envía a ffmpeg junto con la música de fondo.
//!
//! Uso: `cargo run --release -- --demo [--out demo.mp4] [--res 1280x720] [--fps 30]`

use std::error::Error;
use std::f32::consts::{PI, TAU};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;

use nalgebra_glm::{Vec3, vec3};

use crate::agency::{self, METAL, STONE, WOOD};
use crate::building::{self, BuildState, PieceKind};
use crate::camera::OrbitCamera;
use crate::collision;
use crate::config::{
    JUMP_SPEED, MAX_TRACE_DEPTH, MUSIC_PATH, PLAYER_FOV_DEGREES, PLAYER_RUN_SPEED,
    PLAYER_WALK_SPEED,
};
use crate::framebuffer::Framebuffer;
use crate::hud;
use crate::player::Player;
use crate::renderer::{self, RenderParams};
use crate::scene::Scene;

pub struct DemoOptions {
    pub output: String,
    pub width: usize,
    pub height: usize,
    pub fps: u32,
}

impl DemoOptions {
    /// Lee `--out`, `--res WxH` y `--fps` de los argumentos de la línea de
    /// comandos (todos opcionales).
    pub fn from_args(args: &[String]) -> Result<Self, String> {
        let mut options = Self {
            output: "demo.mp4".to_string(),
            width: 1280,
            height: 720,
            fps: 30,
        };
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            let mut value = || {
                iter.next()
                    .ok_or_else(|| format!("falta el valor de {arg}"))
            };
            match arg.as_str() {
                "--demo" => {}
                "--out" => options.output = value()?.clone(),
                "--res" => {
                    let res = value()?;
                    let (w, h) = res
                        .split_once('x')
                        .ok_or_else(|| format!("resolución inválida: {res}"))?;
                    options.width = w.parse().map_err(|_| format!("ancho inválido: {w}"))?;
                    options.height = h.parse().map_err(|_| format!("alto inválido: {h}"))?;
                }
                "--fps" => {
                    let fps = value()?;
                    options.fps = fps.parse().map_err(|_| format!("fps inválido: {fps}"))?;
                }
                other => return Err(format!("argumento desconocido: {other}")),
            }
        }
        Ok(options)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Orbit,
    FirstPerson,
}

/// Un paso del guion. Los instantáneos (`Caption`, `Jump`, `Place`, ...) se
/// aplican y el guion sigue en el mismo cuadro; los demás duran cuadros.
#[derive(Clone, Copy)]
enum Step {
    Caption(Option<&'static str>),
    ShowControls(bool),
    Mode(Mode),
    /// Cambio relativo de yaw y valores absolutos de pitch/radio al final.
    Orbit {
        secs: f32,
        d_yaw: f32,
        pitch: f32,
        radius: f32,
    },
    Teleport {
        position: Vec3,
        yaw: f32,
    },
    Wait(f32),
    /// Gira la mirada del jugador hasta `yaw`/`pitch` absolutos.
    Look {
        secs: f32,
        yaw: f32,
        pitch: f32,
    },
    /// Gira la mirada hasta apuntar a `point` (el centro de la pantalla es
    /// lo que usa la construcción para elegir dónde va la pieza).
    LookAt {
        secs: f32,
        point: Vec3,
    },
    /// Camina (o corre) hasta `(x, z)` girando hacia el destino.
    WalkTo {
        x: f32,
        z: f32,
        run: bool,
    },
    Jump,
    Build {
        kind: PieceKind,
        rotation_deg: i32,
        material: usize,
    },
    BuildOff,
    Place,
}

fn script() -> Vec<Step> {
    use Step::*;
    vec![
        // --- Cámara orbital: menú de controles, giro completo y zoom.
        Mode(self::Mode::Orbit),
        Caption(Some("CAMARA ORBITAL")),
        ShowControls(true),
        Orbit { secs: 4.0, d_yaw: 0.35, pitch: 0.5, radius: 75.0 },
        ShowControls(false),
        Orbit { secs: 10.0, d_yaw: TAU * 0.75, pitch: 0.3, radius: 55.0 },
        Caption(Some("CAMARA ORBITAL - ACERCAR")),
        Orbit { secs: 4.0, d_yaw: 0.6, pitch: 0.12, radius: 24.0 },
        Caption(Some("CAMARA ORBITAL - ALEJAR")),
        Orbit { secs: 4.0, d_yaw: TAU * 0.25 - 0.95, pitch: 0.65, radius: 85.0 },
        // --- Primera persona: muelle, agua y salto.
        Mode(self::Mode::FirstPerson),
        Caption(Some("PRIMERA PERSONA")),
        Teleport { position: vec3(0.0, -1.3, -27.5), yaw: 0.0 },
        Wait(0.6),
        Look { secs: 2.2, yaw: 1.3, pitch: -0.3 },
        Look { secs: 2.6, yaw: -1.3, pitch: -0.15 },
        Look { secs: 1.4, yaw: 0.0, pitch: 0.05 },
        WalkTo { x: 0.0, z: -21.0, run: false },
        Caption(Some("PRIMERA PERSONA - SALTAR")),
        Jump,
        WalkTo { x: 0.0, z: -17.5, run: false },
        Jump,
        WalkTo { x: 0.0, z: -14.0, run: false },
        Caption(Some("PRIMERA PERSONA - CORRER")),
        WalkTo { x: 2.0, z: -5.0, run: true },
        WalkTo { x: 3.0, z: -1.5, run: false },
        // --- Construcción en la plaza: piso, pared y rampa.
        Caption(Some("CONSTRUCCION")),
        Look { secs: 1.0, yaw: PI * 0.5, pitch: -0.3 },
        Build { kind: PieceKind::Floor, rotation_deg: 0, material: WOOD },
        LookAt { secs: 1.2, point: vec3(6.5, -1.2, -1.5) },
        Wait(0.8),
        Place,
        Wait(0.6),
        Build { kind: PieceKind::Wall, rotation_deg: 90, material: STONE },
        LookAt { secs: 1.2, point: vec3(7.5, -0.8, -1.5) },
        Wait(0.8),
        Place,
        Wait(0.6),
        Build { kind: PieceKind::Ramp, rotation_deg: 0, material: METAL },
        LookAt { secs: 1.4, point: vec3(4.0, -1.2, 2.0) },
        Wait(0.7),
        Caption(Some("CONSTRUCCION - ROTAR PIEZA")),
        Build { kind: PieceKind::Ramp, rotation_deg: 90, material: METAL },
        Wait(0.9),
        Place,
        Wait(0.6),
        BuildOff,
        Caption(Some("SUBIR LA RAMPA Y SALTAR")),
        WalkTo { x: 3.0, z: 2.0, run: false },
        WalkTo { x: 8.4, z: 2.0, run: false },
        Jump,
        WalkTo { x: 10.3, z: 2.0, run: false },
        Wait(0.4),
        // --- Entrada a la agencia: escalinata, pórtico y atrio.
        Caption(Some("ENTRANDO A LA AGENCIA")),
        WalkTo { x: 4.0, z: 4.0, run: true },
        WalkTo { x: 1.0, z: 11.0, run: true },
        WalkTo { x: 0.0, z: 16.0, run: false },
        WalkTo { x: 0.0, z: 19.0, run: false },
        Caption(Some("ATRIO")),
        Look { secs: 2.4, yaw: 0.0, pitch: 0.65 },
        Look { secs: 2.4, yaw: -0.9, pitch: 0.2 },
        Look { secs: 2.4, yaw: 0.9, pitch: 0.2 },
        Look { secs: 1.0, yaw: 0.3, pitch: 0.0 },
        WalkTo { x: 2.8, z: 22.5, run: false },
        WalkTo { x: 2.8, z: 25.6, run: false },
        WalkTo { x: 0.0, z: 26.3, run: false },
        WalkTo { x: 0.0, z: 30.5, run: false },
        // --- Oficina de control.
        Caption(Some("OFICINA DE CONTROL")),
        Look { secs: 2.0, yaw: -0.9, pitch: -0.2 },
        Look { secs: 2.4, yaw: 0.9, pitch: -0.2 },
        Look { secs: 1.2, yaw: 0.0, pitch: 0.0 },
        WalkTo { x: 0.0, z: 33.0, run: false },
        Jump,
        Wait(1.5),
        // --- Cierre con vista orbital.
        Mode(self::Mode::Orbit),
        Caption(None),
        Orbit { secs: 6.0, d_yaw: 0.9, pitch: 0.4, radius: 60.0 },
        Wait(0.5),
    ]
}

/// Estado del mundo que avanza el guion.
struct World {
    scene: Scene,
    mode: Mode,
    orbit: OrbitCamera,
    player: Player,
    build: BuildState,
    caption: Option<&'static str>,
    show_controls: bool,
    placed: usize,
    failed_places: usize,
}

impl World {
    fn new() -> Self {
        let mut scene = Scene::new();
        agency::build(&mut scene);
        Self {
            scene,
            mode: Mode::Orbit,
            // yaw = PI: la cámara arranca frente a la fachada (lado -Z).
            orbit: OrbitCamera::new(vec3(0.0, 6.0, 12.0), PI, 0.5, 80.0, 55f32.to_radians()),
            player: Player::new(vec3(0.0, -1.2, -3.0), 0.0),
            build: BuildState::new(STONE),
            caption: None,
            show_controls: false,
            placed: 0,
            failed_places: 0,
        }
    }

    fn render_params(&self, aspect: f32) -> RenderParams {
        match self.mode {
            Mode::Orbit => self.orbit.render_params(aspect),
            Mode::FirstPerson => self
                .player
                .render_params(PLAYER_FOV_DEGREES.to_radians(), aspect),
        }
    }
}

/// Valores capturados al empezar un paso, para interpolar desde ahí.
#[derive(Clone, Copy)]
struct StepStart {
    orbit_yaw: f32,
    orbit_pitch: f32,
    orbit_radius: f32,
    look_yaw: f32,
    look_pitch: f32,
    position: Vec3,
    target_yaw: f32,
    target_pitch: f32,
}

/// Avanza el guion de a un cuadro de duración fija.
struct Director {
    steps: Vec<Step>,
    index: usize,
    elapsed: f32,
    start: Option<StepStart>,
    /// Para detectar si el jugador se atoró caminando.
    stuck_time: f32,
}

const TURN_SPEED: f32 = 2.6; // rad/s al caminar hacia un destino
const ARRIVE_DISTANCE: f32 = 0.25;
const WALK_TIMEOUT: f32 = 25.0;

impl Director {
    fn new(steps: Vec<Step>) -> Self {
        Self {
            steps,
            index: 0,
            elapsed: 0.0,
            start: None,
            stuck_time: 0.0,
        }
    }

    fn finished(&self) -> bool {
        self.index >= self.steps.len()
    }

    /// Avanza un cuadro: aplica pasos instantáneos hasta topar con uno con
    /// duración, lo avanza `dt` y luego aplica la física del jugador.
    fn tick(&mut self, world: &mut World, dt: f32) {
        let mut horizontal = Vec3::zeros();

        while !self.finished() {
            let step = self.steps[self.index];
            let start = *self.start.get_or_insert_with(|| capture(world, step));
            let done = match step {
                Step::Caption(text) => {
                    world.caption = text;
                    true
                }
                Step::ShowControls(show) => {
                    world.show_controls = show;
                    true
                }
                Step::Mode(mode) => {
                    world.mode = mode;
                    if mode == Mode::Orbit {
                        world.scene.preview_cubes.clear();
                    }
                    true
                }
                Step::Teleport { position, yaw } => {
                    world.player = Player::new(position, yaw);
                    true
                }
                Step::Jump => {
                    world.player.jump(JUMP_SPEED);
                    true
                }
                Step::Build {
                    kind,
                    rotation_deg,
                    material,
                } => {
                    world.build.enabled = true;
                    world.build.kind = kind;
                    world.build.rotation_deg = rotation_deg;
                    world.build.material = material;
                    true
                }
                Step::BuildOff => {
                    world.build.enabled = false;
                    world.scene.preview_cubes.clear();
                    true
                }
                Step::Place => {
                    let preview =
                        building::refresh_preview(&mut world.scene, &world.player, &world.build);
                    match preview {
                        Some(p) if building::place_preview(&mut world.scene, &p) => {
                            world.placed += 1;
                        }
                        _ => {
                            world.failed_places += 1;
                            eprintln!(
                                "[demo] aviso: no se pudo colocar la pieza (paso {})",
                                self.index
                            );
                        }
                    }
                    true
                }
                Step::Wait(secs) => self.advance_time(dt) >= secs,
                Step::Orbit {
                    secs,
                    d_yaw,
                    pitch,
                    radius,
                } => {
                    let t = ease(self.advance_time(dt) / secs);
                    world.orbit.yaw = start.orbit_yaw + d_yaw * t;
                    world.orbit.pitch = lerp(start.orbit_pitch, pitch, t);
                    world.orbit.radius = lerp(start.orbit_radius, radius, t);
                    t >= 1.0
                }
                Step::Look { secs, .. } | Step::LookAt { secs, .. } => {
                    let t = ease(self.advance_time(dt) / secs);
                    world.player.yaw = start.look_yaw + angle_diff(start.look_yaw, start.target_yaw) * t;
                    world.player.pitch = lerp(start.look_pitch, start.target_pitch, t);
                    t >= 1.0
                }
                Step::WalkTo { x, z, run } => {
                    let elapsed = self.advance_time(dt);
                    let (delta, arrived) = steer(&mut world.player, x, z, run, dt);
                    horizontal = delta;
                    self.track_stuck(world, start, dt);
                    if !arrived && (elapsed > WALK_TIMEOUT || self.stuck_time > 2.0) {
                        let p = world.player.position;
                        eprintln!(
                            "[demo] aviso: el jugador se atoró yendo a ({x}, {z}); quedó en ({:.2}, {:.2}, {:.2})",
                            p.x, p.y, p.z
                        );
                    }
                    arrived || elapsed > WALK_TIMEOUT || self.stuck_time > 2.0
                }
            };

            if done {
                self.index += 1;
                self.elapsed = 0.0;
                self.start = None;
                self.stuck_time = 0.0;
                if is_instant(step) {
                    continue;
                }
            }
            break;
        }

        if world.mode == Mode::FirstPerson {
            collision::move_player(&world.scene, &mut world.player, horizontal, dt);
            if world.build.enabled {
                building::refresh_preview(&mut world.scene, &world.player, &world.build);
            }
        }
    }

    fn advance_time(&mut self, dt: f32) -> f32 {
        self.elapsed += dt;
        self.elapsed
    }

    fn track_stuck(&mut self, world: &World, start: StepStart, dt: f32) {
        // Se compara contra la posición del cuadro anterior guardada en
        // `start.position`, actualizada abajo.
        let moved = (world.player.position - start.position).norm();
        if moved < 0.01 {
            self.stuck_time += dt;
        } else {
            self.stuck_time = 0.0;
        }
        if let Some(s) = self.start.as_mut() {
            s.position = world.player.position;
        }
    }
}

fn is_instant(step: Step) -> bool {
    matches!(
        step,
        Step::Caption(_)
            | Step::ShowControls(_)
            | Step::Mode(_)
            | Step::Teleport { .. }
            | Step::Jump
            | Step::Build { .. }
            | Step::BuildOff
            | Step::Place
    )
}

fn capture(world: &World, step: Step) -> StepStart {
    let (target_yaw, target_pitch) = match step {
        Step::Look { yaw, pitch, .. } => (yaw, pitch),
        Step::LookAt { point, .. } => {
            let dir = point - world.player.eye_position();
            let flat = (dir.x * dir.x + dir.z * dir.z).sqrt();
            (dir.x.atan2(dir.z), dir.y.atan2(flat))
        }
        _ => (world.player.yaw, world.player.pitch),
    };
    StepStart {
        orbit_yaw: world.orbit.yaw,
        orbit_pitch: world.orbit.pitch,
        orbit_radius: world.orbit.radius,
        look_yaw: world.player.yaw,
        look_pitch: world.player.pitch,
        position: world.player.position,
        target_yaw,
        target_pitch,
    }
}

/// Gira al jugador hacia `(x, z)` y devuelve el desplazamiento horizontal de
/// este cuadro; avanza más lento mientras todavía está girando.
fn steer(player: &mut Player, x: f32, z: f32, run: bool, dt: f32) -> (Vec3, bool) {
    let to_target = vec3(x - player.position.x, 0.0, z - player.position.z);
    let distance = to_target.norm();
    if distance < ARRIVE_DISTANCE {
        return (Vec3::zeros(), true);
    }

    let desired_yaw = to_target.x.atan2(to_target.z);
    let diff = angle_diff(player.yaw, desired_yaw);
    let turn = diff.clamp(-TURN_SPEED * dt, TURN_SPEED * dt);
    // La mirada vuelve suavemente al horizonte mientras camina.
    let pitch_step = (-player.pitch).clamp(-0.8 * dt, 0.8 * dt);
    player.look(turn, pitch_step);

    let speed = if run { PLAYER_RUN_SPEED } else { PLAYER_WALK_SPEED };
    let alignment = angle_diff(player.yaw, desired_yaw).cos().max(0.0).powi(2);
    // Sin pasarse del destino en el último cuadro.
    let step_len = (speed * dt * alignment).min(distance);
    let direction = to_target / distance;
    (direction * step_len, false)
}

/// Diferencia angular más corta de `from` a `to`, en `[-PI, PI]`.
fn angle_diff(from: f32, to: f32) -> f32 {
    (to - from + PI).rem_euclid(TAU) - PI
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Suavizado al inicio y al final (smoothstep), con `t` limitado a `[0, 1]`.
fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Corre el guion completo sin renderizar, para conocer la duración (y
/// detectar pasos que fallan) antes de gastar tiempo en el render.
fn dry_run(dt: f32) -> (usize, World) {
    let mut world = World::new();
    let mut director = Director::new(script());
    let mut frames = 0;
    while !director.finished() {
        director.tick(&mut world, dt);
        frames += 1;
    }
    (frames, world)
}

pub fn run(options: &DemoOptions) -> Result<(), Box<dyn Error>> {
    let dt = 1.0 / options.fps as f32;
    let (total_frames, dry_world) = dry_run(dt);
    let duration = total_frames as f32 / options.fps as f32;
    println!(
        "[demo] guion: {total_frames} cuadros ({duration:.1} s) | piezas colocadas: {} | fallidas: {}",
        dry_world.placed, dry_world.failed_places
    );
    drop(dry_world);

    let (width, height) = (options.width, options.height);
    let fade_out_start = (duration - 1.5).max(0.0);
    let audio_fade_start = (duration - 2.5).max(0.0);
    let mut ffmpeg = Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error"])
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24"])
        .args(["-s", &format!("{width}x{height}")])
        .args(["-r", &options.fps.to_string()])
        .args(["-i", "-"])
        .args(["-stream_loop", "-1", "-i", MUSIC_PATH])
        .args(["-map", "0:v:0", "-map", "1:a:0"])
        .args([
            "-vf",
            &format!("fade=t=in:st=0:d=1,fade=t=out:st={fade_out_start:.2}:d=1.5"),
        ])
        .args([
            "-af",
            &format!("afade=t=in:st=0:d=1,afade=t=out:st={audio_fade_start:.2}:d=2.5"),
        ])
        .args(["-c:v", "libx264", "-preset", "slow", "-crf", "18"])
        .args(["-pix_fmt", "yuv420p"])
        .args(["-c:a", "aac", "-b:a", "192k"])
        .args(["-t", &format!("{duration:.3}")])
        .arg(&options.output)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("no se pudo iniciar ffmpeg (¿está instalado y en el PATH?): {e}"))?;
    let mut stdin = ffmpeg.stdin.take().expect("stdin de ffmpeg");

    let mut world = World::new();
    let mut director = Director::new(script());
    let mut framebuffer = Framebuffer::new(width, height);
    let mut rgb = vec![0u8; width * height * 3];
    let aspect = width as f32 / height as f32;
    let started = Instant::now();

    for frame in 0..total_frames {
        director.tick(&mut world, dt);

        let params = world.render_params(aspect);
        renderer::render(&world.scene, &params, &mut framebuffer, MAX_TRACE_DEPTH);
        let pixels = framebuffer.as_mut_slice();
        if let Some(caption) = world.caption {
            hud::draw_caption(pixels, width, height, caption);
        }
        if world.show_controls {
            hud::draw_controls_panel(pixels, width, height);
        }
        hud::draw_hint(pixels, width, height, world.show_controls);

        for (dst, &p) in rgb.chunks_exact_mut(3).zip(framebuffer.as_slice()) {
            dst[0] = (p >> 16) as u8;
            dst[1] = (p >> 8) as u8;
            dst[2] = p as u8;
        }
        stdin.write_all(&rgb)?;

        let done = frame + 1;
        if done % options.fps as usize == 0 || done == total_frames {
            let elapsed = started.elapsed().as_secs_f32();
            let eta = elapsed / done as f32 * (total_frames - done) as f32;
            println!(
                "[demo] {done}/{total_frames} cuadros ({:.0}%) | {:.0} ms/cuadro | faltan ~{:.0} s",
                done as f32 / total_frames as f32 * 100.0,
                elapsed / done as f32 * 1000.0,
                eta
            );
        }
    }

    drop(stdin);
    let status = ffmpeg.wait()?;
    if !status.success() {
        return Err(format!("ffmpeg terminó con error: {status}").into());
    }
    println!("[demo] video listo: {}", options.output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_diff_takes_the_short_way_around() {
        assert!((angle_diff(0.1, TAU - 0.1) + 0.2).abs() < 1e-5);
        assert!((angle_diff(-3.0, 3.0) - (6.0 - TAU)).abs() < 1e-5);
    }

    #[test]
    fn parses_demo_arguments() {
        let args: Vec<String> = ["--demo", "--out", "x.mp4", "--res", "640x360", "--fps", "24"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let opts = DemoOptions::from_args(&args).unwrap();
        assert_eq!(opts.output, "x.mp4");
        assert_eq!((opts.width, opts.height, opts.fps), (640, 360, 24));
    }
}
