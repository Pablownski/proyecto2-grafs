mod app;
mod color;
mod config;
mod cube;
mod framebuffer;
mod hit;
mod ray;
mod renderer;
mod scene;

use app::App;

fn main() {
    let mut app = App::new();
    app.run();
}
