//! Sistema de construcción (Fase 12): piezas, cuadrícula, preview, validación
//! y colocación/eliminación. Ver secciones 10.4 y 11 del plan.

use nalgebra_glm::{Vec3, vec2, vec3};

use crate::agency::{ATRIUM_FRONT_Z, ATRIUM_MAX_X, ATRIUM_MIN_X, OFFICE_BACK_Z};
use crate::collision::{overlaps, player_aabb};
use crate::config::EPSILON;
use crate::cube::Cube;
use crate::player::Player;
use crate::ray::Ray;
use crate::scene::Scene;

pub const MAX_BUILD_DISTANCE: f32 = 8.0;
const GRID_SIZE: f32 = 0.5;
const MAX_PIECES: usize = 100;
/// Desplaza el punto de impacto hacia afuera de la superficie apuntada,
/// antes de ajustarlo a la cuadrícula (paso 3 de la sección 11.2).
const SURFACE_OFFSET: f32 = 0.05;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Wall,
    Floor,
    Ramp,
}

pub struct BuildState {
    pub enabled: bool,
    pub kind: PieceKind,
    pub rotation_deg: i32,
    pub material: usize,
}

impl BuildState {
    pub fn new(default_material: usize) -> Self {
        Self {
            enabled: false,
            kind: PieceKind::Wall,
            rotation_deg: 0,
            material: default_material,
        }
    }

    pub fn rotate(&mut self) {
        self.rotation_deg = (self.rotation_deg + 90).rem_euclid(360);
    }
}

fn snap(v: f32) -> f32 {
    (v / GRID_SIZE).round() * GRID_SIZE
}

/// Ajusta un punto a la cuadrícula de construcción (paso 4 de la sección 11.2).
pub fn snap_to_grid(point: Vec3) -> Vec3 {
    vec3(snap(point.x), snap(point.y), snap(point.z))
}

/// Vector horizontal "hacia adelante" de la pieza según su rotación de 90°.
fn rotation_forward(rotation_deg: i32) -> Vec3 {
    match rotation_deg.rem_euclid(360) {
        90 => vec3(1.0, 0.0, 0.0),
        180 => vec3(0.0, 0.0, -1.0),
        270 => vec3(-1.0, 0.0, 0.0),
        _ => vec3(0.0, 0.0, 1.0),
    }
}

fn make_piece_cube(center: Vec3, size: Vec3, material: usize) -> Cube {
    let half = size * 0.5;
    let mut cube = Cube::new(center - half, center + half, material);
    cube.build_piece = true;
    cube.uv_scale = vec2(1.0, 1.0);
    cube
}

/// Genera los cubos de una pieza (sección 11.1): pared delgada, piso delgado,
/// o rampa de varios escalones ascendiendo en la dirección de `rotation_deg`.
pub fn piece_cubes(kind: PieceKind, base: Vec3, rotation_deg: i32, material: usize) -> Vec<Cube> {
    let forward = rotation_forward(rotation_deg);
    let along_x = forward.x.abs() > 0.5;

    match kind {
        PieceKind::Wall => {
            let size = if along_x {
                vec3(0.2, 3.0, 3.0)
            } else {
                vec3(3.0, 3.0, 0.2)
            };
            vec![make_piece_cube(base + vec3(0.0, 1.5, 0.0), size, material)]
        }
        PieceKind::Floor => {
            vec![make_piece_cube(
                base + vec3(0.0, 0.1, 0.0),
                vec3(3.0, 0.2, 3.0),
                material,
            )]
        }
        PieceKind::Ramp => {
            const STEPS: usize = 8;
            const STEP_FORWARD: f32 = 0.6;
            const STEP_RISE: f32 = 0.25;
            let tread = if along_x {
                vec3(STEP_FORWARD, STEP_RISE, 1.2)
            } else {
                vec3(1.2, STEP_RISE, STEP_FORWARD)
            };

            (0..STEPS)
                .map(|i| {
                    let n = i as f32;
                    let advance = forward * (STEP_FORWARD * (n + 0.5));
                    let rise = STEP_RISE * (n + 0.5);
                    make_piece_cube(base + advance + vec3(0.0, rise, 0.0), tread, material)
                })
                .collect()
        }
    }
}

/// Punto y normal de la superficie apuntada por el centro de la cámara del
/// jugador, dentro de `MAX_BUILD_DISTANCE` (pasos 1-2 de la sección 11.2).
fn find_target(scene: &Scene, player: &Player) -> Option<(Vec3, Vec3)> {
    let ray = Ray::new(player.eye_position(), player.look_direction());
    scene
        .closest_hit(&ray, EPSILON, MAX_BUILD_DISTANCE)
        .map(|hit| (hit.point, hit.normal))
}

/// Índice en `dynamic_cubes` de la pieza apuntada por la cámara, si hay una
/// dentro de `MAX_BUILD_DISTANCE`.
pub fn find_targeted_piece(scene: &Scene, player: &Player) -> Option<usize> {
    let ray = Ray::new(player.eye_position(), player.look_direction());
    let mut closest_t = MAX_BUILD_DISTANCE;
    let mut result = None;
    for (i, cube) in scene.dynamic_cubes.iter().enumerate() {
        if let Some(hit) = cube.hit(&ray, EPSILON, closest_t) {
            closest_t = hit.t;
            result = Some(i);
        }
    }
    result
}

