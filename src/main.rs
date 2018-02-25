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
        .with_fullscreen(true)
        .with_vsync(false)
        .with_samples(8);

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
        if delta.len() != 0.0 { self.velocity += (delta.norm() * 2000.0 * dt); }

        self.velocity -= self.velocity * dt * 4.0;
        self.velocity.y += dt * 1600.0;
        self.player.translate(self.velocity * dt);

        let mut i = 0;
        loop {
            let first = {
                // First, find all overlaps, then find the smallest overlap
                let obstacles: &[&Collide<ConvexHull>] = &[&self.convex, &self.tilemap];
                obstacles.into_iter().filter_map(|o|{o.overlap(&self.player)})
                    .min_by(|a, b|{ a.0.partial_cmp(&b.0).unwrap() })
            };

            if let Some((overlap, resolve)) = first {
                self.player.translate(-resolve);

                let normal = -resolve.norm();

                if normal.dot(&self.velocity) < 0.0 {
                    let plane = Vector2::new(normal.y, -normal.x);

                    let dot = plane.dot(&self.velocity);
                    self.velocity = dot * plane;
                }

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
        } else if button == MouseButton::Right {
            self.tilemap.remove_tile([bx, by].into())
        } else {
            self.tilemap.add_slab([bx, by].into())
        }
    }
}


struct TileMap {
    tiles: HashMap<Vector2i, ConvexHull>
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

        let mut tile: ConvexHull = AABB {
            left: x,
            right: x + w,
            top: y,
            bottom: y + h,
            edges: [true; 4],
        }.into();

        if let Some(block) = self.tiles.get_mut(&[pos.x - 1, pos.y].into()) {
            tile.ignore_normal(Vector2::new(-1.0, 0.0));
            block.ignore_normal(Vector2::new(1.0, 0.0));
        }
        if let Some(block) = self.tiles.get_mut(&[pos.x + 1, pos.y].into()) {
            tile.ignore_normal(Vector2::new(1.0, 0.0));
            block.ignore_normal(Vector2::new(-1.0, 0.0));
        }
        if let Some(block) = self.tiles.get_mut(&[pos.x, pos.y - 1].into()) {
            tile.ignore_normal(Vector2::new(0.0, -1.0));
            block.ignore_normal(Vector2::new(0.0, 1.0));
        }
        if let Some(block) = self.tiles.get_mut(&[pos.x, pos.y + 1].into()) {
            tile.ignore_normal(Vector2::new(0.0, 1.0));
            block.ignore_normal(Vector2::new(0.0, -1.0));
        }

        self.tiles.insert(pos, tile);

        println!("tiles: {}", self.tiles.len());
    }


    pub fn add_slab(&mut self, pos: Vector2i) {
        let w = 64.0;
        let h = 64.0;

        let x = pos.x as f64 * w;
        let y = pos.y as f64 * h;

        let mut tile = ConvexHull::from_points(&[
            Vector2::new(x + w, y),
            Vector2::new(x + w, y + h),
            Vector2::new(x, y + h),
        ]);

        if let Some(block) = self.tiles.get_mut(&[pos.x + 1, pos.y].into()) {
            tile.ignore_normal(Vector2::new(1.0, 0.0));
            block.ignore_normal(Vector2::new(-1.0, 0.0));
        }
        if let Some(block) = self.tiles.get_mut(&[pos.x, pos.y + 1].into()) {
            tile.ignore_normal(Vector2::new(0.0, 1.0));
            block.ignore_normal(Vector2::new(0.0, -1.0));
        }

        self.tiles.insert(pos, tile);

        println!("tiles: {}", self.tiles.len());
    }


    pub fn remove_tile(&mut self, pos: Vector2i) {
        self.tiles.remove(&pos);
    }


    pub fn draw(&self, renderer: &mut Renderer) {
        for (_, obstacle) in self.tiles.iter() {
            renderer.color = [1.0, 0.0, 0.0, 1.0];
            renderer.fill_convex(obstacle.get_points());

            renderer.color = [0.0, 1.0, 1.0, 1.0];
            for line in obstacle.get_normals_as_lines(24.0) {
                renderer.draw_line(line.0, line.1);
            }
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

