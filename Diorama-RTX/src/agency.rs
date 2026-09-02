//! Generación procedural del diorama de The Agency. Reinterpretación compacta
//! y original inspirada en la estética general del edificio de Fortnite; no
//! reproduce su geometría exacta ni usa ningún recurso extraído del juego.

use nalgebra_glm::{Vec2, Vec3, normalize, vec2, vec3};

use crate::color::Color;
use crate::cube::Cube;
use crate::light::Light;
use crate::material::{
    load_core_materials, load_extra_materials, load_glow_materials, load_preview_materials,
};
use crate::scene::Scene;
use crate::skybox::Skybox;

// Índices de material tras cargar `load_core_materials` + `load_extra_materials`
// + `load_glow_materials`.
pub(crate) const STONE: usize = 0;
const MARBLE: usize = 1;
pub(crate) const WOOD: usize = 2;
const GLASS: usize = 3;
pub(crate) const METAL: usize = 4;
const WATER: usize = 5;
const GRASS: usize = 6;
const SCREEN: usize = 7;
const CHEST: usize = 8;
pub(crate) const PREVIEW_VALID: usize = 9;
pub(crate) const PREVIEW_INVALID: usize = 10;

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
        vec3(100.0, 1.0, 100.0),
        WATER,
    );
    // Isla alargada hacia +Z para que quepan el edificio y la oficina detrás.
    add_box(
        scene,
        vec3(0.0, ISLAND_TOP - 0.25, 6.0),
        vec3(56.0, 0.5, 68.0),
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

/// Podio macizo que rellena el hueco entre la isla (`ISLAND_TOP`) y el nivel
/// de entrada (`ENTRANCE_TOP`) bajo todo el edificio, las columnas y el atrio.
/// Sin esto solo la escalinata sostiene al edificio y se ve flotando.
fn build_podium(scene: &mut Scene) {
    // Debe empezar justo donde termina el último escalón (z=16.4, ver
    // `build_plaza_and_stairs`): si se solapa con la escalinata, el bloque
    // macizo bloquea la colisión del jugador aunque los escalones se vean
    // encima (el jugador choca contra la masa de piedra antes de llegar
    // al peldaño real).
    let podium_start_z = 16.4;
    let podium_end_z = 37.0;
    let height = ENTRANCE_TOP - ISLAND_TOP;
    let size_z = podium_end_z - podium_start_z;
    let center_z = (podium_start_z + podium_end_z) * 0.5;
    add_box(
        scene,
        vec3(0.0, ISLAND_TOP + height * 0.5, center_z),
        vec3(60.0, height, size_z),
        STONE,
    );
}

// Volumen central: en vez de un bloque sólido, es una cáscara hueca (el
// atrio, ver `build_atrium`) con macizos sólidos a los costados y un bloque
// sólido superior, para conservar la silueta exterior de 28x18x12.
pub(crate) const ATRIUM_MIN_X: f32 = -7.0;
pub(crate) const ATRIUM_MAX_X: f32 = 7.0;
pub(crate) const ATRIUM_FRONT_Z: f32 = 17.0;
const ATRIUM_BACK_Z: f32 = 27.0;
const ATRIUM_FLOOR_Y: f32 = ENTRANCE_TOP;
const ATRIUM_CEILING_Y: f32 = ENTRANCE_TOP + 10.0;
const WALL_THICK: f32 = 0.5;
const DOOR_HALF_WIDTH: f32 = 2.0;
const DOOR_TOP_Y: f32 = ENTRANCE_TOP + 3.5;

fn build_main_volume(scene: &mut Scene) {
    // Bloque sólido superior (sobre el atrio), toda la huella del edificio.
    let upper_height = ENTRANCE_TOP + 18.0 - ATRIUM_CEILING_Y;
    add_box(
        scene,
        vec3(0.0, ATRIUM_CEILING_Y + upper_height * 0.5, 23.0),
        vec3(28.0, upper_height, 12.0),
        STONE,
    );

    // Macizos laterales del volumen central (fuera del ancho del atrio).
    let side_height = ATRIUM_CEILING_Y - ATRIUM_FLOOR_Y;
    let side_center_y = ATRIUM_FLOOR_Y + side_height * 0.5;
    add_box(
        scene,
        vec3(-10.5, side_center_y, 23.0),
        vec3(7.0, side_height, 12.0),
        STONE,
    );
    add_box(
        scene,
        vec3(10.5, side_center_y, 23.0),
        vec3(7.0, side_height, 12.0),
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

/// Pared con una puerta: dos segmentos sólidos a los lados de un hueco de
/// `2 * door_half_width`, más un dintel encima hasta el techo. El hueco es
/// geometría real, no una pared invisible: el recorrido queda despejado.
#[allow(clippy::too_many_arguments)]
fn add_wall_with_doorway(
    scene: &mut Scene,
    z: f32,
    min_x: f32,
    max_x: f32,
    floor_y: f32,
    ceiling_y: f32,
    door_half_width: f32,
    door_top_y: f32,
    material: usize,
) {
    let height = ceiling_y - floor_y;
    // El vano de la puerta va de -door_half_width a +door_half_width; el
    // segmento izquierdo debe terminar ahí, no en +door_half_width (eso
    // dejaba la pared sólida, sin hueco real para entrar).
    let left_width = -door_half_width - min_x;
    let left_center_x = min_x + left_width * 0.5;
    add_box(
        scene,
        vec3(left_center_x, floor_y + height * 0.5, z),
        vec3(left_width, height, WALL_THICK),
        material,
    );

    let right_width = max_x - door_half_width;
    let right_center_x = door_half_width + right_width * 0.5;
    add_box(
        scene,
        vec3(right_center_x, floor_y + height * 0.5, z),
        vec3(right_width, height, WALL_THICK),
        material,
    );

    let lintel_height = ceiling_y - door_top_y;
    add_box(
        scene,
        vec3(0.0, door_top_y + lintel_height * 0.5, z),
        vec3(door_half_width * 2.0, lintel_height, WALL_THICK),
        material,
    );
}

/// Atrio interior (Fase 9): piso oscuro pulido, columnas, recepción,
/// balcones en un segundo nivel con escaleras laterales, e iluminación
/// artificial fría. La entrada (frente) y la puerta trasera hacia la oficina
/// son huecos reales: se puede caminar de la plaza a la oficina sin
/// atravesar geometría.
fn build_atrium(scene: &mut Scene) {
    let width = ATRIUM_MAX_X - ATRIUM_MIN_X;
    let depth = ATRIUM_BACK_Z - ATRIUM_FRONT_Z;

    // Piso oscuro pulido (mármol) y techo.
    add_box(
        scene,
        vec3(
            0.0,
            ATRIUM_FLOOR_Y - 0.15,
            (ATRIUM_FRONT_Z + ATRIUM_BACK_Z) * 0.5,
        ),
        vec3(width, 0.3, depth),
        MARBLE,
    );
    add_box(
        scene,
        vec3(
            0.0,
            ATRIUM_CEILING_Y + 0.15,
            (ATRIUM_FRONT_Z + ATRIUM_BACK_Z) * 0.5,
        ),
        vec3(width, 0.3, depth),
        STONE,
    );

    // Pared frontal (entrada desde la plaza) y trasera (puerta a la oficina).
    add_wall_with_doorway(
        scene,
        ATRIUM_FRONT_Z,
        ATRIUM_MIN_X,
        ATRIUM_MAX_X,
        ATRIUM_FLOOR_Y,
        ATRIUM_CEILING_Y,
        DOOR_HALF_WIDTH,
        DOOR_TOP_Y,
        STONE,
    );
    add_wall_with_doorway(
        scene,
        ATRIUM_BACK_Z,
        ATRIUM_MIN_X,
        ATRIUM_MAX_X,
        ATRIUM_FLOOR_Y,
        ATRIUM_CEILING_Y,
        1.5,
        DOOR_TOP_Y,
        STONE,
    );

    // Paredes laterales, sin huecos.
    let side_height = ATRIUM_CEILING_Y - ATRIUM_FLOOR_Y;
    add_box(
        scene,
        vec3(
            ATRIUM_MIN_X,
            ATRIUM_FLOOR_Y + side_height * 0.5,
            (ATRIUM_FRONT_Z + ATRIUM_BACK_Z) * 0.5,
        ),
        vec3(WALL_THICK, side_height, depth),
        STONE,
    );
    add_box(
        scene,
        vec3(
            ATRIUM_MAX_X,
            ATRIUM_FLOOR_Y + side_height * 0.5,
            (ATRIUM_FRONT_Z + ATRIUM_BACK_Z) * 0.5,
        ),
        vec3(WALL_THICK, side_height, depth),
        STONE,
    );

    // Columnas interiores.
    let column_bases = [
        vec3(-4.0, ATRIUM_FLOOR_Y, ATRIUM_FRONT_Z + 3.0),
        vec3(4.0, ATRIUM_FLOOR_Y, ATRIUM_FRONT_Z + 3.0),
        vec3(-4.0, ATRIUM_FLOOR_Y, ATRIUM_FRONT_Z + 7.0),
        vec3(4.0, ATRIUM_FLOOR_Y, ATRIUM_FRONT_Z + 7.0),
    ];
    add_columns(scene, &column_bases, side_height, 0.6, MARBLE);

    // Balcones de segundo nivel a lo largo de las paredes laterales, con
    // barandal, y escaleras laterales que suben desde la planta baja.
    let balcony_y = ATRIUM_FLOOR_Y + side_height * 0.5;
    for &(x_wall, sign) in &[(ATRIUM_MIN_X, 1.0f32), (ATRIUM_MAX_X, -1.0f32)] {
        let ledge_center_x = x_wall + sign * 1.5;
        add_box(
            scene,
            vec3(
                ledge_center_x,
                balcony_y + 0.15,
                (ATRIUM_FRONT_Z + ATRIUM_BACK_Z) * 0.5,
            ),
            vec3(3.0, 0.3, depth - 2.0),
            MARBLE,
        );
        let rail_x = x_wall + sign * 3.0;
        add_box(
            scene,
            vec3(
                rail_x,
                balcony_y + 0.7,
                (ATRIUM_FRONT_Z + ATRIUM_BACK_Z) * 0.5,
            ),
            vec3(0.15, 0.8, depth - 2.0),
            MARBLE,
        );

        add_steps(
            scene,
            vec3(x_wall + sign * 1.5, ATRIUM_FLOOR_Y, ATRIUM_FRONT_Z + 1.2),
            10,
            vec3(1.2, 0.5, 0.8),
            vec3(0.0, 0.5, 0.8),
            STONE,
        );
    }

    // Recepción central, frente a la puerta de la oficina.
    add_box(
        scene,
        vec3(0.0, ATRIUM_FLOOR_Y + 0.5, ATRIUM_BACK_Z - 3.0),
        vec3(4.0, 1.0, 1.2),
        WOOD,
    );

    // Luz artificial fría del atrio. El mármol del piso es muy especular
    // (specular=0.85): con una luz cercana demasiado intensa el brillo se
    // satura por completo y el ruido de la textura queda visible en el
    // clamp (efecto "estática"), así que se mantiene moderada.
    scene.lights.push(Light::Point {
        position: vec3(
            0.0,
            ATRIUM_CEILING_Y - 1.0,
            (ATRIUM_FRONT_Z + ATRIUM_BACK_Z) * 0.5,
        ),
        color: Color::new(0.70, 0.85, 1.0),
        intensity: 55.0,
    });
    scene.lights.push(Light::Point {
        position: vec3(0.0, ATRIUM_CEILING_Y - 1.0, ATRIUM_FRONT_Z + 2.5),
        color: Color::new(0.75, 0.85, 1.0),
        intensity: 22.0,
    });
}

// Oficina de control, justo detrás de la puerta trasera del atrio.
const OFFICE_MIN_X: f32 = -4.0;
const OFFICE_MAX_X: f32 = 4.0;
const OFFICE_FRONT_Z: f32 = ATRIUM_BACK_Z + 2.0; // deja el corredor de la puerta.
pub(crate) const OFFICE_BACK_Z: f32 = OFFICE_FRONT_Z + 7.0;
const OFFICE_FLOOR_Y: f32 = ENTRANCE_TOP;
const OFFICE_CEILING_Y: f32 = ENTRANCE_TOP + 4.0;

/// Oficina de control (Fase 9): escritorios, monitores emisivos, paneles de
/// madera, archiveros, cofre luminoso como punto focal, y un ventanal
/// trasero con vista al agua (la isla se extendió para que quepa completa).
fn build_office(scene: &mut Scene) {
    let width = OFFICE_MAX_X - OFFICE_MIN_X;
    let depth = OFFICE_BACK_Z - OFFICE_FRONT_Z;
    let center_z = (OFFICE_FRONT_Z + OFFICE_BACK_Z) * 0.5;
    let height = OFFICE_CEILING_Y - OFFICE_FLOOR_Y;

    // Corredor corto que conecta la puerta trasera del atrio con la oficina.
    add_box(
        scene,
        vec3(
            0.0,
            OFFICE_FLOOR_Y - 0.15,
            (ATRIUM_BACK_Z + OFFICE_FRONT_Z) * 0.5,
        ),
        vec3(3.0, 0.3, OFFICE_FRONT_Z - ATRIUM_BACK_Z),
        WOOD,
    );

    // Piso de madera y techo.
    add_box(
        scene,
        vec3(0.0, OFFICE_FLOOR_Y - 0.15, center_z),
        vec3(width, 0.3, depth),
        WOOD,
    );
    add_box(
        scene,
        vec3(0.0, OFFICE_CEILING_Y + 0.15, center_z),
        vec3(width, 0.3, depth),
        STONE,
    );

    // Paredes laterales con paneles de madera, y trasera con ventanal.
    add_box(
        scene,
        vec3(OFFICE_MIN_X, OFFICE_FLOOR_Y + height * 0.5, center_z),
        vec3(WALL_THICK, height, depth),
        WOOD,
    );
    add_box(
        scene,
        vec3(OFFICE_MAX_X, OFFICE_FLOOR_Y + height * 0.5, center_z),
        vec3(WALL_THICK, height, depth),
        WOOD,
    );

    add_wall_with_doorway(
        scene,
        OFFICE_BACK_Z,
        OFFICE_MIN_X,
        OFFICE_MAX_X,
        OFFICE_FLOOR_Y,
        OFFICE_CEILING_Y,
        0.1, // hueco casi cerrado: el resto de la pared sostiene el ventanal.
        OFFICE_CEILING_Y,
        STONE,
    );
    // Ventanal trasero con vista al agua, ligeramente saliente de la pared.
    add_box(
        scene,
        vec3(
            0.0,
            OFFICE_FLOOR_Y + height * 0.55,
            OFFICE_BACK_Z + WALL_THICK * 0.5 + 0.05,
        ),
        vec3(width - 1.0, height * 0.6, 0.25),
        GLASS,
    );

    // Escritorios con monitores emisivos.
    for x in [-2.2, 2.2] {
        add_box(
            scene,
            vec3(x, OFFICE_FLOOR_Y + 0.4, OFFICE_FRONT_Z + 1.5),
            vec3(1.6, 0.8, 0.8),
            WOOD,
        );
        add_box(
            scene,
            vec3(x, OFFICE_FLOOR_Y + 1.1, OFFICE_FRONT_Z + 1.1),
            vec3(0.9, 0.6, 0.1),
            SCREEN,
        );
    }

    // Archiveros contra la pared lateral.
    add_box(
        scene,
        vec3(
            OFFICE_MIN_X + 0.5,
            OFFICE_FLOOR_Y + 0.6,
            OFFICE_BACK_Z - 1.0,
        ),
        vec3(0.7, 1.2, 0.5),
        METAL,
    );
    add_box(
        scene,
        vec3(
            OFFICE_MIN_X + 0.5,
            OFFICE_FLOOR_Y + 0.6,
            OFFICE_BACK_Z - 1.7,
        ),
        vec3(0.7, 1.2, 0.5),
        METAL,
    );

    // Cofre luminoso, punto focal de la oficina, con su propia luz dorada.
    let chest_pos = vec3(
        OFFICE_MAX_X - 1.0,
        OFFICE_FLOOR_Y + 0.35,
        OFFICE_BACK_Z - 1.3,
    );
    add_box(scene, chest_pos, vec3(0.9, 0.7, 0.6), CHEST);
    scene.lights.push(Light::Point {
        position: chest_pos + vec3(0.0, 0.8, 0.0),
        color: Color::new(1.0, 0.8, 0.4),
        intensity: 12.0,
    });
}

fn build_windows(scene: &mut Scene) {
    let facade_z = 17.0 - 0.1;
    let window_size = vec3(1.2, 2.0, 0.3);

    // Planta baja: la ventana central de una fila de 7 caería justo en el
    // vano de la puerta (x en [-2, 2]) y lo taparía con vidrio. Se omite esa
    // ventana dividiendo la fila en dos tramos, izquierdo y derecho.
    add_window_row(scene, vec3(-9.6, 2.5, facade_z), 3, 3.2, window_size, GLASS);
    add_window_row(scene, vec3(3.2, 2.5, facade_z), 3, 3.2, window_size, GLASS);

    for floor in 1..4 {
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

/// Construye el diorama completo de The Agency: exterior (Fase 8) e interior
/// (Fase 9) -- agua, isla, muelle, plaza con escalinata, escultura,
/// columnata, volumen principal con alas, torres escalonadas, ventanales,
/// emblema, vegetación, atrio con balcones y escaleras, y oficina de control
/// con monitores emisivos y cofre luminoso.
pub fn build(scene: &mut Scene) {
    scene.materials = load_core_materials(&mut scene.textures);
    scene
        .materials
        .extend(load_extra_materials(&mut scene.textures));
    scene
        .materials
        .extend(load_glow_materials(&mut scene.textures));
    scene
        .materials
        .extend(load_preview_materials(&mut scene.textures));

    build_water_and_island(scene);
    build_dock(scene);
    build_plaza_and_stairs(scene);
    build_portico_columns(scene);
    build_podium(scene);
    build_main_volume(scene);
    build_atrium(scene);
    build_office(scene);
    build_windows(scene);
    build_towers(scene);
    build_vegetation(scene);
    build_lights(scene);

    scene.skybox = Some(Skybox::load(&mut scene.textures, "assets/skybox"));

    // Toda la geometría estática ya está en `scene.cubes`: construir el BVH
    // una sola vez (Fase 13). Las piezas del jugador siguen fuera de él.
    scene.rebuild_static_bvh();
}
