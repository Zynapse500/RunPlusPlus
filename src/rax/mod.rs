use glium;

use std::time::Instant;

mod window;
use self::window::*;
pub use self::window::{KeyCode, MouseButton};

mod renderer;
pub use self::renderer::Renderer;


pub trait Game {
    /// Creates a new game
    fn new() -> Self;


    /// Updates the game
    fn update(&mut self, dt: f64);


    /// Renders the game to the screen
    fn render(&mut self, renderer: &mut Renderer);


    /// Returns true if the game is running, false otherwise
    fn is_running(&self) -> bool;



    /// Called when the window is closed
    fn on_close(&mut self) {}


    /// Called when a key is pressed
    fn on_key_press(&mut self, key: KeyCode) {}

    /// Called when a key is released
    fn on_key_release(&mut self, key: KeyCode) {}


    /// Called when a mouse button is pressed
    fn on_mouse_press(&mut self, button: MouseButton, x: u64, y: u64) {}

    /// Called when a mouse button is released
    fn on_mouse_release(&mut self, button: MouseButton, x: u64, y: u64) {}

    /// Called when the mouse moved
    fn on_mouse_move(&mut self, x: u64, y: u64) {}
}


impl<T: Game> WindowHandler for T {
    fn closed(&mut self) {
        self.on_close();
    }

    fn key_pressed(&mut self, key: KeyCode) {
        self.on_key_press(key);
    }

    fn key_released(&mut self, key: KeyCode) {
        self.on_key_release(key);
    }

    fn mouse_pressed(&mut self, key: MouseButton, x: u64, y: u64) {
        self.on_mouse_press(key, x, y);
    }

    fn mouse_released(&mut self, key: MouseButton, x: u64, y: u64) {
        self.on_mouse_release(key, x, y);
    }

    fn mouse_moved(&mut self, x: u64, y: u64) {
        self.on_mouse_move(x, y);
    }
}


pub struct GameBuilder {
    window: WindowSettings
}

impl GameBuilder {
    pub fn new() -> GameBuilder {
        GameBuilder {
            window: WindowSettings {
                title: "Rax".to_owned(),
                width: 640,
                height: 360,
                vsync: true,
                samples: 4,
            }
        }
    }


    pub fn run<T>(self)
        where T: Game
    {
        let mut window = Window::new(self.window);
        let mut renderer = Renderer::new(window.get_display());

        let mut game = T::new();

        let mut last = Instant::now();
        while window.is_open() && game.is_running() {
            let now = Instant::now();
            let duration = now - last;
            let delta_time = duration.as_secs() as f64 + 1e-9 * duration.subsec_nanos() as f64;
            window.handle_events(&mut game);

            game.update(delta_time);

            renderer.begin();
            game.render(&mut renderer);
            renderer.end();

            last = now;
        }
    }


    pub fn with_title(self, title: &str) -> Self {
        GameBuilder {
            window: WindowSettings {
                title: title.to_owned(),
                .. self.window
            },
            .. self
        }
    }


    pub fn with_size(self, width: u32, height: u32) -> Self {
        GameBuilder {
            window: WindowSettings {
                width, height,
                .. self.window
            },
            .. self
        }
    }


    pub fn with_vsync(self, vsync: bool) -> Self {
        GameBuilder {
            window: WindowSettings {
                vsync,
                .. self.window
            },
            .. self
        }
    }


    pub fn with_samples(self, samples: u16) -> Self {
        GameBuilder {
            window: WindowSettings {
                samples,
                .. self.window
            },
            .. self
        }
    }
}