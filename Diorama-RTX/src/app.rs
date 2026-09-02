use minifb::{Key, Window, WindowOptions};

use crate::config::{FB_HEIGHT, FB_WIDTH, WINDOW_SCALE, WINDOW_TITLE};
use crate::framebuffer::Framebuffer;

pub struct App {
    window: Window,
    framebuffer: Framebuffer,
}

impl App {
    pub fn new() -> Self {
        let window = Window::new(
            WINDOW_TITLE,
            FB_WIDTH * WINDOW_SCALE,
            FB_HEIGHT * WINDOW_SCALE,
            WindowOptions::default(),
        )
        .expect("failed to create window");

        let framebuffer = Framebuffer::new(FB_WIDTH, FB_HEIGHT);

        Self {
            window,
            framebuffer,
        }
    }

    pub fn run(&mut self) {
        self.framebuffer.clear(0x1A2233);

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.window
                .update_with_buffer(
                    self.framebuffer.as_slice(),
                    self.framebuffer.width,
                    self.framebuffer.height,
                )
                .expect("failed to update window buffer");
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
