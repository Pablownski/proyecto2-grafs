use nalgebra_glm::{Vec3, vec3};

use crate::config::{COLLISION_MARGIN, GRAVITY, PLAYER_HEIGHT, PLAYER_RADIUS, STEP_HEIGHT};
use crate::player::Player;
use crate::scene::Scene;

pub(crate) fn player_aabb(position: Vec3) -> (Vec3, Vec3) {
    let min = vec3(
        position.x - PLAYER_RADIUS,
        position.y,
        position.z - PLAYER_RADIUS,
    );
    let max = vec3(
        position.x + PLAYER_RADIUS,
        position.y + PLAYER_HEIGHT,
        position.z + PLAYER_RADIUS,
    );
    (min, max)
}

pub(crate) fn overlaps(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> bool {
    a_min.x < b_max.x
        && a_max.x > b_min.x
        && a_min.y < b_max.y
        && a_max.y > b_min.y
        && a_min.z < b_max.z
        && a_max.z > b_min.z
}

/// Mueve `position[axis]` por `delta` y resuelve colisiones contra los cubos
/// colisionables de la escena, empujando al jugador fuera de cualquier
/// superposición (sección 10.3 del plan). Deja un pequeño margen para no
/// quedar exactamente al ras de la superficie. Devuelve si hubo colisión.
fn move_axis(scene: &Scene, position: &mut Vec3, axis: usize, delta: f32) -> bool {
    if delta == 0.0 {
        return false;
    }

    position[axis] += delta;
    let (mut min, mut max) = player_aabb(*position);
    let mut collided = false;

    for cube in scene.cubes.iter().chain(scene.dynamic_cubes.iter()) {
        if !cube.collidable {
            continue;
        }
        if overlaps(min, max, cube.min, cube.max) {
            let push = if delta > 0.0 {
                cube.min[axis] - max[axis] - COLLISION_MARGIN
            } else {
                cube.max[axis] - min[axis] + COLLISION_MARGIN
            };
            position[axis] += push;
            min[axis] += push;
            max[axis] += push;
            collided = true;
        }
    }

    collided
}

/// Intenta el movimiento horizontal deseado; si choca y el jugador está en
/// el suelo, reintenta el mismo movimiento levantándolo hasta `STEP_HEIGHT`
/// (para poder subir escalones razonables) y conserva el resultado solo si
/// queda libre de colisiones a esa altura.
fn move_horizontal_with_step(scene: &Scene, player: &mut Player, delta: Vec3) {
    let original = player.position;

    let collided_x = move_axis(scene, &mut player.position, 0, delta.x);
    let collided_z = move_axis(scene, &mut player.position, 2, delta.z);

    if (collided_x || collided_z) && player.grounded {
        let mut candidate = original;
        candidate.y += STEP_HEIGHT;
        let blocked_x = move_axis(scene, &mut candidate, 0, delta.x);
        let blocked_z = move_axis(scene, &mut candidate, 2, delta.z);
        if !blocked_x && !blocked_z {
            player.position = candidate;
        }
    }
}

/// Franja bajo los pies del jugador usada para comprobar si sigue apoyado en
/// el suelo, incluso en frames donde el pequeño margen de colisión evita que
/// el propio movimiento vertical detecte contacto.
const GROUND_PROBE: f32 = 0.05;

fn standing_on_something(scene: &Scene, position: Vec3) -> bool {
    let probe_min = vec3(
        position.x - PLAYER_RADIUS,
        position.y - GROUND_PROBE,
        position.z - PLAYER_RADIUS,
    );
    let probe_max = vec3(
        position.x + PLAYER_RADIUS,
        position.y,
        position.z + PLAYER_RADIUS,
    );
    scene
        .cubes
        .iter()
        .chain(scene.dynamic_cubes.iter())
        .any(|cube| cube.collidable && overlaps(probe_min, probe_max, cube.min, cube.max))
}

/// Integra gravedad y resuelve colisiones eje por eje (X, Z, luego Y). Al
/// aterrizar sobre una superficie fija la velocidad vertical en cero y marca
/// `grounded = true`; en el aire, `grounded` queda en `false`.
pub fn move_player(scene: &Scene, player: &mut Player, horizontal_delta: Vec3, dt: f32) {
    move_horizontal_with_step(scene, player, horizontal_delta);

    player.velocity.y += GRAVITY * dt;
    let vertical_delta = player.velocity.y * dt;
    move_axis(scene, &mut player.position, 1, vertical_delta);

    if player.velocity.y <= 0.0 && standing_on_something(scene, player.position) {
        player.grounded = true;
        player.velocity.y = 0.0;
    } else {
        player.grounded = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::Cube;

    fn scene_with_floor() -> Scene {
        let mut scene = Scene::new();
        scene.cubes.push(Cube::new(
            vec3(-10.0, -1.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            0,
        ));
        scene
    }

    fn scene_with_wall() -> Scene {
        let mut scene = Scene::new();
        scene.cubes.push(Cube::new(
            vec3(-10.0, -1.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            0,
        ));
        scene
            .cubes
            .push(Cube::new(vec3(1.9, 0.0, -10.0), vec3(2.5, 3.0, 10.0), 0));
        scene
    }

    #[test]
    fn player_falls_and_lands_on_the_floor() {
        let scene = scene_with_floor();
        let mut player = Player::new(vec3(0.0, 5.0, 0.0), 0.0);

        for _ in 0..300 {
            move_player(&scene, &mut player, Vec3::zeros(), 1.0 / 60.0);
        }

        assert!(player.grounded);
        assert!((player.position.y - 0.0).abs() < 0.05);
        assert_eq!(player.velocity.y, 0.0);
    }

    #[test]
    fn player_does_not_pass_through_a_wall() {
        let scene = scene_with_wall();
        let mut player = Player::new(vec3(0.0, 0.0, 0.0), 0.0);
        player.grounded = true;

        for _ in 0..120 {
            move_player(&scene, &mut player, vec3(0.3, 0.0, 0.0), 1.0 / 60.0);
        }

        // El muro empieza en x=1.9; con radio 0.30 el jugador no debería
        // pasar de x ~= 1.9 - 0.30.
        assert!(player.position.x < 1.9 - PLAYER_RADIUS + 0.05);
    }

    #[test]
    fn player_collides_with_dynamic_cubes_too() {
        let mut scene = scene_with_floor();
        // Una pieza colocada por el jugador (Fase 12), no geometría estática.
        scene
            .dynamic_cubes
            .push(Cube::new(vec3(1.9, 0.0, -10.0), vec3(2.5, 3.0, 10.0), 0));

        let mut player = Player::new(vec3(0.0, 0.0, 0.0), 0.0);
        player.grounded = true;

        for _ in 0..120 {
            move_player(&scene, &mut player, vec3(0.3, 0.0, 0.0), 1.0 / 60.0);
        }

        assert!(player.position.x < 1.9 - PLAYER_RADIUS + 0.05);
    }

    #[test]
    fn jump_returns_to_the_ground_and_cannot_repeat_in_mid_air() {
        let scene = scene_with_floor();
        let mut player = Player::new(vec3(0.0, 0.0, 0.0), 0.0);
        player.grounded = true;

        player.jump(7.5);
        assert!(!player.grounded);

        // Saltar de nuevo en el aire no debe hacer nada (grounded es falso).
        let velocity_after_first_jump = player.velocity.y;
        player.jump(7.5);
        assert_eq!(player.velocity.y, velocity_after_first_jump);

        let mut left_ground = false;
        for _ in 0..300 {
            move_player(&scene, &mut player, Vec3::zeros(), 1.0 / 60.0);
            if !player.grounded {
                left_ground = true;
            }
        }

        assert!(
            left_ground,
            "el jugador debió estar en el aire durante el salto"
        );
        assert!(player.grounded, "el jugador debió volver a aterrizar");
        assert!((player.position.y - 0.0).abs() < 0.05);
    }
}
