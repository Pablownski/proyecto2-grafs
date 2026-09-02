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
}
