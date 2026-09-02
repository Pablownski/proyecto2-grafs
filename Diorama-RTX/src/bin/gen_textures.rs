//! Herramienta de desarrollo: genera las texturas originales de la Fase 4
//! como PNGs procedurales deterministas (sin extraer nada de terceros).
//! Uso: `cargo run --bin gen_textures`.

use image::{Rgb, RgbImage};

const SIZE: u32 = 128;

/// Ruido por hash determinista (sin dependencias externas de aleatoriedad).
fn hash_noise(x: u32, y: u32, seed: u32) -> f32 {
    let mut n = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(2_246_822_519));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    n ^= n >> 16;
    (n as f32) / (u32::MAX as f32)
}

fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn agency_stone() -> RgbImage {
    let mut img = RgbImage::new(SIZE, SIZE);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let base = 0.86;
            let speckle = (hash_noise(x, y, 11) - 0.5) * 0.12;
            let block_line = if x % 32 < 2 || y % 32 < 2 { -0.08 } else { 0.0 };
            let v = base + speckle + block_line;
            img.put_pixel(x, y, Rgb([to_u8(v), to_u8(v * 0.97), to_u8(v * 0.90)]));
        }
    }
    img
}

fn black_marble() -> RgbImage {
    let mut img = RgbImage::new(SIZE, SIZE);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let fx = x as f32;
            let fy = y as f32;
            let streak = ((fx * 0.15 + fy * 0.35).sin() * 0.5 + 0.5) * 0.25;
            let noise = (hash_noise(x, y, 23) - 0.5) * 0.04;
            let base = 0.10;
            let v = base + streak + noise;
            img.put_pixel(
                x,
                y,
                Rgb([to_u8(v * 0.9), to_u8(v * 0.95), to_u8(v * 1.05)]),
            );
        }
    }
    img
}

fn wood_panel() -> RgbImage {
    let mut img = RgbImage::new(SIZE, SIZE);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let fx = x as f32;
            let grain = ((fx * 0.35).sin() * 0.5 + 0.5) * 0.18;
            let plank = if y % 32 < 2 { -0.10 } else { 0.0 };
            let noise = (hash_noise(x, y, 37) - 0.5) * 0.03;
            let v = 0.42 + grain + plank + noise;
            img.put_pixel(x, y, Rgb([to_u8(v), to_u8(v * 0.55), to_u8(v * 0.30)]));
        }
    }
    img
}

fn glass() -> RgbImage {
    let mut img = RgbImage::new(SIZE, SIZE);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let mullion = x % 64 < 2 || y % 64 < 2;
            let base = if mullion { 0.55 } else { 0.90 };
            let noise = (hash_noise(x, y, 41) - 0.5) * 0.02;
            let v = base + noise;
            img.put_pixel(x, y, Rgb([to_u8(v * 0.92), to_u8(v * 0.98), to_u8(v)]));
        }
    }
    img
}

fn brushed_metal() -> RgbImage {
    let mut img = RgbImage::new(SIZE, SIZE);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let fy = y as f32;
            let brush = ((fy * 1.7).sin() * 0.5 + 0.5) * 0.10;
            let noise = (hash_noise(x, y, 59) - 0.5) * 0.05;
            let v = 0.55 + brush + noise;
            img.put_pixel(x, y, Rgb([to_u8(v * 0.92), to_u8(v * 0.96), to_u8(v)]));
        }
    }
    img
}

// Paleta compartida del skybox: el horizonte de los cuatro muros coincide con
// el borde exterior de las caras top/bottom, así que no hay costuras visibles
// aunque cada cara se genere de forma independiente.
const HORIZON: (f32, f32, f32) = (0.55, 0.42, 0.30);
const ZENITH: (f32, f32, f32) = (0.15, 0.30, 0.55);
const DEEP_ZENITH: (f32, f32, f32) = (0.08, 0.20, 0.45);
const GROUND: (f32, f32, f32) = (0.10, 0.09, 0.08);
const SUN_GLOW: (f32, f32, f32) = (1.0, 0.85, 0.55);

fn lerp3(a: (f32, f32, f32), b: (f32, f32, f32), t: f32) -> (f32, f32, f32) {
    let t = t.clamp(0.0, 1.0);
    (
        a.0 + (b.0 - a.0) * t,
        a.1 + (b.1 - a.1) * t,
        a.2 + (b.2 - a.2) * t,
    )
}

