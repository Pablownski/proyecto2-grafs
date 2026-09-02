use nalgebra_glm::{Vec3, cross, normalize};
use rayon::prelude::*;

use crate::color::Color;
use crate::config::{AMBIENT_STRENGTH, EPSILON, MAX_TRACE_DEPTH};
use crate::framebuffer::Framebuffer;
use crate::hit::HitRecord;
use crate::material::Material;
use crate::ray::Ray;
use crate::scene::Scene;

/// Cámara mínima y fija usada solo para verificar la Fase 2 (varios cubos a
/// distintas profundidades). Se reemplaza por `OrbitCamera` en la Fase 3.
pub struct RenderParams {
    pub eye: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub fov_y: f32,
    pub aspect: f32,
}

impl RenderParams {
    pub fn look_at(eye: Vec3, target: Vec3, world_up: Vec3, fov_y: f32, aspect: f32) -> Self {
        let forward = normalize(&(target - eye));
        let right = normalize(&cross(&forward, &world_up));
        let up = cross(&right, &forward);
        Self {
            eye,
            forward,
            right,
            up,
            fov_y,
            aspect,
        }
    }
}

fn primary_ray(params: &RenderParams, x: usize, y: usize, width: usize, height: usize) -> Ray {
    let nx = (x as f32 + 0.5) / width as f32 * 2.0 - 1.0;
    let ny = 1.0 - (y as f32 + 0.5) / height as f32 * 2.0;

    let tan_half_fov = (params.fov_y * 0.5).tan();
    let cam_x = nx * params.aspect * tan_half_fov;
    let cam_y = ny * tan_half_fov;

    let direction = normalize(&(params.forward + params.right * cam_x + params.up * cam_y));
    Ray::new(params.eye, direction)
}

pub fn render(scene: &Scene, params: &RenderParams, framebuffer: &mut Framebuffer) {
    let width = framebuffer.width;
    let height = framebuffer.height;

    framebuffer
        .as_mut_slice()
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, pixel) in row.iter_mut().enumerate() {
                let ray = primary_ray(params, x, y, width, height);
                let color = trace(scene, &ray, MAX_TRACE_DEPTH);
                *pixel = color.to_u32();
            }
        });
}

/// Color del cielo cuando un rayo no golpea geometría: el cubemap del skybox
/// si ya está cargado, o un azul plano de reserva mientras no lo esté.
fn background_color(scene: &Scene, direction: Vec3) -> Color {
    match &scene.skybox {
        Some(skybox) => skybox.sample(direction, &scene.textures),
        None => Color::new(0.05, 0.06, 0.10),
    }
}

/// Traza un rayo primario o secundario: impacto más cercano -> sombreado
/// local + reflexión/refracción recursivas hasta `depth == 0`. El skybox
/// también se ve en rayos secundarios (reflejos, refracción), porque todos
/// pasan por esta misma función.
fn trace(scene: &Scene, ray: &Ray, depth: u32) -> Color {
    match scene.closest_hit(ray, EPSILON, f32::INFINITY) {
        Some(hit) => shade(scene, &hit, ray, depth),
        None => background_color(scene, ray.direction),
    }
}

/// Iluminación local (ambiente + difusa Lambert + especular Blinn-Phong, con
/// sombras) más las contribuciones recursivas de reflexión y refracción.
fn shade(scene: &Scene, hit: &HitRecord, ray: &Ray, depth: u32) -> Color {
    let material = match scene.materials.get(hit.material_id) {
        Some(material) => material,
        None => return Color::WHITE,
    };
    let texture = scene.textures.get(material.texture_id);
    let tex_color = texture.sample(hit.uv.x, hit.uv.y);
    let albedo = material.albedo.mul_color(tex_color);
    let view_dir = -ray.direction;

    // La emisión brilla por sí misma (monitores, cofre luminoso) pero no
    // ilumina el resto de la escena: no se agrega ninguna luz por esto.
    let mut color = albedo * AMBIENT_STRENGTH + material.emission;

    for light in &scene.lights {
        let (light_dir, distance_to_light, radiance) = light.sample(hit.point);
        let n_dot_l = hit.normal.dot(&light_dir).max(0.0);
        if n_dot_l <= 0.0 {
            continue;
        }

        // Rayo de sombra: bloqueado solo si el obstáculo está antes de la luz.
        let shadow_origin = hit.point + hit.normal * EPSILON;
        let shadow_ray = Ray::new(shadow_origin, light_dir);
        let shadow_t_max = if distance_to_light.is_finite() {
            distance_to_light - EPSILON
        } else {
            f32::INFINITY
        };
        if scene
            .closest_hit(&shadow_ray, EPSILON, shadow_t_max)
            .is_some()
        {
            continue;
        }

        let diffuse = albedo.mul_color(radiance) * (material.diffuse_weight() * n_dot_l);

        let half_vector = normalize(&(light_dir + view_dir));
        let n_dot_h = hit.normal.dot(&half_vector).max(0.0);
        let spec_strength = n_dot_h.powf(material.shininess);
        let specular = radiance * (material.specular * spec_strength);

        color = color + diffuse + specular;
    }

    if depth == 0 {
        return color.clamp();
    }

    if material.transparency > 0.0 {
        color = color + refractive_contribution(scene, hit, ray, material, depth);
    }

    if material.reflectivity > 0.0 {
        let reflect_dir = reflect(ray.direction, hit.normal);
        let reflect_origin = hit.point + hit.normal * EPSILON;
        let reflect_color = trace(scene, &Ray::new(reflect_origin, reflect_dir), depth - 1);
        color = color + reflect_color * material.reflectivity;
    }

    color.clamp()
}