/// La construcción está permitida en la plaza, el muelle y el exterior, pero
/// no dentro del volumen del atrio/oficina (sección 11.3).
fn is_within_build_zone(point: Vec3) -> bool {
    let inside_interior = point.x > ATRIUM_MIN_X - 1.0
        && point.x < ATRIUM_MAX_X + 1.0
        && point.z > ATRIUM_FRONT_Z - 1.0
        && point.z < OFFICE_BACK_Z + 1.0;
    !inside_interior
}

fn piece_center(cube: &Cube) -> Vec3 {
    (cube.min + cube.max) * 0.5
}

/// Valida distancia (ya limitada por `find_target`), zona permitida, que no
/// se solape con el jugador ni con geometría estática, y el máximo de piezas
/// (sección 11.3).
pub fn validate_placement(scene: &Scene, player: &Player, pieces: &[Cube]) -> bool {
    if scene.dynamic_cubes.len() + pieces.len() > MAX_PIECES {
        return false;
    }

    let (player_min, player_max) = player_aabb(player.position);

    for piece in pieces {
        if !is_within_build_zone(piece_center(piece)) {
            return false;
        }
        if overlaps(piece.min, piece.max, player_min, player_max) {
            return false;
        }
        if scene
            .cubes
            .iter()
            .any(|cube| overlaps(piece.min, piece.max, cube.min, cube.max))
        {
            return false;
        }
    }

    true
}

pub struct Preview {
    pub cubes: Vec<Cube>,
    pub valid: bool,
}

/// Calcula la vista previa completa: apunta, desplaza hacia afuera, ajusta a
/// la cuadrícula, genera la pieza rotada y la valida (sección 11.2).
pub fn compute_preview(scene: &Scene, player: &Player, state: &BuildState) -> Option<Preview> {
    let (point, normal) = find_target(scene, player)?;
    let base = snap_to_grid(point + normal * SURFACE_OFFSET);
    let cubes = piece_cubes(state.kind, base, state.rotation_deg, state.material);
    let valid = validate_placement(scene, player, &cubes);
    Some(Preview { cubes, valid })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_to_grid_rounds_to_nearest_grid_point() {
        // Cuadrícula de 0.5: 1.24->1.0, 0.51->0.5, -0.76->-1.0.
        let snapped = snap_to_grid(vec3(1.24, 0.51, -0.76));
        assert!((snapped - vec3(1.0, 0.5, -1.0)).norm() < 1e-6);
    }

    #[test]
    fn wall_rotated_90_degrees_swaps_footprint_axes() {
        let unrotated = &piece_cubes(PieceKind::Wall, Vec3::zeros(), 0, 0)[0];
        let rotated = &piece_cubes(PieceKind::Wall, Vec3::zeros(), 90, 0)[0];

        let size_a = unrotated.max - unrotated.min;
        let size_b = rotated.max - rotated.min;
        assert!((size_a.x - size_b.z).abs() < 1e-6);
        assert!((size_a.z - size_b.x).abs() < 1e-6);
    }

    #[test]
    fn ramp_has_multiple_ascending_steps() {
        let steps = piece_cubes(PieceKind::Ramp, Vec3::zeros(), 0, 0);
        assert!(steps.len() >= 6 && steps.len() <= 10);
        for pair in steps.windows(2) {
            assert!(
                pair[1].min.y > pair[0].min.y,
                "cada escalón debe quedar más alto que el anterior"
            );
        }
    }

    #[test]
    fn placement_inside_the_player_is_invalid() {
        let scene = Scene::new();
        let player = Player::new(vec3(0.0, 0.0, 0.0), 0.0);
        let pieces = piece_cubes(PieceKind::Floor, vec3(0.0, 0.5, 0.0), 0, 0);
        assert!(!validate_placement(&scene, &player, &pieces));
    }

    #[test]
    fn placement_far_from_the_player_and_geometry_is_valid() {
        let scene = Scene::new();
        let player = Player::new(vec3(0.0, -1.2, -3.0), 0.0);
        let pieces = piece_cubes(PieceKind::Floor, vec3(0.0, -1.2, 0.0), 0, 0);
        assert!(validate_placement(&scene, &player, &pieces));
    }

    #[test]
    fn placement_inside_the_atrium_is_invalid() {
        let scene = Scene::new();
        let player = Player::new(vec3(0.0, -1.2, -3.0), 0.0);
        let pieces = piece_cubes(PieceKind::Floor, vec3(0.0, 1.0, 20.0), 0, 0);
        assert!(!validate_placement(&scene, &player, &pieces));
    }

    #[test]
    fn placement_overlapping_static_geometry_is_invalid() {
        let mut scene = Scene::new();
        scene
            .cubes
            .push(Cube::new(vec3(-1.0, -1.0, -1.0), vec3(1.0, 1.0, 1.0), 0));
        let player = Player::new(vec3(10.0, -1.2, 10.0), 0.0);
        let pieces = piece_cubes(PieceKind::Floor, vec3(0.0, 0.0, 0.0), 0, 0);
        assert!(!validate_placement(&scene, &player, &pieces));
    }
}
