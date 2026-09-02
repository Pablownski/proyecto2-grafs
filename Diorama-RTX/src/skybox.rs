use nalgebra_glm::Vec3;

use crate::color::Color;
use crate::texture::TextureManager;

pub struct Skybox {
    right: usize,
    left: usize,
    top: usize,
    bottom: usize,
    front: usize,
    back: usize,
}

impl Skybox {
    /// Carga las seis caras desde `<dir>/{right,left,top,bottom,front,back}.png`.
    pub fn load(textures: &mut TextureManager, dir: &str) -> Self {
        Self {
            right: textures.load(&format!("{dir}/right.png")),
            left: textures.load(&format!("{dir}/left.png")),
            top: textures.load(&format!("{dir}/top.png")),
            bottom: textures.load(&format!("{dir}/bottom.png")),
            front: textures.load(&format!("{dir}/front.png")),
            back: textures.load(&format!("{dir}/back.png")),
        }
    }

    /// Color del cielo en la dirección dada (sección 9.8 del plan).
    pub fn sample(&self, direction: Vec3, textures: &TextureManager) -> Color {
        let (texture_id, u, v) = self.face_uv(direction);
        textures.get(texture_id).sample(u, v)
    }

    /// Identifica la componente absoluta dominante de `direction`, elige la
    /// cara correspondiente y calcula el UV a partir de las otras dos
    /// componentes, orientado para que cada cara coincida con sus vecinas.
    fn face_uv(&self, direction: Vec3) -> (usize, f32, f32) {
        let ax = direction.x.abs();
        let ay = direction.y.abs();
        let az = direction.z.abs();

        let (texture_id, u, v) = if ax >= ay && ax >= az {
            if direction.x > 0.0 {
                (self.right, -direction.z / ax, -direction.y / ax)
            } else {
                (self.left, direction.z / ax, -direction.y / ax)
            }
        } else if ay >= ax && ay >= az {
            if direction.y > 0.0 {
                (self.top, direction.x / ay, direction.z / ay)
            } else {
                (self.bottom, direction.x / ay, -direction.z / ay)
            }
        } else if direction.z > 0.0 {
            (self.front, direction.x / az, -direction.y / az)
        } else {
            (self.back, -direction.x / az, -direction.y / az)
        };

        (texture_id, (u + 1.0) * 0.5, (v + 1.0) * 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_glm::vec3;

    fn test_skybox() -> Skybox {
        Skybox {
            right: 0,
            left: 1,
            top: 2,
            bottom: 3,
            front: 4,
            back: 5,
        }
    }

    #[test]
    fn picks_right_face_for_dominant_positive_x() {
        let sb = test_skybox();
        let (id, u, v) = sb.face_uv(vec3(1.0, 0.2, -0.1));
        assert_eq!(id, sb.right);
        assert!((0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v));
    }

    #[test]
    fn picks_left_face_for_dominant_negative_x() {
        let sb = test_skybox();
        let (id, _, _) = sb.face_uv(vec3(-1.0, 0.2, -0.1));
        assert_eq!(id, sb.left);
    }

    #[test]
    fn picks_top_face_for_dominant_positive_y() {
        let sb = test_skybox();
        let (id, _, _) = sb.face_uv(vec3(0.1, 1.0, -0.1));
        assert_eq!(id, sb.top);
    }

    #[test]
    fn picks_bottom_face_for_dominant_negative_y() {
        let sb = test_skybox();
        let (id, _, _) = sb.face_uv(vec3(0.1, -1.0, -0.1));
        assert_eq!(id, sb.bottom);
    }

    #[test]
    fn picks_front_face_for_dominant_positive_z() {
        let sb = test_skybox();
        let (id, _, _) = sb.face_uv(vec3(0.1, 0.1, 1.0));
        assert_eq!(id, sb.front);
    }

    #[test]
    fn picks_back_face_for_dominant_negative_z() {
        let sb = test_skybox();
        let (id, _, _) = sb.face_uv(vec3(0.1, 0.1, -1.0));
        assert_eq!(id, sb.back);
    }

    #[test]
    fn axis_aligned_direction_maps_to_face_center() {
        let sb = test_skybox();
        let (_, u, v) = sb.face_uv(vec3(0.0, 0.0, 1.0));
        assert!((u - 0.5).abs() < 1e-6);
        assert!((v - 0.5).abs() < 1e-6);
    }
}
