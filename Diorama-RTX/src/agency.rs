//! Generación procedural del diorama de The Agency. Reinterpretación compacta
//! y original inspirada en la estética general del edificio de Fortnite; no
//! reproduce su geometría exacta ni usa ningún recurso extraído del juego.

use nalgebra_glm::{Vec2, Vec3, normalize, vec2, vec3};

use crate::color::Color;
use crate::cube::Cube;
use crate::light::Light;
use crate::material::{load_core_materials, load_extra_materials};
use crate::scene::Scene;
use crate::skybox::Skybox;

// Índices de material tras cargar `load_core_materials` + `load_extra_materials`.
const STONE: usize = 0;
const MARBLE: usize = 1;
const WOOD: usize = 2;
const GLASS: usize = 3;
const METAL: usize = 4;
const WATER: usize = 5;
const GRASS: usize = 6;

// Niveles del terreno (ver sección 5.2: +Y arriba, plano XZ horizontal).
const WATER_TOP: f32 = -2.0;
const ISLAND_TOP: f32 = -1.5;
const PLAZA_TOP: f32 = -1.2;
const ENTRANCE_TOP: f32 = 0.6;

/// Un mosaico cada `TILE_SIZE` unidades de mundo, para que las texturas no se
/// vean ni demasiado estiradas ni demasiado repetidas en cubos grandes.
const TILE_SIZE: f32 = 2.0;

fn auto_uv_scale(size: Vec3) -> Vec2 {
    let avg = (size.x + size.y + size.z) / 3.0;
    let s = (avg / TILE_SIZE).max(1.0);
    vec2(s, s)
}

/// Agrega un cubo definido por su centro y tamaño (en vez de min/max) y le
/// asigna un `uv_scale` proporcional a su tamaño. Devuelve el índice en
/// `scene.cubes` por si el llamador necesita ajustar el UV manualmente.
fn add_box(scene: &mut Scene, center: Vec3, size: Vec3, material: usize) -> usize {
    let half = size * 0.5;
    let mut cube = Cube::new(center - half, center + half, material);
    cube.uv_scale = auto_uv_scale(size);
    scene.cubes.push(cube);
    scene.cubes.len() - 1
}

/// Escalera de `count` escalones. `step_size` es el tamaño de cada peldaño;
/// `rise` es cuánto sube y avanza cada escalón (incluye componente Y).
fn add_steps(
    scene: &mut Scene,
    base: Vec3,
    count: usize,
    step_size: Vec3,
    rise: Vec3,
    material: usize,
) {
    for i in 0..count {
        let n = i as f32;
        let offset = rise * n;
        let center = vec3(
            base.x + offset.x,
            base.y + offset.y + step_size.y * 0.5,
            base.z + offset.z,
        );
        add_box(scene, center, step_size, material);
    }
}

/// Columnas cuadradas (aproximación de columnas redondas, sección 9 del plan)
/// en cada posición de `bases`, con la misma altura y grosor.
fn add_columns(scene: &mut Scene, bases: &[Vec3], height: f32, thickness: f32, material: usize) {
    for &base in bases {
        add_box(
            scene,
            base + vec3(0.0, height * 0.5, 0.0),
            vec3(thickness, height, thickness),
            material,
        );
    }
}

/// Fila de ventanales de vidrio, ligeramente salidos de la fachada para que
/// no compitan con la piedra en el mismo plano (evita z-fighting visual).
fn add_window_row(
    scene: &mut Scene,
    start: Vec3,
    count: usize,
    spacing: f32,
    size: Vec3,
    material: usize,
) {
    for i in 0..count {
        let center = start + vec3(spacing * i as f32, 0.0, 0.0);
        add_box(scene, center, size, material);
    }
}

/// Torre escalonada/voxelizada: pila de niveles cuadrados que se van
/// angostando con la altura, aproximando una torre curva con cubos.
fn add_voxel_tower(
    scene: &mut Scene,
    base_center: Vec3,
    base_radius: f32,
    height: f32,
    levels: usize,
    material: usize,
) {
    let level_height = height / levels as f32;
    for lvl in 0..levels {
        let t = if levels > 1 {
            lvl as f32 / (levels - 1) as f32
        } else {
            0.0
        };
        let side = (base_radius * (1.0 - 0.35 * t)) * 2.0;
        let y_center = base_center.y + level_height * (lvl as f32 + 0.5);
        // Ligero solape vertical para que no queden costuras entre niveles.
        add_box(
            scene,
            vec3(base_center.x, y_center, base_center.z),
            vec3(side, level_height * 1.02, side),
            material,
        );
    }
}