/// Cara lateral del skybox: degradado horizonte (abajo) -> cenit (arriba).
/// La cara `front` además lleva un resplandor solar contenido lejos de los
/// bordes, para no romper la continuidad con las caras vecinas.
fn sky_side(with_sun: bool) -> RgbImage {
    let mut img = RgbImage::new(SIZE, SIZE);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let v = 1.0 - (y as f32 + 0.5) / SIZE as f32;
            let (mut r, mut g, mut b) = lerp3(HORIZON, ZENITH, v);

            if with_sun {
                let u = (x as f32 + 0.5) / SIZE as f32;
                let dx = u - 0.5;
                let dy = v - 0.62;
                let dist = (dx * dx + dy * dy).sqrt();
                let glow = (1.0 - (dist / 0.30).min(1.0)).powf(3.0);
                r += (SUN_GLOW.0 - r) * glow;
                g += (SUN_GLOW.1 - g) * glow;
                b += (SUN_GLOW.2 - b) * glow;
            }

            let noise = (hash_noise(x, y, 71) - 0.5) * 0.015;
            img.put_pixel(
                x,
                y,
                Rgb([to_u8(r + noise), to_u8(g + noise), to_u8(b + noise)]),
            );
        }
    }
    img
}

/// Cara superior: degradado radial cenit profundo (centro) -> cenit de borde
/// (orilla), que coincide con el tope de las caras laterales.
fn sky_top() -> RgbImage {
    let mut img = RgbImage::new(SIZE, SIZE);
    let center = (SIZE as f32 - 1.0) / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = (x as f32 - center) / center;
            let dy = (y as f32 - center) / center;
            let r_dist = (dx * dx + dy * dy).sqrt().min(1.0);
            let (r, g, b) = lerp3(DEEP_ZENITH, ZENITH, r_dist);
            let noise = (hash_noise(x, y, 83) - 0.5) * 0.01;
            img.put_pixel(
                x,
                y,
                Rgb([to_u8(r + noise), to_u8(g + noise), to_u8(b + noise)]),
            );
        }
    }
    img
}

/// Cara inferior: degradado radial suelo (centro) -> horizonte de borde
/// (orilla), que coincide con la base de las caras laterales.
fn sky_bottom() -> RgbImage {
    let mut img = RgbImage::new(SIZE, SIZE);
    let center = (SIZE as f32 - 1.0) / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = (x as f32 - center) / center;
            let dy = (y as f32 - center) / center;
            let r_dist = (dx * dx + dy * dy).sqrt().min(1.0);
            let (r, g, b) = lerp3(GROUND, HORIZON, r_dist);
            let noise = (hash_noise(x, y, 97) - 0.5) * 0.01;
            img.put_pixel(
                x,
                y,
                Rgb([to_u8(r + noise), to_u8(g + noise), to_u8(b + noise)]),
            );
        }
    }
    img
}

fn sky_side_plain() -> RgbImage {
    sky_side(false)
}

fn sky_side_with_sun() -> RgbImage {
    sky_side(true)
}

type Generator = fn() -> RgbImage;

fn main() {
    let dir = "assets/textures";
    std::fs::create_dir_all(dir).expect("failed to create assets/textures");

    let textures: [(&str, Generator); 5] = [
        ("agency_stone.png", agency_stone),
        ("black_marble.png", black_marble),
        ("wood_panel.png", wood_panel),
        ("glass.png", glass),
        ("brushed_metal.png", brushed_metal),
    ];

    for (name, generator) in textures {
        let path = format!("{dir}/{name}");
        generator()
            .save(&path)
            .unwrap_or_else(|e| panic!("failed to save {path}: {e}"));
        println!("generated {path}");
    }

    let skybox_dir = "assets/skybox";
    std::fs::create_dir_all(skybox_dir).expect("failed to create assets/skybox");

    let skybox: [(&str, Generator); 6] = [
        ("right.png", sky_side_plain),
        ("left.png", sky_side_plain),
        ("top.png", sky_top),
        ("bottom.png", sky_bottom),
        ("front.png", sky_side_with_sun),
        ("back.png", sky_side_plain),
    ];

    for (name, generator) in skybox {
        let path = format!("{skybox_dir}/{name}");
        generator()
            .save(&path)
            .unwrap_or_else(|e| panic!("failed to save {path}: {e}"));
        println!("generated {path}");
    }
}
