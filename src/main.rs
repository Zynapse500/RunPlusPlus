#[macro_use]
extern crate glium;
extern crate trap;

use trap::{Vector2, Vector2i};

#[allow(dead_code)]
mod rax;

use rax::Renderer;
use rax::{KeyCode, MouseButton};

use rax::collision::*;

use std::collections::{HashSet};

mod player;
use player::{Player, PlayerCommand};

mod tile_map;
use tile_map::{TileMap, Tile};

fn main() {
    println!("Hello, world!");

    let game = rax::GameBuilder::new()
        .with_title("R++")
        .with_size(640, 360)
        .with_fullscreen(true)
        .with_vsync(false)
        .with_samples(8);

    game.run::<RunPlusPlus>();
}


struct RunPlusPlus {
    frame_counter: FrameCounter,

    time: f64,
    accumulator: f64,

    running: bool,

    pressed_keys: HashSet<KeyCode>,
    window_size: Vector2i,

    player: Player,
    convex: ConvexHull,

    camera_center: Vector2,

    tile_map: TileMap,
}


impl rax::Game for RunPlusPlus {
    fn new() -> Self {
        let tile_map = if let Some(tile_map) = TileMap::open("levels/tmp.lvl", 48.0) {
            println!("Loaded map!");
            tile_map
        } else {
            println!("Failed to load map!");
            TileMap::new(64.0)
        };

        RunPlusPlus {
            frame_counter: FrameCounter::new(),

            time: 0.0,
            accumulator: 0.0,

            running: true,

            pressed_keys: HashSet::new(),
            window_size: Vector2i::new(1, 1),

            player: tile_map.spawn_player(),
            convex: ConvexHull::from_points(&[
                Vector2::new(300.0 - 400.0, 200.0 + 200.0),
                Vector2::new(500.0 - 400.0, 200.0 + 250.0),
                Vector2::new(400.0 - 400.0, 200.0 + 400.0),
                Vector2::new(300.0 - 400.0, 200.0 + 450.0),
                Vector2::new(250.0 - 400.0, 200.0 + 300.0),
            ]),

            camera_center: Vector2::new(0.0, 0.0),

            tile_map,
        }
    }

    fn update(&mut self, dt: f64) {
        self.time += dt;
        self.accumulator += dt;

        let target_frame_time = 1.0 / 240.0;
        while self.accumulator > target_frame_time {
            let dt = target_frame_time;

            if self.pressed_keys.contains(&KeyCode::A) { self.player.submit_command(PlayerCommand::MoveLeft); }
            if self.pressed_keys.contains(&KeyCode::D) { self.player.submit_command(PlayerCommand::MoveRight); }

            if self.pressed_keys.contains(&KeyCode::S) {
                if self.pressed_keys.contains(&KeyCode::A) || self.pressed_keys.contains(&KeyCode::D) {
                    self.player.submit_command(PlayerCommand::Slide);
                }
            }

            self.player.update(dt, &[&self.tile_map, &self.convex]);

            self.camera_center += (self.player.get_center() - self.camera_center) * dt * 4.0;
            // self.camera_center = (self.player.get_center());

            self.accumulator -= target_frame_time;
            if self.accumulator > 1.0 {
                self.accumulator = 0.0;
            }
        }
    }

    fn render(&mut self, renderer: &mut Renderer) {
        if let Some(fps) = self.frame_counter.tick() {
            println!("FPS: {}", fps.round());
        }

        renderer.set_center(self.camera_center);

        renderer.clear(0.2, 0.2, 0.2);

        renderer.color = [0.0, 1.0, 0.0, 1.0];
        renderer.fill_convex(self.convex.get_points());


        renderer.color = [0.03, 0.03, 0.03, 1.0];
        self.tile_map.draw_shadows(renderer, self.player.get_center());

        self.tile_map.draw(renderer);

        self.player.draw(renderer);
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_key_press(&mut self, key: KeyCode) {
        match key {
            KeyCode::Escape => self.running = false,

            KeyCode::R => {
                self.player = Player::new(Vector2::new(0.0, 0.0));
            }

            KeyCode::Space => self.player.submit_command(PlayerCommand::Jump),

            KeyCode::S => {
                if !self.pressed_keys.contains(&KeyCode::A) && !self.pressed_keys.contains(&KeyCode::D) {
                    self.player.submit_command(PlayerCommand::Drop)
                }
            },

            KeyCode::F1 => self.tile_map.save("levels/tmp.lvl").unwrap_or_else(|e|{println!("{}", e)}),

            _ => ()
        }

        self.pressed_keys.insert(key);
    }

    fn on_key_release(&mut self, key: KeyCode) {
        self.pressed_keys.remove(&key);

        match key {
            KeyCode::Space => self.player.submit_command(PlayerCommand::StopJump),

            _ => ()
        }
    }

    fn on_mouse_press(&mut self, button: MouseButton, x: u64, y: u64) {
        let tile_size = self.tile_map.get_tile_size();

        let world_x = x as f64 + self.camera_center.x - self.window_size.x as f64 / 2.0;
        let world_y = y as f64 + self.camera_center.y - self.window_size.y as f64 / 2.0;

        let bx = (world_x / tile_size).floor() as i64;
        let by = (world_y / tile_size).floor() as i64;

        if button == MouseButton::Left {
            if self.pressed_keys.contains(&KeyCode::Key1) {
                self.tile_map.add_tile([bx, by].into(), Tile::WedgeUpLeft)
            } else if self.pressed_keys.contains(&KeyCode::Key2) {
                self.tile_map.add_tile([bx, by].into(), Tile::WedgeUpRight)
            } else if self.pressed_keys.contains(&KeyCode::Key3) {
                self.tile_map.add_tile([bx, by].into(), Tile::WedgeDownLeft)
            } else if self.pressed_keys.contains(&KeyCode::Key4) {
                self.tile_map.add_tile([bx, by].into(), Tile::WedgeDownRight)
            } else {
                self.tile_map.add_tile([bx, by].into(), Tile::Square)
            }
        } else if button == MouseButton::Right {
            self.tile_map.remove_tile([bx, by].into())
        }
    }


    fn on_size_change(&mut self, width: u64, height: u64) {
        self.window_size = Vector2i::new(width as i64, height as i64);
    }
}



use std::time::Instant;

struct FrameCounter {
    start: Instant,
    frames: u64,
}

impl FrameCounter {
    pub fn new() -> FrameCounter {
        FrameCounter {
            start: Instant::now(),
            frames: 0,
        }
    }


    pub fn tick(&mut self) -> Option<f64> {
        self.frames += 1;
        let now = Instant::now();
        let duration = now - self.start;

        let secs = duration.as_secs() as f64 + 1e-9 * duration.subsec_nanos() as f64;
        if secs > 0.5 {
            let fps = self.frames as f64 / secs;
            self.frames = 0;
            self.start = now;
            Some(fps)
        } else {
            None
        }
    }
}

