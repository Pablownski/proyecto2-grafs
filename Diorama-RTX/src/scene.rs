use crate::acceleration::Bvh;
use crate::cube::Cube;
use crate::hit::HitRecord;
use crate::light::Light;
use crate::material::Material;
use crate::ray::Ray;
use crate::skybox::Skybox;
use crate::texture::TextureManager;

pub struct Scene {
    pub cubes: Vec<Cube>,
    /// Piezas colocadas por el jugador con el sistema de construcción (Fase 12).
    pub dynamic_cubes: Vec<Cube>,
    /// Vista previa de construcción del frame actual (no participa en
    /// sombras ni colisiones, solo en el rayo primario de la cámara).
    pub preview_cubes: Vec<Cube>,
    pub materials: Vec<Material>,
    pub textures: TextureManager,
    pub lights: Vec<Light>,
    pub skybox: Option<Skybox>,
    /// BVH sobre `cubes` (Fase 13). Solo válido mientras `cubes` no cambie;
    /// se reconstruye con `rebuild_static_bvh`. `None` usa búsqueda lineal.
    bvh: Option<Bvh>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            cubes: Vec::new(),
            dynamic_cubes: Vec::new(),
            preview_cubes: Vec::new(),
            materials: Vec::new(),
            textures: TextureManager::new(),
            lights: Vec::new(),
            skybox: None,
            bvh: None,
        }
    }

    /// Construye (o reconstruye) el BVH sobre `cubes`. Debe llamarse después
    /// de terminar de poblar la geometría estática y antes de renderizar;
    /// `dynamic_cubes` y `preview_cubes` no entran al BVH y no lo invalidan.
    pub fn rebuild_static_bvh(&mut self) {
        self.bvh = if self.cubes.is_empty() {
            None
        } else {
            Some(Bvh::build(&self.cubes))
        };
    }

    /// Impacto más cercano contra `cubes` (vía BVH si existe) + `dynamic_cubes`
    /// (búsqueda lineal). Se usa para sombras, reflejos, refracción y
    /// colisiones: la vista previa nunca debe afectarlas.
    pub fn closest_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.closest_hit_impl(ray, t_min, t_max, false)
    }

    /// Igual que `closest_hit`, pero también contra `preview_cubes`. Solo
    /// debe usarse para el rayo primario de la cámara.
    pub fn closest_hit_with_preview(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.closest_hit_impl(ray, t_min, t_max, true)
    }

    fn closest_hit_impl(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        include_preview: bool,
    ) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut result = None;

        let static_hit = match &self.bvh {
            Some(bvh) => bvh.closest_hit(&self.cubes, ray, t_min, closest),
            None => Self::closest_hit_in(self.cubes.iter(), ray, t_min, closest),
        };
        if let Some(hit) = static_hit {
            closest = hit.t;
            result = Some(hit);
        }

        if let Some(hit) = Self::closest_hit_in(self.dynamic_cubes.iter(), ray, t_min, closest) {
            closest = hit.t;
            result = Some(hit);
        }

        if include_preview
            && let Some(hit) = Self::closest_hit_in(self.preview_cubes.iter(), ray, t_min, closest)
        {
            result = Some(hit);
        }

        result
    }

    fn closest_hit_in<'a>(
        cubes: impl Iterator<Item = &'a Cube>,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<HitRecord> {
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

    #[test]
    fn closest_hit_includes_dynamic_cubes() {
        let mut scene = Scene::new();
        scene
            .dynamic_cubes
            .push(Cube::new(vec3(-0.5, -0.5, 1.0), vec3(0.5, 0.5, 2.0), 7));
        let ray = Ray::new(vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, 1.0));
        let hit = scene.closest_hit(&ray, 0.001, f32::INFINITY).unwrap();
        assert_eq!(hit.material_id, 7);
    }

    #[test]
    fn closest_hit_excludes_preview_cubes_but_with_preview_includes_them() {
        let mut scene = Scene::new();
        scene
            .preview_cubes
            .push(Cube::new(vec3(-0.5, -0.5, 1.0), vec3(0.5, 0.5, 2.0), 9));
        let ray = Ray::new(vec3(0.0, 0.0, -5.0), vec3(0.0, 0.0, 1.0));

        assert!(scene.closest_hit(&ray, 0.001, f32::INFINITY).is_none());
        let hit = scene
            .closest_hit_with_preview(&ray, 0.001, f32::INFINITY)
            .unwrap();
        assert_eq!(hit.material_id, 9);
    }

    #[test]
    fn rebuild_static_bvh_matches_linear_search_result() {
        let mut scene = Scene::new();
        for i in 0..40 {
            let x = i as f32 * 2.0;
            scene.cubes.push(Cube::new(
                vec3(x - 0.5, -0.5, -0.5),
                vec3(x + 0.5, 0.5, 0.5),
                i,
            ));
        }
        let ray = Ray::new(vec3(20.0, 0.0, -10.0), vec3(0.0, 0.0, 1.0));

        let without_bvh = scene.closest_hit(&ray, 0.001, f32::INFINITY).unwrap();

        scene.rebuild_static_bvh();
        let with_bvh = scene.closest_hit(&ray, 0.001, f32::INFINITY).unwrap();

        assert_eq!(without_bvh.material_id, with_bvh.material_id);
        assert!((without_bvh.t - with_bvh.t).abs() < 1e-4);
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
