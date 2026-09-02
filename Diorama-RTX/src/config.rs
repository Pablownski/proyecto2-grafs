pub const FB_WIDTH: usize = 400;
pub const FB_HEIGHT: usize = 225;
pub const WINDOW_SCALE: usize = 2;

pub const WINDOW_TITLE: &str = "The Agency Raytracing Diorama";

/// Separa los rayos secundarios y de sombra de la superficie que los origina.
pub const EPSILON: f32 = 0.001;

// Cámara orbital (Fase 3).
pub const ORBIT_YAW_SPEED: f32 = 1.6; // rad/s
pub const ORBIT_PITCH_SPEED: f32 = 1.2; // rad/s
pub const ORBIT_ZOOM_SPEED: f32 = 14.0; // unidades/s
pub const ORBIT_SCROLL_ZOOM_FACTOR: f32 = 0.6; // unidades por "notch" de rueda

/// Límite superior de `dt` para evitar saltos grandes tras pausar la ventana.
pub const MAX_DT: f32 = 0.1;

/// Luz ambiental mínima (Fase 5), para evitar negro absoluto en zonas sin luz directa.
pub const AMBIENT_STRENGTH: f32 = 0.06;

/// Límite de rebotes recursivos de reflexión/refracción (Fase 6). Modo de
/// calidad; el modo interactivo reducido llega en la Fase 13.
pub const MAX_TRACE_DEPTH: u32 = 4;

// Primera persona (Fase 10).
pub const PLAYER_WALK_SPEED: f32 = 4.0; // unidades/s
pub const PLAYER_RUN_SPEED: f32 = 7.0; // unidades/s, con Shift
pub const PLAYER_LOOK_YAW_SPEED: f32 = 1.6; // rad/s
pub const PLAYER_LOOK_PITCH_SPEED: f32 = 1.2; // rad/s
pub const PLAYER_EYE_HEIGHT: f32 = 1.6;
pub const PLAYER_FOV_DEGREES: f32 = 70.0;

// Colisiones y salto (Fase 11).
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_RADIUS: f32 = 0.30;
pub const GRAVITY: f32 = -18.0;
pub const JUMP_SPEED: f32 = 7.5;
/// Altura máxima de escalón que el jugador puede subir caminando.
pub const STEP_HEIGHT: f32 = 0.55;
/// Pequeño margen para no quedar exactamente al ras de una superficie.
pub const COLLISION_MARGIN: f32 = 0.01;

// Optimización y calidad adaptativa (Fase 13). Mientras la cámara se mueve
// se renderiza a esta resolución y profundidad reducidas; al quedarse
// quieta un instante corto se refina a `FB_WIDTH`/`FB_HEIGHT`/`MAX_TRACE_DEPTH`.
pub const INTERACTIVE_FB_WIDTH: usize = FB_WIDTH / 2;
pub const INTERACTIVE_FB_HEIGHT: usize = FB_HEIGHT / 2;
pub const INTERACTIVE_TRACE_DEPTH: u32 = 2;
/// Tiempo de quietud antes de volver a renderizar con calidad completa.
pub const IDLE_REFINE_DELAY: f32 = 0.25;
/// Límite de hilos que reparte `std::thread::scope` por franjas horizontales.
pub const MAX_RENDER_THREADS: usize = 8;
