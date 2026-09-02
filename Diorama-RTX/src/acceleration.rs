//! BVH sobre los cubos estáticos de la escena (sección 13.3 del plan). Las
//! piezas dinámicas y la vista previa de construcción siguen en búsqueda
//! lineal, tal como indica el plan.

use nalgebra_glm::{Vec3, vec3};

use crate::cube::Cube;
use crate::hit::HitRecord;
use crate::ray::Ray;

/// Cantidad de cubos por hoja (entre 4 y 8, sección 13.3).
const LEAF_SIZE: usize = 6;

enum Node {
    Leaf {
        min: Vec3,
        max: Vec3,
        start: usize,
        count: usize,
    },
    Internal {
        min: Vec3,
        max: Vec3,
        left: usize,
        right: usize,
    },
}

fn node_bounds(node: &Node) -> (Vec3, Vec3) {
    match node {
        Node::Leaf { min, max, .. } => (*min, *max),
        Node::Internal { min, max, .. } => (*min, *max),
    }
}

pub struct Bvh {
    nodes: Vec<Node>,
    order: Vec<usize>,
    root: Option<usize>,
}

fn min_vec(a: Vec3, b: Vec3) -> Vec3 {
    vec3(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z))
}

fn max_vec(a: Vec3, b: Vec3) -> Vec3 {
    vec3(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z))
}

fn centroid(cube: &Cube) -> Vec3 {
    (cube.min + cube.max) * 0.5
}

fn bounds_of(cubes: &[Cube], indices: &[usize]) -> (Vec3, Vec3) {
    let mut min = vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for &i in indices {
        min = min_vec(min, cubes[i].min);
        max = max_vec(max, cubes[i].max);
    }
    (min, max)
}

/// Intersección rayo-AABB solo para descartar nodos (sin normal ni UV).
fn aabb_hit(min: Vec3, max: Vec3, ray: &Ray, t_min: f32, t_max: f32) -> bool {
    let mut tmin = t_min;
    let mut tmax = t_max;
    for axis in 0..3 {
        let inv_d = 1.0 / ray.direction[axis];
        let mut t0 = (min[axis] - ray.origin[axis]) * inv_d;
        let mut t1 = (max[axis] - ray.origin[axis]) * inv_d;
        if inv_d < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }
        tmin = tmin.max(t0);
        tmax = tmax.min(t1);
        if tmax < tmin {
            return false;
        }
    }
    true
}

fn build_range(
    cubes: &[Cube],
    order: &mut [usize],
    start: usize,
    end: usize,
    nodes: &mut Vec<Node>,
) -> usize {
    let (min, max) = bounds_of(cubes, &order[start..end]);
    let count = end - start;

    if count <= LEAF_SIZE {
        nodes.push(Node::Leaf {
            min,
            max,
            start,
            count,
        });
        return nodes.len() - 1;
    }

    // Dividir por el eje de mayor extensión de los centroides.
    let mut c_min = vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut c_max = vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for &i in &order[start..end] {
        let c = centroid(&cubes[i]);
        c_min = min_vec(c_min, c);
        c_max = max_vec(c_max, c);
    }
    let extent = c_max - c_min;
    let axis = if extent.x >= extent.y && extent.x >= extent.z {
        0
    } else if extent.y >= extent.z {
        1
    } else {
        2
    };

    order[start..end].sort_by(|&a, &b| {
        let ca = centroid(&cubes[a])[axis];
        let cb = centroid(&cubes[b])[axis];
        ca.partial_cmp(&cb).unwrap()
    });

    let mid = start + count / 2;
    let left = build_range(cubes, order, start, mid, nodes);
    let right = build_range(cubes, order, mid, end, nodes);
    nodes.push(Node::Internal {
        min,
        max,
        left,
        right,
    });
    nodes.len() - 1
}

impl Bvh {
    pub fn build(cubes: &[Cube]) -> Self {
        let mut order: Vec<usize> = (0..cubes.len()).collect();
        let len = order.len();
        let mut nodes = Vec::new();
        let root = if len == 0 {
            None
        } else {
            Some(build_range(cubes, &mut order, 0, len, &mut nodes))
        };
        Bvh { nodes, order, root }
    }

