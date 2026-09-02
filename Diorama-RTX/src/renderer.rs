use nalgebra_glm::{Vec3, cross, normalize};

use crate::color::Color;
use crate::config::{AMBIENT_STRENGTH, EPSILON};
use crate::framebuffer::Framebuffer;
use crate::hit::HitRecord;
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
    let background = Color::new(0.05, 0.06, 0.10);

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let ray = primary_ray(params, x, y, framebuffer.width, framebuffer.height);
            let color = match scene.closest_hit(&ray, EPSILON, f32::INFINITY) {
                Some(hit) => shade(scene, &hit, -ray.direction),
                None => background,
            };
            framebuffer.set_pixel(x, y, color.to_u32());
        }
    }
}

/// Iluminación local: ambiente + difusa Lambert + especular Blinn-Phong por
/// cada luz, con rayos de sombra. Todavía sin reflexión ni refracción
/// recursivas (Fase 6).
fn shade(scene: &Scene, hit: &HitRecord, view_dir: Vec3) -> Color {
    let material = match scene.materials.get(hit.material_id) {
        Some(material) => material,
        None => return Color::WHITE,
    };
    let texture = scene.textures.get(material.texture_id);
    let tex_color = texture.sample(hit.uv.x, hit.uv.y);
    let albedo = material.albedo.mul_color(tex_color);

    let mut color = albedo * AMBIENT_STRENGTH;

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

    color.clamp()
}
