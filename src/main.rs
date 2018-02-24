#[macro_use]
extern crate glium;
extern crate trap;

use trap::{Vector2, Vector2i};

mod rax;
use rax::Renderer;
use rax::{KeyCode, MouseButton};

mod collision;
use collision::*;

use std::collections::{HashSet, HashMap};

fn main() {
    println!("Hello, world!");

    let game = rax::GameBuilder::new()
        .with_title("R++")
        .with_size(1280, 720)
        .with_samples(8)
        .with_vsync(true);

    game.run::<RunPlusPlus>();
}


struct RunPlusPlus {
    frame_counter: FrameCounter,
    time: f64,

    running: bool,

    pressed_keys: HashSet<KeyCode>,

    player: ConvexHull,
    convex: ConvexHull,

    tilemap: TileMap,

    velocity: Vector2
}


impl rax::Game for RunPlusPlus {
    fn new() -> Self {
        RunPlusPlus {
            frame_counter: FrameCounter::new(),
            time: 0.0,

            running: true,

            pressed_keys: HashSet::new(),

            /*player: AABB {
                left: 150.0,
                right: 175.0,
                top: 150.0,
                bottom: 200.0,

                edges: [true; 4]
            },*/
            player: ConvexHull::from_points(&[
                Vector2::new(100.0, 100.0),
                Vector2::new(125.0, 100.0),
                Vector2::new(125.0, 150.0),
                Vector2::new(100.0, 150.0),
            ]),
            convex: ConvexHull::from_points(&[
                Vector2::new(300.0, 200.0),
                Vector2::new(500.0, 250.0),
                Vector2::new(400.0, 400.0),
                Vector2::new(300.0, 450.0),
                Vector2::new(250.0, 300.0),
            ]),

            tilemap: TileMap::new(),

            velocity: [0.0; 2].into(),
        }
    }

