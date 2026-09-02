use crate::cube::Cube;
use crate::hit::HitRecord;
use crate::light::Light;
use crate::material::Material;
use crate::ray::Ray;
use crate::texture::TextureManager;

pub struct Scene {
    pub cubes: Vec<Cube>,
    pub materials: Vec<Material>,
    pub textures: TextureManager,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            cubes: Vec::new(),
            materials: Vec::new(),
            textures: TextureManager::new(),
            lights: Vec::new(),
        }
    }

    /// Impacto más cercano en `[t_min, t_max]`. Independiente del orden de
    /// `cubes`: cada impacto encontrado reduce el límite superior de búsqueda.
    pub fn closest_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut result = None;
        for cube in &self.cubes {
            if let Some(hit) = cube.hit(ray, t_min, closest) {
                closest = hit.t;
                result = Some(hit);
            }
        }
        result
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_glm::vec3;

    #[test]
    fn closest_hit_ignores_insertion_order() {
        let near = Cube::new(vec3(-0.5, -0.5, 1.0), vec3(0.5, 0.5, 2.0), 0);
        let far = Cube::new(vec3(-0.5, -0.5, 5.0), vec3(0.5, 0.5, 6.0), 1);

        let mut scene_a = Scene::new();
        scene_a.cubes.push(near.clone_for_test());
        scene_a.cubes.push(far.clone_for_test());

        let mut scene_b = Scene::new();
        scene_b.cubes.push(far.clone_for_test());
        scene_b.cubes.push(near.clone_for_test());

        let ray = Ray::new(vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, 1.0));

        let hit_a = scene_a.closest_hit(&ray, 0.001, f32::INFINITY).unwrap();
        let hit_b = scene_b.closest_hit(&ray, 0.001, f32::INFINITY).unwrap();

        assert_eq!(hit_a.material_id, 0);
        assert_eq!(hit_b.material_id, 0);
        assert!((hit_a.t - hit_b.t).abs() < 1e-5);
    }

    #[test]
    fn closest_hit_on_empty_scene_is_none() {
        let scene = Scene::new();
        let ray = Ray::new(vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, 1.0));
        assert!(scene.closest_hit(&ray, 0.001, f32::INFINITY).is_none());
    }

    // Pequeño helper de prueba: `Cube` no deriva `Clone` en el diseño final
    // porque las piezas de escena no se duplican en tiempo de ejecución.
    impl Cube {
        fn clone_for_test(&self) -> Cube {
            Cube {
                min: self.min,
                max: self.max,
                material_id: self.material_id,
                uv_scale: self.uv_scale,
                collidable: self.collidable,
                build_piece: self.build_piece,
            }
        }
    }
}
