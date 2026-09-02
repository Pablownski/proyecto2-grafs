use crate::color::Color;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pixels: Vec<Color>,
}

impl Texture {
    pub fn from_path(path: &str) -> image::ImageResult<Self> {
        let img = image::open(path)?.into_rgb8();
        let (width, height) = img.dimensions();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for p in img.pixels() {
            pixels.push(Color::new(
                p[0] as f32 / 255.0,
                p[1] as f32 / 255.0,
                p[2] as f32 / 255.0,
            ));
        }
        Ok(Self {
            width: width as usize,
            height: height as usize,
            pixels,
        })
    }

    /// Muestreo nearest-neighbor. `u`/`v` se repiten con `fract` (vía
    /// `rem_euclid`, que también maneja valores negativos correctamente).
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let uu = u.rem_euclid(1.0);
        let vv = v.rem_euclid(1.0);
        let x = ((uu * self.width as f32) as usize).min(self.width - 1);
        // V=0 corresponde a la parte inferior de la textura, por lo que se
        // invierte para leer filas de imagen (Y crece hacia abajo).
        let y = (((1.0 - vv) * self.height as f32) as usize).min(self.height - 1);
        self.pixels[y * self.width + x]
    }
}

pub struct TextureManager {
    textures: Vec<Texture>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
        }
    }

    /// Carga una textura desde disco una sola vez y devuelve su id.
    pub fn load(&mut self, path: &str) -> usize {
        let texture = Texture::from_path(path)
            .unwrap_or_else(|e| panic!("failed to load texture {path}: {e}"));
        self.textures.push(texture);
        self.textures.len() - 1
    }

    pub fn get(&self, id: usize) -> &Texture {
        &self.textures[id]
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker_texture() -> Texture {
        // 2x2: negro, blanco / blanco, negro.
        let pixels = vec![Color::BLACK, Color::WHITE, Color::WHITE, Color::BLACK];
        Texture {
            width: 2,
            height: 2,
            pixels,
        }
    }

    #[test]
    fn sample_wraps_with_fract_for_values_above_one() {
        let tex = checker_texture();
        let inside = tex.sample(0.25, 0.75);
        let wrapped = tex.sample(1.25, 1.75);
        assert_eq!(inside, wrapped);
    }

    #[test]
    fn sample_handles_negative_uv() {
        let tex = checker_texture();
        let positive = tex.sample(0.25, 0.75);
        let negative = tex.sample(-0.75, -0.25);
        assert_eq!(positive, negative);
    }

    #[test]
    fn sample_picks_expected_texel() {
        let tex = checker_texture();
        // (0.25, 0.75) -> x=0, v alta -> fila superior -> texel (0,0) = BLACK.
        assert_eq!(tex.sample(0.25, 0.75), Color::BLACK);
        // (0.75, 0.75) -> x=1, fila superior -> texel (1,0) = WHITE.
        assert_eq!(tex.sample(0.75, 0.75), Color::WHITE);
    }
}