fn build_water_and_island(scene: &mut Scene) {
    add_box(
        scene,
        vec3(0.0, WATER_TOP - 0.5, 0.0),
        vec3(80.0, 1.0, 80.0),
        WATER,
    );
    add_box(
        scene,
        vec3(0.0, ISLAND_TOP - 0.25, 2.0),
        vec3(48.0, 0.5, 48.0),
        GRASS,
    );
}

fn build_dock(scene: &mut Scene) {
    // Muelle frontal: extiende desde el borde de la isla hacia el agua (-Z).
    add_box(
        scene,
        vec3(0.0, ISLAND_TOP - 0.05, -25.0),
        vec3(4.0, 0.3, 10.0),
        WOOD,
    );
    // Postes del muelle.
    for z in [-20.5, -25.0, -29.5] {
        for x in [-2.2, 2.2] {
            add_box(
                scene,
                vec3(x, ISLAND_TOP + 0.4, z),
                vec3(0.3, 1.6, 0.3),
                WOOD,
            );
        }
    }
}

fn build_plaza_and_stairs(scene: &mut Scene) {
    add_box(
        scene,
        vec3(0.0, PLAZA_TOP - 0.15, 4.0),
        vec3(24.0, 0.3, 16.0),
        STONE,
    );

    // Barandales bajos a los lados de la plaza.
    for x in [-11.7, 11.7] {
        add_box(
            scene,
            vec3(x, PLAZA_TOP + 0.4, 4.0),
            vec3(0.3, 0.8, 15.0),
            MARBLE,
        );
    }

    // Seis escalones ascendiendo desde la plaza hasta la entrada del edificio.
    add_steps(
        scene,
        vec3(0.0, PLAZA_TOP, 12.0),
        6,
        vec3(10.0, 0.3, 0.8),
        vec3(0.0, 0.3, 0.8),
        STONE,
    );

    // Jardineras a los lados de la escalinata.
    for x in [-6.5, 6.5] {
        add_box(
            scene,
            vec3(x, PLAZA_TOP + 0.3, 8.0),
            vec3(1.4, 0.6, 2.0),
            STONE,
        );
        add_box(
            scene,
            vec3(x, PLAZA_TOP + 0.75, 8.0),
            vec3(1.0, 0.5, 1.6),
            GRASS,
        );
    }

    build_sculpture(scene);
}

fn build_sculpture(scene: &mut Scene) {
    // Escultura central reinterpretada: pila de cubos decrecientes.
    add_box(
        scene,
        vec3(0.0, PLAZA_TOP + 0.5, 4.0),
        vec3(2.0, 1.0, 2.0),
        METAL,
    );
    add_box(
        scene,
        vec3(0.0, PLAZA_TOP + 1.4, 4.0),
        vec3(1.4, 0.8, 1.4),
        MARBLE,
    );
    add_box(
        scene,
        vec3(0.2, PLAZA_TOP + 2.1, 3.8),
        vec3(0.8, 0.6, 0.8),
        METAL,
    );
}

fn build_portico_columns(scene: &mut Scene) {
    let bases: Vec<Vec3> = (0..6)
        .map(|i| vec3(-9.0 + i as f32 * 3.6, ENTRANCE_TOP, 15.0))
        .collect();
    add_columns(scene, &bases, 4.0, 0.8, MARBLE);
}