    /// Impacto más cercano contra `cubes` (deben ser los mismos con los que
    /// se llamó a `build`, en el mismo orden).
    pub fn closest_hit(
        &self,
        cubes: &[Cube],
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<HitRecord> {
        let root = self.root?;
        let mut closest = t_max;
        let mut result = None;
        self.visit(cubes, root, ray, t_min, &mut closest, &mut result);
        result
    }

    fn visit(
        &self,
        cubes: &[Cube],
        node_idx: usize,
        ray: &Ray,
        t_min: f32,
        closest: &mut f32,
        result: &mut Option<HitRecord>,
    ) {
        let node = &self.nodes[node_idx];
        let (min, max) = node_bounds(node);
        if !aabb_hit(min, max, ray, t_min, *closest) {
            return;
        }

        match node {
            Node::Leaf { start, count, .. } => {
                for &i in &self.order[*start..*start + *count] {
                    if let Some(hit) = cubes[i].hit(ray, t_min, *closest) {
                        *closest = hit.t;
                        *result = Some(hit);
                    }
                }
            }
            Node::Internal { left, right, .. } => {
                self.visit(cubes, *left, ray, t_min, closest, result);
                self.visit(cubes, *right, ray, t_min, closest, result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_glm::vec3;

    fn linear_closest_hit(cubes: &[Cube], ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut result = None;
        for cube in cubes {
            if let Some(hit) = cube.hit(ray, t_min, closest) {
                closest = hit.t;
                result = Some(hit);
            }
        }
        result
    }

    fn grid_of_cubes(n: i32) -> Vec<Cube> {
        let mut cubes = Vec::new();
        for x in -n..=n {
            for z in -n..=n {
                let fx = x as f32 * 3.0;
                let fz = z as f32 * 3.0;
                cubes.push(Cube::new(
                    vec3(fx - 0.5, -0.5, fz - 0.5),
                    vec3(fx + 0.5, 0.5, fz + 0.5),
                    (x + z) as usize,
                ));
            }
        }
        cubes
    }

    #[test]
    fn matches_linear_search_on_a_grid_of_cubes() {
        let cubes = grid_of_cubes(6); // 13x13 = 169 cubos, fuerza varios niveles.
        let bvh = Bvh::build(&cubes);

        let rays = [
            Ray::new(vec3(0.0, 0.2, -50.0), vec3(0.0, 0.0, 1.0)),
            Ray::new(
                vec3(-20.0, 0.1, -20.0),
                nalgebra_glm::normalize(&vec3(1.0, 0.0, 1.0)),
            ),
            Ray::new(
                vec3(5.0, 5.0, 5.0),
                nalgebra_glm::normalize(&vec3(-1.0, -1.0, -1.0)),
            ),
            Ray::new(vec3(100.0, 100.0, 100.0), vec3(1.0, 0.0, 0.0)), // no golpea nada
        ];

        for ray in rays {
            let expected = linear_closest_hit(&cubes, &ray, 0.001, f32::INFINITY);
            let actual = bvh.closest_hit(&cubes, &ray, 0.001, f32::INFINITY);
            match (expected, actual) {
                (None, None) => {}
                (Some(e), Some(a)) => {
                    assert!((e.t - a.t).abs() < 1e-4);
                    assert_eq!(e.material_id, a.material_id);
                }
                (e, a) => panic!(
                    "resultado distinto: lineal={:?}, bvh={:?}",
                    e.map(|h| h.t),
                    a.map(|h| h.t)
                ),
            }
        }
    }

    #[test]
    fn empty_scene_returns_none() {
        let cubes: Vec<Cube> = Vec::new();
        let bvh = Bvh::build(&cubes);
        let ray = Ray::new(vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, 1.0));
        assert!(
            bvh.closest_hit(&cubes, &ray, 0.001, f32::INFINITY)
                .is_none()
        );
    }

    #[test]
    fn single_cube_is_hit_correctly() {
        let cubes = vec![Cube::new(vec3(-1.0, -1.0, -1.0), vec3(1.0, 1.0, 1.0), 3)];
        let bvh = Bvh::build(&cubes);
        let ray = Ray::new(vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, 1.0));
        let hit = bvh
            .closest_hit(&cubes, &ray, 0.001, f32::INFINITY)
            .expect("debería impactar");
        assert!((hit.t - 4.0).abs() < 1e-4);
        assert_eq!(hit.material_id, 3);
    }

    #[test]
    fn overlapping_cubes_return_the_closest_one() {
        let cubes = vec![
            Cube::new(vec3(-1.0, -1.0, 1.0), vec3(1.0, 1.0, 3.0), 0),
            Cube::new(vec3(-1.0, -1.0, -1.0), vec3(1.0, 1.0, 5.0), 1), // envuelve al anterior
        ];
        let bvh = Bvh::build(&cubes);
        let ray = Ray::new(vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, 1.0));
        let hit = bvh
            .closest_hit(&cubes, &ray, 0.001, f32::INFINITY)
            .expect("debería impactar");
        // El cubo 1 empieza antes (z=-1) que el cubo 0 (z=1) desde este origen.
        assert_eq!(hit.material_id, 1);
    }

    #[test]
    fn ray_that_misses_every_node_returns_none() {
        let cubes = grid_of_cubes(4);
        let bvh = Bvh::build(&cubes);
        let ray = Ray::new(vec3(0.0, 50.0, 0.0), vec3(0.0, 1.0, 0.0)); // se aleja hacia arriba
        assert!(
            bvh.closest_hit(&cubes, &ray, 0.001, f32::INFINITY)
                .is_none()
        );
    }
}
