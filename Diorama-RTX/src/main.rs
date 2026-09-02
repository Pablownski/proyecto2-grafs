mod agency;
mod app;
mod camera;
mod color;
mod config;
mod cube;
mod framebuffer;
mod hit;
mod light;
mod material;
mod ray;
mod renderer;
mod scene;
mod skybox;
mod texture;

use app::App;

fn main() {
    let mut app = App::new();
    app.run();
}
