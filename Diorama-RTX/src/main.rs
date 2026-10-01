mod acceleration;
mod agency;
mod app;
mod audio;
mod building;
mod camera;
mod collision;
mod color;
mod config;
mod cube;
mod demo;
mod framebuffer;
mod hit;
mod hud;
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--demo") {
        let result = demo::DemoOptions::from_args(&args)
            .map_err(Into::into)
            .and_then(|options| demo::run(&options));
        if let Err(err) = result {
            eprintln!("[demo] error: {err}");
            std::process::exit(1);
        }
        return;
    }

    let mut app = App::new();
    app.run();
}