fn build_main_volume(scene: &mut Scene) {
    let central_height = 18.0;
    let central_center = vec3(0.0, ENTRANCE_TOP + central_height * 0.5, 23.0);
    add_box(
        scene,
        central_center,
        vec3(28.0, central_height, 12.0),
        STONE,
    );

    let wing_height = 10.0;
    let wing_center_y = ENTRANCE_TOP + wing_height * 0.5;
    add_box(
        scene,
        vec3(21.0, wing_center_y, 22.0),
        vec3(14.0, wing_height, 10.0),
        STONE,
    );
    add_box(
        scene,
        vec3(-21.0, wing_center_y, 22.0),
        vec3(14.0, wing_height, 10.0),
        STONE,
    );

    // Emblema: rombo de metal centrado en la fachada, sobre la entrada.
    let emblem_z = 17.0 - 0.15; // ligeramente saliente de la fachada.
    add_box(scene, vec3(0.0, 13.0, emblem_z), vec3(1.6, 1.6, 0.3), METAL);
    add_box(scene, vec3(0.0, 14.3, emblem_z), vec3(0.5, 1.0, 0.3), METAL);
    add_box(scene, vec3(0.0, 11.7, emblem_z), vec3(0.5, 1.0, 0.3), METAL);
    add_box(
        scene,
        vec3(-1.3, 13.0, emblem_z),
        vec3(1.0, 0.5, 0.3),
        METAL,
    );
    add_box(scene, vec3(1.3, 13.0, emblem_z), vec3(1.0, 0.5, 0.3), METAL);
}

fn build_windows(scene: &mut Scene) {
    let facade_z = 17.0 - 0.1;
    let window_size = vec3(1.2, 2.0, 0.3);
    for floor in 0..4 {
        let y = 2.5 + floor as f32 * 3.2;
        add_window_row(scene, vec3(-9.6, y, facade_z), 7, 3.2, window_size, GLASS);
    }

    // Ventanales de las alas laterales.
    for &wing_x in &[21.0, -21.0] {
        let wing_facade_z = 17.0 - 0.1;
        for floor in 0..2 {
            let y = 2.5 + floor as f32 * 3.2;
            add_window_row(
                scene,
                vec3(wing_x - 4.8, y, wing_facade_z),
                4,
                3.2,
                window_size,
                GLASS,
            );
        }
    }
}

fn build_towers(scene: &mut Scene) {
    for &x in &[15.5, -15.5] {
        add_voxel_tower(
            scene,
            vec3(x, ENTRANCE_TOP + 18.0, 18.0),
            3.2,
            10.0,
            5,
            STONE,
        );
    }
}

fn build_vegetation(scene: &mut Scene) {
    let tree_positions = [
        vec3(-18.0, 0.0, 12.0),
        vec3(18.0, 0.0, -6.0),
        vec3(-16.0, 0.0, -12.0),
        vec3(15.0, 0.0, 14.0),
    ];
    for pos in tree_positions {
        add_box(
            scene,
            vec3(pos.x, ISLAND_TOP + 0.9, pos.z),
            vec3(0.4, 1.8, 0.4),
            WOOD,
        );
        add_box(
            scene,
            vec3(pos.x, ISLAND_TOP + 2.4, pos.z),
            vec3(2.2, 2.0, 2.2),
            GRASS,
        );
    }

    let rock_positions = [
        vec3(-20.0, 0.0, -2.0),
        vec3(20.0, 0.0, 6.0),
        vec3(10.0, 0.0, -18.0),
    ];
    for pos in rock_positions {
        add_box(
            scene,
            vec3(pos.x, ISLAND_TOP + 0.4, pos.z),
            vec3(1.4, 0.8, 1.2),
            STONE,
        );
    }
}

fn build_lights(scene: &mut Scene) {
    scene.lights.push(Light::Directional {
        direction: normalize(&vec3(-0.35, -0.85, -0.4)),
        color: Color::new(1.0, 0.95, 0.85),
        intensity: 1.1,
    });
    scene.lights.push(Light::Point {
        position: vec3(0.0, 12.0, 10.0),
        color: Color::new(0.6, 0.8, 1.0),
        intensity: 40.0,
    });
}

/// Construye el diorama completo del exterior de The Agency (Fase 8):
/// agua, isla, muelle, plaza con escalinata, escultura, columnata, volumen
/// principal con alas, torres escalonadas, ventanales, emblema y vegetación.
pub fn build(scene: &mut Scene) {
    scene.materials = load_core_materials(&mut scene.textures);
    scene
        .materials
        .extend(load_extra_materials(&mut scene.textures));

    build_water_and_island(scene);
    build_dock(scene);
    build_plaza_and_stairs(scene);
    build_portico_columns(scene);
    build_main_volume(scene);
    build_windows(scene);
    build_towers(scene);
    build_vegetation(scene);
    build_lights(scene);

    scene.skybox = Some(Skybox::load(&mut scene.textures, "assets/skybox"));
}
