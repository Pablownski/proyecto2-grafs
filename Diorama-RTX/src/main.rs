mod app;
mod config;
mod framebuffer;

use app::App;

fn main() {
    let mut app = App::new();
    app.run();
}
