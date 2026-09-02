mod app;
mod color;
mod config;
mod framebuffer;
mod ray;

use app::App;

fn main() {
    let mut app = App::new();
    app.run();
}
