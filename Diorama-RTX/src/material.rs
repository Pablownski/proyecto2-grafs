use crate::color::Color;
use crate::texture::TextureManager;

pub struct Material {
    // Se usa a partir de Fase 9 (nombres visibles en depuración/README).
    #[allow(dead_code)]
    pub name: String,
    pub texture_id: usize,
    pub albedo: Color,
    pub specular: f32,
    pub shininess: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub refractive_index: f32,
    pub emission: Color,
}

impl Material {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        texture_id: usize,
        albedo: Color,
        specular: f32,
        shininess: f32,
        transparency: f32,
        reflectivity: f32,
        refractive_index: f32,
        emission: Color,
    ) -> Self {
        Self {
            name: name.to_string(),
            texture_id,
            albedo,
            specular,
            shininess,
            transparency: transparency.clamp(0.0, 1.0),
            reflectivity: reflectivity.clamp(0.0, 1.0),
            refractive_index,
            emission,
        }
    }

    /// Peso de la contribución difusa, nunca negativo.
    pub fn diffuse_weight(&self) -> f32 {
        (1.0 - self.reflectivity - self.transparency).max(0.0)
    }
}

/// Tabla central de los cinco materiales principales (sección 8 del plan).
/// Cada uno carga su propia textura y expone sus parámetros ópticos.
pub fn load_core_materials(textures: &mut TextureManager) -> Vec<Material> {
    vec![
        Material::new(
            "Piedra de la agencia",
            textures.load("assets/textures/agency_stone.png"),
            Color::new(0.90, 0.88, 0.82),
            0.18,
            24.0,
            0.00,
            0.04,
            1.00,
            Color::BLACK,
        ),
        Material::new(
            "Mármol negro",
            textures.load("assets/textures/black_marble.png"),
            Color::new(0.12, 0.14, 0.18),
            0.85,
            128.0,
            0.00,
            0.35,
            1.00,
            Color::BLACK,
        ),
        Material::new(
            "Panel de madera",
            textures.load("assets/textures/wood_panel.png"),
            Color::new(0.55, 0.24, 0.10),
            0.25,
            32.0,
            0.00,
            0.05,
            1.00,
            Color::BLACK,
        ),
        Material::new(
            "Vidrio",
            textures.load("assets/textures/glass.png"),
            Color::new(0.85, 0.95, 1.00),
            1.00,
            256.0,
            0.88,
            0.08,
            1.50,
            Color::BLACK,
        ),
        Material::new(
            "Metal cepillado",
            textures.load("assets/textures/brushed_metal.png"),
            Color::new(0.45, 0.50, 0.56),
            1.00,
            192.0,
            0.00,
            0.65,
            1.00,
            Color::BLACK,
        ),
    ]
}

/// Materiales adicionales que enriquecen la escena (sección 8 del plan):
/// agua y césped. No cuentan para el mínimo de cinco materiales calificables,
/// pero también tienen textura y parámetros propios.
pub fn load_extra_materials(textures: &mut TextureManager) -> Vec<Material> {
    vec![
        Material::new(
            "Agua",
            textures.load("assets/textures/water.png"),
            Color::new(0.05, 0.25, 0.36),
            0.90,
            160.0,
            0.45,
            0.35,
            1.333,
            Color::BLACK,
        ),
        Material::new(
            "Césped",
            textures.load("assets/textures/grass.png"),
            Color::new(0.25, 0.65, 0.18),
            0.08,
            8.0,
            0.00,
            0.01,
            1.00,
            Color::BLACK,
        ),
    ]
}

/// Materiales emisivos para el interior (Fase 9): pantallas de monitor y el
/// cofre/dispositivo luminoso. `emission` los hace brillar por sí mismos,
/// pero no reemplaza una luz real que ilumine el resto de la escena.
pub fn load_glow_materials(textures: &mut TextureManager) -> Vec<Material> {
    vec![
        Material::new(
            "Pantalla",
            textures.load("assets/textures/screen.png"),
            Color::new(0.15, 0.20, 0.22),
            0.20,
            16.0,
            0.0,
            0.10,
            1.0,
            Color::new(0.15, 0.85, 0.95),
        ),
        Material::new(
            "Cofre luminoso",
            textures.load("assets/textures/brushed_metal.png"),
            Color::new(0.35, 0.30, 0.15),
            0.60,
            96.0,
            0.0,
            0.30,
            1.0,
            Color::new(1.0, 0.75, 0.20),
        ),
    ]
}

/// Materiales de vista previa para el sistema de construcción (Fase 12):
/// translúcidos (reutilizan la textura de vidrio) para no ocultar del todo
/// la geometría detrás; verde si la colocación es válida, rojo si no.
pub fn load_preview_materials(textures: &mut TextureManager) -> Vec<Material> {
    vec![
        Material::new(
            "Vista previa válida",
            textures.load("assets/textures/glass.png"),
            Color::new(0.25, 0.95, 0.35),
            0.3,
            32.0,
            0.55,
            0.05,
            1.0,
            Color::new(0.05, 0.35, 0.10),
        ),
        Material::new(
            "Vista previa inválida",
            textures.load("assets/textures/glass.png"),
            Color::new(0.95, 0.20, 0.20),
            0.3,
            32.0,
            0.55,
            0.05,
            1.0,
            Color::new(0.35, 0.05, 0.05),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diffuse_weight_never_goes_negative() {
        let m = Material::new(
            "test",
            0,
            Color::WHITE,
            1.0,
            1.0,
            0.7,
            0.6,
            1.0,
            Color::BLACK,
        );
        assert_eq!(m.diffuse_weight(), 0.0);
    }

    #[test]
    fn constructor_clamps_transparency_and_reflectivity() {
        let m = Material::new(
            "test",
            0,
            Color::WHITE,
            1.0,
            1.0,
            1.5,
            -0.5,
            1.0,
            Color::BLACK,
        );
        assert_eq!(m.transparency, 1.0);
        assert_eq!(m.reflectivity, 0.0);
    }

    #[test]
    fn load_core_materials_returns_five_materials_each_with_its_own_texture() {
        let mut textures = TextureManager::new();
        let materials = load_core_materials(&mut textures);
        assert_eq!(materials.len(), 5);

        let mut texture_ids: Vec<usize> = materials.iter().map(|m| m.texture_id).collect();
        texture_ids.sort_unstable();
        texture_ids.dedup();
        assert_eq!(
            texture_ids.len(),
            5,
            "cada material debe usar una textura distinta"
        );
    }
}
