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
    accumulator: f64,

    running: bool,

    pressed_keys: HashSet<KeyCode>,
    window_size: Vector2i,

    player: ConvexHull,
    convex: ConvexHull,

    ground_normal: Option<Vector2>,
    wall_normal: Option<Vector2>,

    camera_center: Vector2,

    tilemap: TileMap,

    velocity: Vector2,
}


impl rax::Game for RunPlusPlus {
    fn new() -> Self {
        RunPlusPlus {
            frame_counter: FrameCounter::new(),

            time: 0.0,
            accumulator: 0.0,

            running: true,

            pressed_keys: HashSet::new(),
            window_size: Vector2i::new(1, 1),

            player: ConvexHull::from_points(&[
                Vector2::new(300.0, 100.0),
                Vector2::new(325.0, 100.0),
                Vector2::new(325.0, 150.0),
                Vector2::new(300.0, 150.0),
            ]),
            convex: ConvexHull::from_points(&[
                Vector2::new(300.0, 200.0 + 200.0),
                Vector2::new(500.0, 200.0 + 250.0),
                Vector2::new(400.0, 200.0 + 400.0),
                Vector2::new(300.0, 200.0 + 450.0),
                Vector2::new(250.0, 200.0 + 300.0),
            ]),

            ground_normal: None,
            wall_normal: None,

            camera_center: Vector2::new(0.0, 0.0),

            tilemap: TileMap::new(),

            velocity: [0.0; 2].into(),
        }
    }

