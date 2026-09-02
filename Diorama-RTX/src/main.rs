mod acceleration;
mod agency;
mod app;
mod building;
mod camera;
mod collision;
mod color;
mod config;
mod cube;
mod framebuffer;
mod hit;
mod light;
mod material;
mod player;
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