/// Combina reflexión y refracción (ley de Snell + Schlick) para materiales
/// transparentes. Si hay reflexión interna total, toda la contribución es
/// reflejada.
fn refractive_contribution(
    scene: &Scene,
    hit: &HitRecord,
    ray: &Ray,
    material: &Material,
    depth: u32,
) -> Color {
    let (eta_i, eta_t) = if hit.front_face {
        (1.0, material.refractive_index)
    } else {
        (material.refractive_index, 1.0)
    };
    let eta = eta_i / eta_t;
    let cos_i = (-ray.direction).dot(&hit.normal).clamp(-1.0, 1.0);

    let reflect_dir = reflect(ray.direction, hit.normal);
    let reflect_origin = hit.point + hit.normal * EPSILON;

    match refract(ray.direction, hit.normal, eta) {
        Some(refract_dir) => {
            let kr = schlick(cos_i, eta);
            let refract_origin = hit.point - hit.normal * EPSILON;

            let reflect_color = trace(scene, &Ray::new(reflect_origin, reflect_dir), depth - 1);
            let refract_color = trace(scene, &Ray::new(refract_origin, refract_dir), depth - 1);

            (reflect_color * kr + refract_color * (1.0 - kr)) * material.transparency
        }
        // Reflexión interna total: no hay rayo refractado, todo se refleja.
        None => {
            trace(scene, &Ray::new(reflect_origin, reflect_dir), depth - 1) * material.transparency
        }
    }
}

/// `R = I - 2*dot(I, N)*N`. `normal` debe apuntar contra `incident`.
fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - normal * (2.0 * incident.dot(&normal))
}

/// Ley de Snell en forma vectorial. `normal` debe apuntar contra `incident` y
/// `eta` es la relación `eta_i / eta_t`. `None` indica reflexión interna total.
fn refract(incident: Vec3, normal: Vec3, eta: f32) -> Option<Vec3> {
    let cos_i = (-incident).dot(&normal).clamp(-1.0, 1.0);
    let k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
    if k < 0.0 {
        None
    } else {
        Some(normalize(
            &(incident * eta + normal * (eta * cos_i - k.sqrt())),
        ))
    }
}

/// Aproximación de Schlick para la reflectancia de Fresnel.
/// `cosine` es el coseno del ángulo de incidencia y `eta` la relación `eta_i / eta_t`.
fn schlick(cosine: f32, eta: f32) -> f32 {
    let r0 = ((1.0 - eta) / (1.0 + eta)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_glm::vec3;

    #[test]
    fn reflect_perpendicular_incidence_bounces_straight_back() {
        let incident = vec3(0.0, 0.0, 1.0);
        let normal = vec3(0.0, 0.0, -1.0);
        let r = reflect(incident, normal);
        assert!((r - vec3(0.0, 0.0, -1.0)).norm() < 1e-6);
    }

    #[test]
    fn reflect_grazing_angle_flips_only_normal_component() {
        let incident = vec3(1.0, 0.0, 0.0);
        let normal = vec3(0.0, 1.0, 0.0);
        // El rayo es perpendicular a la normal (dot = 0): no debería cambiar.
        let r = reflect(incident, normal);
        assert!((r - incident).norm() < 1e-6);
    }

    #[test]
    fn refract_perpendicular_incidence_does_not_bend() {
        let incident = vec3(0.0, 0.0, 1.0);
        let normal = vec3(0.0, 0.0, -1.0);
        let eta = 1.0 / 1.5; // aire -> vidrio
        let dir = refract(incident, normal, eta).expect("no debería haber TIR");
        assert!((dir - vec3(0.0, 0.0, 1.0)).norm() < 1e-5);
    }

    #[test]
    fn refract_air_to_glass_bends_towards_normal() {
        // Ángulo de incidencia de 45°, entrando a un medio más denso.
        let incident = normalize(&vec3(1.0, -1.0, 0.0));
        let normal = vec3(0.0, 1.0, 0.0);
        let eta = 1.0 / 1.5;
        let dir = refract(incident, normal, eta).expect("no debería haber TIR");
        // El rayo refractado debe seguir cruzando la superficie (componente Y negativa)
        // pero doblarse hacia la normal: el ángulo respecto a -N es menor que el de incidencia.
        let cos_i = (-incident).dot(&normal);
        let cos_t = (-dir).dot(&normal);
        assert!(dir.y < 0.0);
        assert!(cos_t > cos_i);
    }

    #[test]
    fn refract_glass_to_air_bends_away_from_normal() {
        let incident = normalize(&vec3(0.3, -1.0, 0.0));
        let normal = vec3(0.0, 1.0, 0.0);
        let eta = 1.5 / 1.0; // vidrio -> aire
        let dir = refract(incident, normal, eta).expect("ángulo pequeño, sin TIR");
        let cos_i = (-incident).dot(&normal);
        let cos_t = (-dir).dot(&normal);
        assert!(cos_t < cos_i);
    }

    #[test]
    fn refract_beyond_critical_angle_is_total_internal_reflection() {
        // Rayo casi rasante saliendo de un medio denso: supera el ángulo crítico.
        let incident = normalize(&vec3(0.99, -0.14, 0.0));
        let normal = vec3(0.0, 1.0, 0.0);
        let eta = 1.5 / 1.0;
        assert!(refract(incident, normal, eta).is_none());
    }

    #[test]
    fn schlick_at_normal_incidence_matches_r0() {
        let eta: f32 = 1.0 / 1.5;
        let r0 = ((1.0 - eta) / (1.0 + eta)).powi(2);
        assert!((schlick(1.0, eta) - r0).abs() < 1e-6);
    }

    #[test]
    fn schlick_grows_towards_grazing_angles() {
        let eta = 1.0 / 1.5;
        assert!(schlick(0.1, eta) > schlick(0.9, eta));
    }
}
