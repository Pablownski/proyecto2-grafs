pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pixels: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
        }
    }

    // Se usa a partir de Fase 13 (redibujo parcial en calidad adaptativa).
    #[allow(dead_code)]
    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    // El render paralelo (Fase 13) escribe directo con `as_mut_slice`; se
    // conserva por su chequeo de límites, cubierto por tests.
    #[allow(dead_code)]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = color;
        }
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.pixels
    }

    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        &mut self.pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_framebuffer_is_black() {
        let fb = Framebuffer::new(4, 3);
        assert_eq!(fb.as_slice().len(), 12);
        assert!(fb.as_slice().iter().all(|&p| p == 0));
    }

    #[test]
    fn clear_fills_every_pixel() {
        let mut fb = Framebuffer::new(3, 3);
        fb.clear(0x00FF00);
        assert!(fb.as_slice().iter().all(|&p| p == 0x00FF00));
    }

    #[test]
    fn set_pixel_writes_expected_index() {
        let mut fb = Framebuffer::new(4, 3);
        fb.set_pixel(2, 1, 0xFF0000);
        let expected_index = fb.width + 2;
        assert_eq!(fb.as_slice()[expected_index], 0xFF0000);
    }

    #[test]
    fn set_pixel_out_of_bounds_is_ignored() {
        let mut fb = Framebuffer::new(2, 2);
        fb.set_pixel(10, 10, 0xFFFFFF);
        assert!(fb.as_slice().iter().all(|&p| p == 0));
    }
}
