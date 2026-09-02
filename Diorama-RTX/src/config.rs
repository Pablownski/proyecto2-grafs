pub const FB_WIDTH: usize = 400;
pub const FB_HEIGHT: usize = 225;
pub const WINDOW_SCALE: usize = 2;

pub const WINDOW_TITLE: &str = "The Agency Raytracing Diorama";

/// Separa los rayos secundarios y de sombra de la superficie que los origina.
pub const EPSILON: f32 = 0.001;

// Cámara orbital (Fase 3).
pub const ORBIT_YAW_SPEED: f32 = 1.6; // rad/s
pub const ORBIT_PITCH_SPEED: f32 = 1.2; // rad/s
pub const ORBIT_ZOOM_SPEED: f32 = 6.0; // unidades/s
pub const ORBIT_SCROLL_ZOOM_FACTOR: f32 = 0.6; // unidades por "notch" de rueda

/// Límite superior de `dt` para evitar saltos grandes tras pausar la ventana.
pub const MAX_DT: f32 = 0.1;

/// Luz ambiental mínima (Fase 5), para evitar negro absoluto en zonas sin luz directa.
pub const AMBIENT_STRENGTH: f32 = 0.06;

/// Límite de rebotes recursivos de reflexión/refracción (Fase 6). Modo de
/// calidad; el modo interactivo reducido llega en la Fase 13.
pub const MAX_TRACE_DEPTH: u32 = 4;