    fn update(&mut self, dt: f64) {
        self.time += dt;
        self.accumulator += dt;

        let target_frame_time = 1.0 / 240.0;
        while self.accumulator > target_frame_time {
            let dt = target_frame_time;
            let mut delta = Vector2::new(0.0, 0.0);

            let plane = if let Some(normal) = self.ground_normal {
                Vector2::new(-normal.y, normal.x)
            } else {
                Vector2::new(1.0, 0.0)
            };

            if self.wall_normal.is_none() {
                if self.pressed_keys.contains(&KeyCode::A) { delta -= plane; }
                if self.pressed_keys.contains(&KeyCode::D) { delta += plane; }
            }

            if delta.len() != 0.0 { self.velocity += (delta.norm() * 300.0 * dt); }


            self.velocity.x -= self.velocity.x * dt * 1.0;
            self.velocity.y -= self.velocity.y * dt * 1.0;

            if let Some(normal) = self.wall_normal {
                let dot = normal.dot(&Vector2::new(-1.0, 0.0));
                if dot < -0.95 ||
                    dot > 0.95 {
                    println!("Wall!!");
                    if self.velocity.y > 0.0 {
                        self.velocity.y -= self.velocity.y * dt * 9.0;
                    }
                }
            }

            if self.velocity.y > 0.0 {
                self.velocity.y += 400.0 * dt;
            } else {
                self.velocity.y += 200.0 * dt;
            }

            self.player.translate(self.velocity * dt);

            self.ground_normal = None;
            let mut i = 0;
            loop {
                let first = {
                    // First, find all overlaps, then find the smallest overlap
                    let obstacles: &[&Collide<ConvexHull>] = &[&self.convex, &self.tilemap];
                    obstacles.into_iter().filter_map(|o| { o.overlap(&self.player) })
                        .min_by(|a, b| { a.0.partial_cmp(&b.0).unwrap() })
                };

                if let Some((overlap, resolve)) = first {
                    self.player.translate(-resolve);

                    let normal = -resolve.norm();

                    if normal.dot(&self.velocity) < 0.0 {
                        let plane = Vector2::new(normal.y, -normal.x);

                        let dot = plane.dot(&self.velocity);
                        self.velocity = dot * plane;
                    }

                    if normal.dot(&Vector2::new(0.0, -1.0)) > 0.5 {
                        self.ground_normal = Some(normal);
                    }

                    let dot = normal.dot(&Vector2::new(0.0, -1.0));
                    if dot < 0.05 && dot > -0.05 {
                        self.wall_normal = Some(normal);
                    }

                    i += 1;
                    if i > 10 {
                        break;
                    }
                } else {
                    break;
                }
            }

            if self.velocity.x != 0.0 {
                self.wall_normal = None;
            }

            self.camera_center += (self.player.average() - self.camera_center) * dt * 4.0;

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

        renderer.center = self.camera_center;

        renderer.clear(0.2, 0.2, 0.2);

        renderer.color = [0.0, 1.0, 0.0, 1.0];
        renderer.fill_convex(self.convex.get_points());



        renderer.color = [0.03, 0.03, 0.03, 1.0];
        self.tilemap.draw_shadows(renderer, self.player.average());

        self.tilemap.draw(renderer);

        renderer.color = [0.0, 0.0, 1.0, 1.0];
        renderer.fill_convex(self.player.get_points());
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_key_press(&mut self, key: KeyCode) {
        match key {
            KeyCode::Escape => self.running = false,

            KeyCode::R => {
                self.player = ConvexHull::from_points(&[
                    Vector2::new(300.0, 100.0),
                    Vector2::new(325.0, 100.0),
                    Vector2::new(325.0, 150.0),
                    Vector2::new(300.0, 150.0),
                ]);
                self.velocity = Vector2::new(0.0, 0.0);
            }

            KeyCode::Space => {
                if let Some(normal) = self.ground_normal {
                    self.velocity += normal * 250.0;
                } else if let Some(normal) = self.wall_normal {
                    self.velocity += (normal + Vector2::new(0.0, -1.5)).norm() * 250.0;
                }
                self.wall_normal = None;
            }

            KeyCode::S => {
                self.wall_normal = None;
            }

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

        let world_x = x as f64 + self.camera_center.x - self.window_size.x as f64 / 2.0;
        let world_y = y as f64 + self.camera_center.y - self.window_size.y as f64 / 2.0;

        let bx = (world_x / w).floor() as i64;
        let by = (world_y / h).floor() as i64;

        if button == MouseButton::Left {
            if self.pressed_keys.contains(&KeyCode::Key1) {
                self.tilemap.add_tile([bx, by].into(), Tile::WedgeUpLeft)
            } else if self.pressed_keys.contains(&KeyCode::Key2) {
                self.tilemap.add_tile([bx, by].into(), Tile::WedgeUpRight)
            } else if self.pressed_keys.contains(&KeyCode::Key3) {
                self.tilemap.add_tile([bx, by].into(), Tile::WedgeDownLeft)
            } else if self.pressed_keys.contains(&KeyCode::Key4) {
                self.tilemap.add_tile([bx, by].into(), Tile::WedgeDownRight)
            } else {
                self.tilemap.add_tile([bx, by].into(), Tile::Square)
            }
        } else if button == MouseButton::Right {
            self.tilemap.remove_tile([bx, by].into())
        }
    }


    fn on_size_change(&mut self, width: u64, height: u64) {
        self.window_size = Vector2i::new(width as i64, height as i64);
    }
}


struct TileMap {
    tiles: HashMap<Vector2i, (Tile, ConvexHull)>,
    player_start: Vector2i,
    tile_size: f64,
}

impl TileMap {
    pub fn new() -> TileMap {
        TileMap {
            tiles: HashMap::new(),
            player_start: Vector2i::new(2, 2),
            tile_size: 64.0,
        }
    }


    pub fn add_tile(&mut self, pos: Vector2i, tile: Tile) {
        let mut hull = tile.get_convex_hull(self.tile_size);
        hull.translate(Vector2::from(pos) * self.tile_size);

        self.tiles.insert(pos, (tile, hull));

        self.update_tile(pos);

        let deltas = &[
            Vector2i::new(1, 0),
            Vector2i::new(-1, 0),
            Vector2i::new(0, 1),
            Vector2i::new(0, -1),
        ];

        for delta in deltas.iter() {
            self.update_tile(pos + *delta);
        }
    }

    pub fn remove_tile(&mut self, pos: Vector2i) {
        self.tiles.remove(&pos);

        let deltas = &[
            Vector2i::new(1, 0),
            Vector2i::new(-1, 0),
            Vector2i::new(0, 1),
            Vector2i::new(0, -1),
        ];

        for delta in deltas.iter() {
            self.update_tile(pos + *delta);
        }
    }

    fn update_tile(&mut self, pos: Vector2i) {
        let directions = Direction::all();

        let mut neighbours = Vec::new();

        for direction in directions.iter() {
            if let Some(other) = self.tiles.get(&(direction.as_delta() + pos)) {
                if other.0.is_solid(*direction) {
                    neighbours.push((other.0, *direction));
                }
            }
        }

        if let Some(ref mut this) = self.tiles.get_mut(&pos) {
            this.1.clear_ignored_normals();
            for n in neighbours.into_iter() {
                if this.0.is_solid(n.1.opposite()) {
                    this.1.ignore_normal(n.1.as_delta().into());
                }
            }
        }
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        for (_, &(_, ref obstacle)) in self.tiles.iter() {
            renderer.color = [1.0, 0.0, 0.0, 0.2];
            renderer.fill_convex(obstacle.get_points());
        }

        for (_, &(_, ref obstacle)) in self.tiles.iter() {
            renderer.color = [0.0, 1.0, 1.0, 0.2];
            for line in obstacle.get_normals_as_lines(24.0) {
                renderer.draw_line(line.0, line.1);
            }
        }
    }


    pub fn draw_shadows(&self, renderer: &mut Renderer, center: Vector2) {
        for (_, &(_, ref obstacle)) in self.tiles.iter() {
            let points = obstacle.get_points();

            for i in 0..points.len() {
                let a = points[i];
                let b = points[(i + 1) % points.len()];

                let mid = (a + b) / 2.0;

                let a_far = a + (a - center).norm() * 3000.0;
                let b_far = b + (b - center).norm() * 3000.0;
                let mid_far = mid + (mid - center).norm() * 3000.0;

                renderer.fill_convex(&[a, a_far, mid_far, b_far, b]);
            }
        }
    }
}


impl Collide<ConvexHull> for TileMap {
    fn overlap(&self, other: &ConvexHull) -> Option<(f64, Vector2)> {
        let smallest = std::f64::INFINITY;
        let mut best = None;

        let bounding_box = other.bounding_box();

        for (_, &(_, ref obstacle)) in self.tiles.iter() {
// Broad phase
            if bounding_box.intersects(&obstacle.bounding_box()) {
// Narrow phase
                if let Some((overlap, resolve)) = obstacle.overlap(other) {
                    if overlap < smallest {
                        best = Some(resolve);
                    }
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


#[derive(Copy, Clone)]
enum Tile {
    Square,
    WedgeUpLeft,
    WedgeUpRight,
    WedgeDownLeft,
    WedgeDownRight,
}


#[derive(Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn all() -> [Direction; 4] {
        [Direction::Up, Direction::Down, Direction::Left, Direction::Right]
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn as_delta(&self) -> Vector2i {
        match *self {
            Direction::Up => Vector2i::new(0, -1),
            Direction::Down => Vector2i::new(0, 1),
            Direction::Left => Vector2i::new(-1, 0),
            Direction::Right => Vector2i::new(1, 0),
        }
    }
}


impl Tile {
    pub fn get_convex_hull(&self, size: f64) -> ConvexHull {
        match *self {
            Tile::Square => {
                ConvexHull::from_points(&[
                    Vector2::new(0.0, 0.0),
                    Vector2::new(size, 0.0),
                    Vector2::new(size, size),
                    Vector2::new(0.0, size),
                ])
            }

            Tile::WedgeUpLeft => {
                ConvexHull::from_points(&[
                    Vector2::new(size, 0.0),
                    Vector2::new(size, size),
                    Vector2::new(0.0, size),
                ])
            }
            Tile::WedgeUpRight => {
                ConvexHull::from_points(&[
                    Vector2::new(0.0, 0.0),
                    Vector2::new(size, size),
                    Vector2::new(0.0, size),
                ])
            }
            Tile::WedgeDownLeft => {
                ConvexHull::from_points(&[
                    Vector2::new(0.0, 0.0),
                    Vector2::new(size, 0.0),
                    Vector2::new(size, size),
                ])
            }
            Tile::WedgeDownRight => {
                ConvexHull::from_points(&[
                    Vector2::new(0.0, 0.0),
                    Vector2::new(size, 0.0),
                    Vector2::new(0.0, size),
                ])
            }

            _ => panic!("Cannot get convex hull!")
        }
    }

    pub fn is_solid(&self, incoming_direction: Direction) -> bool {
        match *self {
            Tile::Square => true,
            Tile::WedgeUpLeft => {
                match incoming_direction {
                    Direction::Down | Direction::Right => false,
                    _ => true
                }
            }
            Tile::WedgeUpRight => {
                match incoming_direction {
                    Direction::Down | Direction::Left => false,
                    _ => true
                }
            }
            Tile::WedgeDownLeft => {
                match incoming_direction {
                    Direction::Up | Direction::Right => false,
                    _ => true
                }
            }
            Tile::WedgeDownRight => {
                match incoming_direction {
                    Direction::Up | Direction::Left => false,
                    _ => true
                }
            }
        }
    }
}


impl Bounded for TileMap {
    fn bounding_box(&self) -> AABB {
        let mut left = std::i64::MAX;
        let mut right = std::i64::MIN;
        let mut top = std::i64::MAX;
        let mut bottom = std::i64::MIN;

        for (pos, _) in self.tiles.iter() {
            if pos.x < left { left = pos.x; }
            if pos.x > right { right = pos.x; }
            if pos.y < top { top = pos.y; }
            if pos.y > bottom { bottom = pos.y; }
        }

        AABB {
            left: left as f64 * self.tile_size,
            right: right as f64 * self.tile_size,
            top: top as f64 * self.tile_size,
            bottom: bottom as f64 * self.tile_size,
            edges: [true; 4],
        }
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

