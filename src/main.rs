
// TODO: Fix normals inside slanted tiles       - Might be fixed with specifying ranges for each edge

// TODO: Add better level editing support

// TODO: Add exit
// TODO: Add "traps"        - bombs, static lasers, etc.
// TODO: Add coins
// TODO: Add level timer
// TODO: Add enemies        - rocket launchers, targeting lasers, chasing orbs, etc.
// TODO: Add buttons        - to activate doors
// TODO: Add obstacles      - doors


const TILE_SIZE: f64 = 48.0;


#[macro_use]
extern crate glium;
extern crate trap;

#[allow(dead_code)]
mod rax;
use rax::{Game, Renderer};
use rax::{KeyCode, MouseButton};

mod frame_counter;

mod player;
mod tile_map;

mod runplusplus;
use runplusplus::RunPlusPlus;

mod level_editor;
use level_editor::LevelEditor;

fn main() {
    println!("Hello, world!");

    let game = rax::GameBuilder::new()
        .with_title("R++")
        .with_size(1280, 720)
        .with_fullscreen(false)
        .with_vsync(true)
        .with_samples(8);

    game.run(MainGame::new());
}


/// Directs events to the correct mode
struct MainGame {
    current_mode: GameMode,

    running: bool,
    window_size: [u64; 2]
}


/// Possible game modes
enum GameMode {
    Game(Box<RunPlusPlus>),
    LevelEditor(Box<LevelEditor>)
}

impl MainGame {
    pub fn new() -> Self {
        MainGame {
            current_mode: GameMode::Game(Box::new(RunPlusPlus::new())),

            running: true,
            window_size: [0, 0],
        }
    }
}


impl Game for MainGame {
    fn update(&mut self, dt: f64) {
        self.current_mode.update(dt)
    }

    fn render(&mut self, renderer: &mut Renderer) {
        self.current_mode.render(renderer)
    }

    fn is_running(&self) -> bool {
        self.running && self.current_mode.is_running()
    }

    fn on_close(&mut self) {
        self.running = false;
        self.current_mode.on_close();
    }

    fn on_key_press(&mut self, key: KeyCode) {
        self.current_mode.on_key_press(key);

        if key == KeyCode::Escape {
            self.running = false;
        }
        if key == KeyCode::F1 {
            if !self.current_mode.is_game() {
                self.current_mode.on_close();
                self.current_mode = GameMode::Game(Box::new(RunPlusPlus::new()));
                self.current_mode.on_size_change(self.window_size[0], self.window_size[1]);
            }
        }
        if key == KeyCode::F2 {
            if !self.current_mode.is_level_editor() {
                self.current_mode.on_close();
                self.current_mode = GameMode::LevelEditor(Box::new(LevelEditor::new()));
                self.current_mode.on_size_change(self.window_size[0], self.window_size[1]);
            }
        }
    }

    fn on_key_release(&mut self, key: KeyCode) {
        self.current_mode.on_key_release(key)
    }

    fn on_mouse_press(&mut self, button: MouseButton, x: u64, y: u64) {
        self.current_mode.on_mouse_press(button, x, y)
    }

    fn on_mouse_release(&mut self, button: MouseButton, x: u64, y: u64) {
        self.current_mode.on_mouse_release(button, x, y)
    }

    fn on_mouse_move(&mut self, x: u64, y: u64) {
        self.current_mode.on_mouse_move(x, y)
    }

    fn on_size_change(&mut self, width: u64, height: u64) {
        self.window_size = [width, height];
        self.current_mode.on_size_change(width, height)
    }
}


impl GameMode {
    pub fn is_game(&self) -> bool {
        match *self {
            GameMode::Game(_) => true,
            _ => false
        }
    }

    pub fn is_level_editor(&self) -> bool {
        match *self {
            GameMode::LevelEditor(_) => true,
            _ => false
        }
    }
}


impl Game for GameMode {
    fn update(&mut self, dt: f64) {
        match *self {
            GameMode::Game(ref mut game) => game.update(dt),
            GameMode::LevelEditor(ref mut editor) => editor.update(dt),
        }
    }

    fn render(&mut self, renderer: &mut Renderer) {
        match *self {
            GameMode::Game(ref mut game) => game.render(renderer),
            GameMode::LevelEditor(ref mut editor) => editor.render(renderer),
        }
    }

    fn is_running(&self) -> bool {
        match *self {
            GameMode::Game(ref game) => game.is_running(),
            GameMode::LevelEditor(ref editor) => editor.is_running(),
        }
    }
    fn on_close(&mut self) {
        match *self {
            GameMode::Game(ref mut game) => game.on_close(),
            GameMode::LevelEditor(ref mut editor) => editor.on_close(),
        }
    }

    fn on_key_press(&mut self, key: KeyCode) {
        match *self {
            GameMode::Game(ref mut game) => game.on_key_press(key),
            GameMode::LevelEditor(ref mut editor) => editor.on_key_press(key),
        }
    }

    fn on_key_release(&mut self, key: KeyCode) {
        match *self {
            GameMode::Game(ref mut game) => game.on_key_release(key),
            GameMode::LevelEditor(ref mut editor) => editor.on_key_release(key),
        }
    }

    fn on_mouse_press(&mut self, button: MouseButton, x: u64, y: u64) {
        match *self {
            GameMode::Game(ref mut game) => game.on_mouse_press(button, x, y),
            GameMode::LevelEditor(ref mut editor) => editor.on_mouse_press(button, x, y),
        }
    }

    fn on_mouse_release(&mut self, button: MouseButton, x: u64, y: u64) {
        match *self {
            GameMode::Game(ref mut game) => game.on_mouse_release(button, x, y),
            GameMode::LevelEditor(ref mut editor) => editor.on_mouse_release(button, x, y),
        }
    }

    fn on_mouse_move(&mut self, x: u64, y: u64) {
        match *self {
            GameMode::Game(ref mut game) => game.on_mouse_move(x, y),
            GameMode::LevelEditor(ref mut editor) => editor.on_mouse_move(x, y),
        }
    }

    fn on_size_change(&mut self, width: u64, height: u64) {
        match *self {
            GameMode::Game(ref mut game) => game.on_size_change(width, height),
            GameMode::LevelEditor(ref mut editor) => editor.on_size_change(width, height),
        }
    }
}