    fn update(&mut self, dt: f64) {
        self.time += dt * 2.0;

        let mut delta = Vector2::new(0.0, 0.0);
        if self.pressed_keys.contains(&KeyCode::A) { delta.x -= 1.0; }
        if self.pressed_keys.contains(&KeyCode::D) { delta.x += 1.0; }
        if self.pressed_keys.contains(&KeyCode::W) { delta.y -= 1.0; }
        if self.pressed_keys.contains(&KeyCode::S) { delta.y += 1.0; }
        if delta.len() != 0.0 { self.velocity += (delta.norm() * 600.0 * dt); }

        self.velocity -= self.velocity * dt * 4.0;
        self.velocity.y += dt * 1600.0;
        self.player.translate(self.velocity * dt);

        let mut i = 0;
        loop {
            if let Some((overlap, resolve)) = self.tilemap.overlap(&self.player) {
                self.player.translate(-resolve);

                let normal = -resolve.norm();
                let plane = Vector2::new(normal.y, -normal.x);

                let dot = plane.dot(&self.velocity);
                self.velocity = dot * plane;

                i += 1;
                if i > 100 {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn render(&mut self, renderer: &mut Renderer) {
        if let Some(fps) = self.frame_counter.tick() {
            println!("FPS: {}", fps.round());
        }

        renderer.clear(0.2, 0.2, 0.2);

        self.tilemap.draw(renderer);

        renderer.color = [0.0, 1.0, 0.0, 1.0];
        renderer.fill_convex(self.convex.get_points());

        renderer.color = [0.0, 0.0, 1.0, 1.0];
        renderer.fill_convex(self.player.get_points());
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_key_press(&mut self, key: KeyCode) {
        match key {
            KeyCode::Escape => self.running = false,

            KeyCode::Space => self.velocity.y -= 900.0,

            _ => ()
        }

        self.pressed_keys.insert(key);
    }

    fn on_key_release(&mut self, key: KeyCode) {
        self.pressed_keys.remove(&key);
    }

    fn on_mouse_press(&mut self, button: MouseButton, x: u64, y: u64) {
        let w = 64.0;
        let h = 64.0;

        let bx = (x as f64 / w).floor() as i64;
        let by = (y as f64 / h).floor() as i64;

        if button == MouseButton::Left {
            self.tilemap.add_tile([bx, by].into())
        } else {
            self.tilemap.remove_tile([bx, by].into())
        }
    }
}


struct TileMap {
    tiles: HashMap<Vector2i, AABB>
}

impl TileMap {
    pub fn new() -> TileMap {
        TileMap {
            tiles: HashMap::new(),
        }
    }


    pub fn add_tile(&mut self, pos: Vector2i) {
        let w = 64.0;
        let h = 64.0;

        let x = pos.x as f64 * w;
        let y = pos.y as f64 * h;

        let mut tile = AABB {
            left: x,
            right: x + w,
            top: y,
            bottom: y + h,
            edges: [true; 4],
        };

        if let Some(block) = self.tiles.get_mut(&[pos.x - 1, pos.y].into()) {
            tile.edges[0] = false;
            block.edges[1] = false;
        }
        if let Some(block) = self.tiles.get_mut(&[pos.x + 1, pos.y].into()) {
            tile.edges[1] = false;
            block.edges[0] = false;
        }
        if let Some(block) = self.tiles.get_mut(&[pos.x, pos.y - 1].into()) {
            tile.edges[2] = false;
            block.edges[3] = false;
        }
        if let Some(block) = self.tiles.get_mut(&[pos.x, pos.y + 1].into()) {
            tile.edges[3] = false;
            block.edges[2] = false;
        }

        self.tiles.insert(pos, tile);

        println!("tiles: {}", self.tiles.len());
    }


    pub fn remove_tile(&mut self, pos: Vector2i) {
        self.tiles.remove(&pos);

        if let Some(block) = self.tiles.get_mut(&[pos.x - 1, pos.y].into()) {
            block.edges[1] = true;
        }
        if let Some(block) = self.tiles.get_mut(&[pos.x + 1, pos.y].into()) {
            block.edges[0] = true;
        }
        if let Some(block) = self.tiles.get_mut(&[pos.x, pos.y - 1].into()) {
            block.edges[3] = true;
        }
        if let Some(block) = self.tiles.get_mut(&[pos.x, pos.y + 1].into()) {
            block.edges[2] = true;
        }
    }


    pub fn draw(&self, renderer: &mut Renderer) {
        for (_, obstacle) in self.tiles.iter() {
            renderer.color = [1.0, 0.0, 0.0, 1.0];
            renderer.fill_rectangle(obstacle.left, obstacle.right, obstacle.top, obstacle.bottom);

            renderer.color = [0.0, 0.0, 1.0, 1.0];
            if obstacle.edges[0] {
                renderer.fill_rectangle(obstacle.left, obstacle.left + 2.0, obstacle.top, obstacle.bottom);
            }
            if obstacle.edges[1] {
                renderer.fill_rectangle(obstacle.right - 2.0, obstacle.right, obstacle.top, obstacle.bottom);
            }
            if obstacle.edges[2] {
                renderer.fill_rectangle(obstacle.left, obstacle.right, obstacle.top, obstacle.top + 2.0);
            }
            if obstacle.edges[3] {
                renderer.fill_rectangle(obstacle.left, obstacle.right, obstacle.bottom - 2.0, obstacle.bottom);
            }
        }
    }
}

impl Collide<AABB> for TileMap {
    fn overlap(&self, other: &AABB) -> Option<(f64, Vector2)> {
        let smallest = std::f64::INFINITY;
        let mut best = None;

        for (_, obstacle) in self.tiles.iter() {
            if let Some((overlap, resolve)) = obstacle.overlap(other) {
                if overlap < smallest {
                    best = Some(resolve);
                }
            }
        }

        if let Some(resolve) = best {
            Some((smallest, resolve))
        } else {
            None
        }
    }
}


impl Collide<ConvexHull> for TileMap {
    fn overlap(&self, other: &ConvexHull) -> Option<(f64, Vector2)> {
        let smallest = std::f64::INFINITY;
        let mut best = None;

        for (_, obstacle) in self.tiles.iter() {
            if let Some((overlap, resolve)) = obstacle.overlap(other) {
                if overlap < smallest {
                    best = Some(resolve);
                }
            }
        }

        if let Some(resolve) = best {
            Some((smallest, resolve))
        } else {
            None
        }
    }
}



use std::time::Instant;
struct FrameCounter {
    start: Instant,
    frames: u64
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

