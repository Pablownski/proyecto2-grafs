//! Capa de interfaz dibujada encima del render: el aviso "Presiona M" y el
//! panel de controles. minifb no dibuja texto, así que se usa una fuente
//! bitmap 5x7 propia (solo mayúsculas, dígitos y algunos signos).

const GLYPH_W: usize = 5;
const GLYPH_H: usize = 7;
const TEXT_SCALE: usize = 2;
/// Avance horizontal por carácter (glifo + 1 columna de separación).
const CHAR_ADVANCE: usize = (GLYPH_W + 1) * TEXT_SCALE;
const LINE_HEIGHT: usize = (GLYPH_H + 2) * TEXT_SCALE;

const COLOR_TEXT: u32 = 0xF2F2F2;
const COLOR_TITLE: u32 = 0xFFD54A;
const COLOR_SECTION: u32 = 0x5FD3F3;
const COLOR_KEY: u32 = 0xFFD54A;
const COLOR_PANEL: u32 = 0x0B0F16;
const COLOR_BORDER: u32 = 0x5FD3F3;

/// Ancho (en caracteres) de la columna de teclas del panel.
const KEY_COLUMN_CHARS: usize = 18;

enum Line {
    Title(&'static str),
    Section(&'static str),
    Entry(&'static str, &'static str),
}

const CONTROLS: &[Line] = &[
    Line::Title("CONTROLES"),
    Line::Section("GENERAL"),
    Line::Entry("TAB", "CAMARA ORBITAL / 1RA PERSONA"),
    Line::Entry("M", "MOSTRAR / OCULTAR CONTROLES"),
    Line::Entry("ESC", "SALIR"),
    Line::Section("CAMARA ORBITAL"),
    Line::Entry("FLECHAS", "ROTAR ALREDEDOR DEL DIORAMA"),
    Line::Entry("Q / E / RUEDA", "ACERCAR / ALEJAR"),
    Line::Section("PRIMERA PERSONA"),
    Line::Entry("W A S D", "MOVERSE"),
    Line::Entry("FLECHAS", "MIRAR"),
    Line::Entry("SHIFT / ESPACIO", "CORRER / SALTAR"),
    Line::Section("CONSTRUCCION (1RA PERSONA)"),
    Line::Entry("B", "ACTIVAR / DESACTIVAR"),
    Line::Entry("1 / 2 / 3", "PARED / PISO / RAMPA"),
    Line::Entry("R", "ROTAR PIEZA"),
    Line::Entry("Z / X / C", "MADERA / PIEDRA / METAL"),
    Line::Entry("CLIC IZQ / ENTER", "COLOCAR"),
    Line::Entry("CLIC DER / SUPR", "ELIMINAR"),
];

/// Aviso fijo en la esquina inferior izquierda.
pub fn draw_hint(buffer: &mut [u32], width: usize, height: usize, panel_open: bool) {
    let text = if panel_open {
        "PRESIONA M PARA CERRAR"
    } else {
        "PRESIONA M PARA VER LAS TECLAS"
    };
    let pad = 6;
    let box_w = text_width(text) + pad * 2;
    let box_h = GLYPH_H * TEXT_SCALE + pad * 2;
    let x = 10;
    let y = height.saturating_sub(box_h + 10);
    fill_rect_blend(buffer, width, height, x, y, box_w, box_h, COLOR_PANEL, 0.65);
    draw_text(buffer, width, height, x + pad, y + pad, text, COLOR_TEXT);
}

/// Panel centrado con la lista de teclas, como un "mapa" desplegable.
pub fn draw_controls_panel(buffer: &mut [u32], width: usize, height: usize) {
    let pad = 16;
    let content_chars = CONTROLS
        .iter()
        .map(|line| match line {
            Line::Title(t) | Line::Section(t) => t.len(),
            Line::Entry(_, desc) => KEY_COLUMN_CHARS + desc.len(),
        })
        .max()
        .unwrap_or(0);
    let panel_w = content_chars * CHAR_ADVANCE + pad * 2;
    let panel_h = CONTROLS.len() * LINE_HEIGHT + pad * 2;
    let x0 = width.saturating_sub(panel_w) / 2;
    let y0 = height.saturating_sub(panel_h) / 2;

    fill_rect_blend(buffer, width, height, x0, y0, panel_w, panel_h, COLOR_PANEL, 0.85);
    draw_border(buffer, width, height, x0, y0, panel_w, panel_h, COLOR_BORDER);

    let text_x = x0 + pad;
    for (i, line) in CONTROLS.iter().enumerate() {
        let y = y0 + pad + i * LINE_HEIGHT;
        match line {
            Line::Title(t) => {
                let tx = x0 + (panel_w.saturating_sub(text_width(t))) / 2;
                draw_text(buffer, width, height, tx, y, t, COLOR_TITLE);
            }
            Line::Section(t) => draw_text(buffer, width, height, text_x, y, t, COLOR_SECTION),
            Line::Entry(key, desc) => {
                draw_text(buffer, width, height, text_x, y, key, COLOR_KEY);
                let desc_x = text_x + KEY_COLUMN_CHARS * CHAR_ADVANCE;
                draw_text(buffer, width, height, desc_x, y, desc, COLOR_TEXT);
            }
        }
    }
}

fn text_width(text: &str) -> usize {
    (text.chars().count() * CHAR_ADVANCE).saturating_sub(TEXT_SCALE)
}

fn draw_text(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    text: &str,
    color: u32,
) {
    for (i, ch) in text.chars().enumerate() {
        let rows = glyph(ch.to_ascii_uppercase());
        let gx = x + i * CHAR_ADVANCE;
        for (row, bits) in rows.iter().enumerate() {
            for col in 0..GLYPH_W {
                if bits & (1 << (GLYPH_W - 1 - col)) == 0 {
                    continue;
                }
                for sy in 0..TEXT_SCALE {
                    for sx in 0..TEXT_SCALE {
                        let px = gx + col * TEXT_SCALE + sx;
                        let py = y + row * TEXT_SCALE + sy;
                        if px < width && py < height {
                            buffer[py * width + px] = color;
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fill_rect_blend(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: u32,
    alpha: f32,
) {
    for py in y..(y + h).min(height) {
        for px in x..(x + w).min(width) {
            let idx = py * width + px;
            buffer[idx] = blend(buffer[idx], color, alpha);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_border(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: u32,
) {
    const THICKNESS: usize = 2;
    for py in y..(y + h).min(height) {
        for px in x..(x + w).min(width) {
            let on_edge = px < x + THICKNESS
                || px >= x + w - THICKNESS
                || py < y + THICKNESS
                || py >= y + h - THICKNESS;
            if on_edge {
                buffer[py * width + px] = color;
            }
        }
    }
}

/// Mezcla dos colores 0RGB: `alpha` es el peso de `top`.
fn blend(base: u32, top: u32, alpha: f32) -> u32 {
    let mix = |shift: u32| {
        let b = ((base >> shift) & 0xFF) as f32;
        let t = ((top >> shift) & 0xFF) as f32;
        ((b * (1.0 - alpha) + t * alpha).round() as u32).min(255) << shift
    };
    mix(16) | mix(8) | mix(0)
}

/// Filas del glifo 5x7; el bit 4 es la columna izquierda. Los caracteres
/// sin glifo se dibujan como espacio.
fn glyph(ch: char) -> [u8; GLYPH_H] {
    match ch {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        ':' => [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000],
        '/' => [0b00000, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        ',' => [0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b00100, 0b01000],
        '(' => [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
        '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
        '?' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100],
        _ => [0; GLYPH_H],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FB_HEIGHT, FB_WIDTH};

    #[test]
    fn blend_extremes_return_each_color() {
        assert_eq!(blend(0x102030, 0xA0B0C0, 0.0), 0x102030);
        assert_eq!(blend(0x102030, 0xA0B0C0, 1.0), 0xA0B0C0);
    }

    #[test]
    fn controls_panel_fits_inside_framebuffer() {
        let mut buffer = vec![0u32; FB_WIDTH * FB_HEIGHT];
        draw_controls_panel(&mut buffer, FB_WIDTH, FB_HEIGHT);
        // El borde superior izquierdo debe quedar dentro de la pantalla.
        assert!(buffer.iter().any(|&p| p == COLOR_BORDER));
        assert!(buffer[0] != COLOR_BORDER, "el panel no debe tocar la esquina");
    }

    #[test]
    fn drawing_on_tiny_buffer_does_not_panic() {
        let mut buffer = vec![0u32; 10 * 10];
        draw_controls_panel(&mut buffer, 10, 10);
        draw_hint(&mut buffer, 10, 10, false);
    }
}
