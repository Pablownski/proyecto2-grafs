use nalgebra_glm::{Vec2, Vec3};

pub struct HitRecord {
    pub t: f32,
    // Se usan a partir de Fase 5 (sombras, iluminación) y Fase 4 (muestreo de textura).
    #[allow(dead_code)]
    pub point: Vec3,
    #[allow(dead_code)]
    pub normal: Vec3,
    #[allow(dead_code)]
    pub uv: Vec2,
    pub material_id: usize,
    #[allow(dead_code)]
    pub front_face: bool,
}

impl HitRecord {
    /// `outward_normal` debe estar normalizada y apuntar hacia afuera del sólido.
    /// La normal guardada siempre queda orientada contra el rayo entrante.
    pub fn new(
        t: f32,
        point: Vec3,
        outward_normal: Vec3,
        uv: Vec2,
        material_id: usize,
        ray_direction: Vec3,
    ) -> Self {
        let front_face = ray_direction.dot(&outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self {
            t,
            point,
            normal,
            uv,
            material_id,
            front_face,
        }
    }
}
