use nalgebra_glm::{Vec2, Vec3, vec2};

use crate::config::EPSILON;
use crate::hit::HitRecord;
use crate::ray::Ray;

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material_id: usize,
    pub uv_scale: Vec2,
    // Se usan a partir de Fase 11 (colisiones) y Fase 12 (construcción).
    #[allow(dead_code)]
    pub collidable: bool,
    #[allow(dead_code)]
    pub build_piece: bool,
}

impl Cube {
    pub fn new(min: Vec3, max: Vec3, material_id: usize) -> Self {
        Self {
            min,
            max,
            material_id,
            uv_scale: vec2(1.0, 1.0),
            collidable: true,
            build_piece: false,
        }
    }

    /// Intersección rayo-AABB con el algoritmo *slab*. Sigue los pasos de la
    /// sección 9.1 del plan: intervalos por eje, intercambio si la dirección
    /// es negativa, y elección de la entrada o la salida según si el origen
    /// del rayo está dentro del cubo.
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut tmin = t_min;
        let mut tmax = t_max;

        // Eje y lado (min o max) que produjeron el tmin/tmax vigentes, para
        // poder reconstruir la normal de la cara golpeada sin recalcularla.
        let mut enter_axis = 0usize;
        let mut enter_is_min_face = true;
        let mut exit_axis = 0usize;
        let mut exit_is_min_face = false;

        for axis in 0..3 {
            let origin = ray.origin[axis];
            let dir = ray.direction[axis];
            let bmin = self.min[axis];
            let bmax = self.max[axis];

            if dir.abs() < 1e-8 {
                if origin < bmin || origin > bmax {
                    return None;
                }
                continue;
            }

            let inv_d = 1.0 / dir;
            let mut t_near = (bmin - origin) * inv_d;
            let mut t_far = (bmax - origin) * inv_d;
            let mut near_is_min_face = true;
            if t_near > t_far {
                std::mem::swap(&mut t_near, &mut t_far);
                near_is_min_face = false;
            }

            if t_near > tmin {
                tmin = t_near;
                enter_axis = axis;
                enter_is_min_face = near_is_min_face;
            }
            if t_far < tmax {
                tmax = t_far;
                exit_axis = axis;
                exit_is_min_face = !near_is_min_face;
            }

            if tmax < tmin {
                return None;
            }
        }

        if tmax <= EPSILON {
            return None;
        }

        // Origen fuera del cubo: usar la entrada. Origen dentro: usar la salida.
        let (t_hit, hit_axis, hit_is_min_face) = if tmin > EPSILON {
            (tmin, enter_axis, enter_is_min_face)
        } else {
            (tmax, exit_axis, exit_is_min_face)
        };

        if t_hit < t_min || t_hit > t_max {
            return None;
        }

        let point = ray.at(t_hit);
        let mut outward_normal = Vec3::zeros();
        outward_normal[hit_axis] = if hit_is_min_face { -1.0 } else { 1.0 };

        let uv = self.face_uv(hit_axis, &point);

        Some(HitRecord::new(
            t_hit,
            point,
            outward_normal,
            uv,
            self.material_id,
            ray.direction,
        ))
    }

    /// UV local por cara, según la tabla de la sección 9.2 del plan.
    fn face_uv(&self, axis: usize, point: &Vec3) -> Vec2 {
        let local = point - self.min;
        let (u, v) = match axis {
            0 => (local.z, local.y), // +X / -X
            1 => (local.x, local.z), // +Y / -Y
            _ => (local.x, local.y), // +Z / -Z
        };
        vec2(u * self.uv_scale.x, v * self.uv_scale.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_glm::vec3;

    fn unit_cube() -> Cube {
        Cube::new(vec3(-1.0, -1.0, -1.0), vec3(1.0, 1.0, 1.0), 0)
    }

    #[test]
    fn frontal_hit_on_minus_z_face() {
        let cube = unit_cube();
        let ray = Ray::new(vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, 1.0));
        let hit = cube.hit(&ray, EPSILON, f32::INFINITY).expect("should hit");
        assert!((hit.t - 4.0).abs() < 1e-5);
        assert_eq!(hit.normal, vec3(0.0, 0.0, -1.0));
        assert!(hit.front_face);
    }

    #[test]
    fn lateral_hit_on_plus_x_face() {
        let cube = unit_cube();
        let ray = Ray::new(vec3(5.0, 0.0, 0.0), vec3(-1.0, 0.0, 0.0));
        let hit = cube.hit(&ray, EPSILON, f32::INFINITY).expect("should hit");
        assert!((hit.t - 4.0).abs() < 1e-5);
        assert_eq!(hit.normal, vec3(1.0, 0.0, 0.0));
    }

    #[test]
    fn top_hit_on_plus_y_face() {
        let cube = unit_cube();
        let ray = Ray::new(vec3(0.0, 5.0, 0.0), vec3(0.0, -1.0, 0.0));
        let hit = cube.hit(&ray, EPSILON, f32::INFINITY).expect("should hit");
        assert!((hit.t - 4.0).abs() < 1e-5);
        assert_eq!(hit.normal, vec3(0.0, 1.0, 0.0));
    }

    #[test]
    fn ray_that_misses_returns_none() {
        let cube = unit_cube();
        let ray = Ray::new(vec3(5.0, 5.0, -5.0), vec3(0.0, 0.0, 1.0));
        assert!(cube.hit(&ray, EPSILON, f32::INFINITY).is_none());
    }

    #[test]
    fn ray_parallel_to_a_face_misses() {
        let cube = unit_cube();
        // Paralelo al eje Z, fuera del rango X del cubo.
        let ray = Ray::new(vec3(5.0, 0.0, -5.0), vec3(0.0, 0.0, 1.0));
        assert!(cube.hit(&ray, EPSILON, f32::INFINITY).is_none());
    }

    #[test]
    fn ray_originating_inside_the_cube_hits_exit_face() {
        let cube = unit_cube();
        let ray = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
        let hit = cube
            .hit(&ray, EPSILON, f32::INFINITY)
            .expect("should hit exit face");
        assert!((hit.t - 1.0).abs() < 1e-5);
        // La cara física golpeada es +Z, pero la normal almacenada siempre se
        // orienta contra el rayo entrante; como el rayo sale en +Z, queda -Z.
        assert_eq!(hit.normal, vec3(0.0, 0.0, -1.0));
        assert!(!hit.front_face);
    }

    #[test]
    fn cube_behind_the_camera_is_not_hit() {
        let cube = unit_cube();
        let ray = Ray::new(vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, -1.0));
        assert!(cube.hit(&ray, EPSILON, f32::INFINITY).is_none());
    }
}
