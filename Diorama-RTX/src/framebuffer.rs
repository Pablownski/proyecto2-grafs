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

    /// Copia la imagen a `dst` (`dst_width`x`dst_height`). Si el tamaño no
    /// coincide la agranda/achica con interpolación bilineal: el render
    /// interactivo de baja resolución se ve suave en vez de en bloques.
    pub fn resize_into(&self, dst: &mut [u32], dst_width: usize, dst_height: usize) {
        if self.width == dst_width && self.height == dst_height {
            dst.copy_from_slice(&self.pixels);
            return;
        }

        // Muestreo centrado en el píxel: (x + 0.5) en destino -> origen.
        let sample = |i: usize, dst_len: usize, src_len: usize| {
            let pos = ((i as f32 + 0.5) * src_len as f32 / dst_len as f32 - 0.5)
                .clamp(0.0, (src_len - 1) as f32);
            let i0 = pos as usize;
            (i0, (i0 + 1).min(src_len - 1), pos - i0 as f32)
        };
        let columns: Vec<(usize, usize, f32)> = (0..dst_width)
            .map(|x| sample(x, dst_width, self.width))
            .collect();

        for y in 0..dst_height {
            let (y0, y1, fy) = sample(y, dst_height, self.height);
            let row0 = &self.pixels[y0 * self.width..(y0 + 1) * self.width];
            let row1 = &self.pixels[y1 * self.width..(y1 + 1) * self.width];
            let dst_row = &mut dst[y * dst_width..(y + 1) * dst_width];
            for (out, &(x0, x1, fx)) in dst_row.iter_mut().zip(&columns) {
                let top = lerp_rgb(row0[x0], row0[x1], fx);
                let bottom = lerp_rgb(row1[x0], row1[x1], fx);
                *out = lerp_rgb(top, bottom, fy);
            }
        }
    }
}

/// Interpola dos colores 0RGB canal por canal (`t = 0` -> `a`, `t = 1` -> `b`).
fn lerp_rgb(a: u32, b: u32, t: f32) -> u32 {
    let channel = |shift: u32| {
        let ca = ((a >> shift) & 0xFF) as f32;
        let cb = ((b >> shift) & 0xFF) as f32;
        ((ca + (cb - ca) * t).round() as u32) << shift
    };
    channel(16) | channel(8) | channel(0)
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
    fn resize_into_same_size_copies_exactly() {
        let mut fb = Framebuffer::new(2, 2);
        fb.as_mut_slice().copy_from_slice(&[1, 2, 3, 4]);
        let mut dst = vec![0; 4];
        fb.resize_into(&mut dst, 2, 2);
        assert_eq!(dst, vec![1, 2, 3, 4]);
    }

    #[test]
    fn resize_into_blends_between_neighbours() {
        let mut fb = Framebuffer::new(2, 1);
        fb.as_mut_slice().copy_from_slice(&[0x000000, 0xFFFFFF]);
        let mut dst = vec![0; 4];
        fb.resize_into(&mut dst, 4, 1);
        // Los extremos conservan el color original y el centro queda gris.
        assert_eq!(dst[0], 0x000000);
        assert_eq!(dst[3], 0xFFFFFF);
        let mid = dst[1] & 0xFF;
        assert!(mid > 0x20 && mid < 0x80, "esperaba un gris oscuro, fue {mid:#x}");
    }

    #[test]
    fn set_pixel_out_of_bounds_is_ignored() {
        let mut fb = Framebuffer::new(2, 2);
        fb.set_pixel(10, 10, 0xFFFFFF);
        assert!(fb.as_slice().iter().all(|&p| p == 0));
    }
}
