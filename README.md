# Diorama RTX — The Agency

Diorama estilo Minecraft renderizado con **ray tracing en CPU**, escrito en Rust desde cero (sin motores ni GPU). La escena recrea *The Agency*: un edificio sobre una isla rodeada de agua, con muelle, plaza con escalinata y escultura, pórtico de columnas, un atrio interior de dos niveles y una oficina de control.

Se puede recorrer con una cámara orbital o en primera persona (caminar, correr, saltar), y además tiene un sistema de construcción para colocar paredes, pisos y rampas dentro del diorama.

## Video

[![Video del diorama](https://img.youtube.com/vi/mHpj259rvkM/hqdefault.jpg)](https://youtu.be/mHpj259rvkM)

▶ **https://youtu.be/mHpj259rvkM**

El video muestra la cámara orbital (rotación y zoom), el recorrido en primera persona, la construcción de piezas y la entrada a la agencia hasta la oficina.

## Características

- **Ray tracing recursivo**: iluminación Blinn-Phong con sombras, reflexión y refracción (ley de Snell + Fresnel con la aproximación de Schlick, incluyendo reflexión interna total).
- **Skybox**: cubemap de 6 caras que se ve directamente y también en reflejos y refracciones.
- **Cámara orbital**: rotación de 360° alrededor del diorama y zoom para acercarse y alejarse.
- **Primera persona**: movimiento con colisiones, gravedad, escalones y salto.
- **Construcción**: piezas (pared, piso, rampa) con vista previa verde/roja, cuadrícula, rotación y validación de colocación.
- **Iluminación**: luz direccional (sol), luces puntuales en el atrio y la oficina, y materiales emisivos (monitores y cofre luminoso).
- **Rendimiento**:
  - BVH para las intersecciones.
  - Render multihilo con reparto dinámico de filas.
  - Poda de rebotes que no se notan en la imagen.
  - Calidad adaptativa: resolución reducida mientras la cámara se mueve y calidad completa cuando se detiene.
- **Interfaz**: panel de controles que se abre y cierra con `M`, y música de fondo en loop.

## Materiales

Cada material tiene su propia textura y sus propios parámetros.

| Material | Textura | Albedo (RGB) | Specular | Transparencia | Reflectividad | Índice de refracción |
|---|---|---|---|---|---|---|
| Piedra de la agencia | `agency_stone.png` | 0.90, 0.88, 0.82 | 0.18 | 0.00 | 0.04 | 1.00 |
| Mármol negro | `black_marble.png` | 0.12, 0.14, 0.18 | 0.70 | 0.00 | 0.18 | 1.00 |
| Panel de madera | `wood_panel.png` | 0.55, 0.24, 0.10 | 0.25 | 0.00 | 0.05 | 1.00 |
| **Vidrio** | `glass.png` | 0.85, 0.95, 1.00 | 1.00 | 0.88 | 0.08 | **1.50** |
| Metal cepillado | `brushed_metal.png` | 0.45, 0.50, 0.56 | 1.00 | 0.00 | 0.65 | 1.00 |
| **Agua** | `water.png` | 0.05, 0.25, 0.36 | 0.90 | 0.45 | 0.35 | **1.333** |
| Césped | `grass.png` | 0.25, 0.65, 0.18 | 0.08 | 0.00 | 0.01 | 1.00 |
| Piso oscuro mate | `matte_floor.png` | 0.12, 0.14, 0.18 | 0.05 | 0.00 | 0.00 | 1.00 |
| Pantalla (emisiva) | `screen.png` | 0.15, 0.20, 0.22 | 0.20 | 0.00 | 0.10 | 1.00 |

- **Refracción**: el vidrio de los ventanales (IOR 1.5) y el agua que rodea la isla (IOR 1.333).
- **Reflexión**: principalmente el metal (la escultura), el agua y el mármol.

## Controles

Presiona `M` dentro del juego para ver este panel en pantalla.

| Tecla | Acción |
|---|---|
| `Tab` | Cambiar entre cámara orbital y primera persona |
| `M` | Mostrar u ocultar el panel de controles |
| `Esc` | Salir |
| **Cámara orbital** | |
| Flechas | Rotar alrededor del diorama |
| `Q` / `E` / rueda del mouse | Acercar / alejar |
| **Primera persona** | |
| `W` `A` `S` `D` | Moverse |
| Flechas | Mirar |
| `Shift` | Correr |
| `Espacio` | Saltar |
| **Construcción** (en primera persona) | |
| `B` | Activar / desactivar |
| `1` / `2` / `3` | Pared / piso / rampa |
| `R` | Rotar pieza |
| `Z` / `X` / `C` | Madera / piedra / metal |
| Clic izquierdo o `Enter` | Colocar |
| Clic derecho o `Supr` | Eliminar |

## Cómo ejecutarlo

Requiere [Rust](https://rustup.rs/) 1.85 o superior (edición 2024).

```bash
cd Diorama-RTX
cargo run --release
```

### Generar el video demo

El proyecto incluye un recorrido automático que renderiza cada cuadro a calidad completa y lo exporta a MP4 con la música de fondo. Requiere tener [ffmpeg](https://ffmpeg.org/) en el `PATH`.

```bash
cargo run --release -- --demo                                # genera demo.mp4 (1280x720, 30 fps)
cargo run --release -- --demo --out video.mp4 --res 1920x1080 --fps 60
```

## Estructura

| Archivo | Contenido |
|---|---|
| `src/renderer.rs` | Trazado de rayos, sombreado, reflexión y refracción |
| `src/acceleration.rs` | BVH de los cubos estáticos |
| `src/material.rs` | Definición de los materiales |
| `src/skybox.rs` | Cubemap del cielo |
| `src/agency.rs` | Construcción de la escena |
| `src/camera.rs` / `src/player.rs` | Cámara orbital y jugador en primera persona |
| `src/collision.rs` | Colisiones, gravedad y escalones |
| `src/building.rs` | Sistema de construcción |
| `src/hud.rs` | Panel de controles y textos en pantalla |
| `src/audio.rs` | Música de fondo |
| `src/demo.rs` | Recorrido automático para el video |
| `src/app.rs` | Ventana, entrada y bucle principal |
