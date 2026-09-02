use nalgebra_glm::{Vec3, cross, normalize};

use crate::color::Color;
use crate::config::EPSILON;
use crate::framebuffer::Framebuffer;
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
                Some(hit) => shade_flat(scene, &hit),
                None => background,
            };
            framebuffer.set_pixel(x, y, color.to_u32());
        }
    }
}

/// Sombreado plano (sin iluminación todavía, ver Fase 5): textura del
/// material modulada por su albedo, para verificar que cada material se
/// muestrea correctamente por cara.
fn shade_flat(scene: &Scene, hit: &crate::hit::HitRecord) -> Color {
    match scene.materials.get(hit.material_id) {
        Some(material) => {
            let texture = scene.textures.get(material.texture_id);
            let tex_color = texture.sample(hit.uv.x, hit.uv.y);
            material.albedo.mul_color(tex_color)
        }
        None => Color::WHITE,
    }
}
