use std;

use std::collections::HashMap;
use trap::{Vector2, Vector2i};

use rax::collision::*;
use rax::{Renderer};

pub struct TileMap {
    tiles: HashMap<Vector2i, (Tile, ConvexHull)>,
    tile_size: f64,
}


#[derive(Copy, Clone)]
pub enum Tile {
    Square,
    WedgeUpLeft,
    WedgeUpRight,
    WedgeDownLeft,
    WedgeDownRight,
}


#[derive(Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}


impl TileMap {
    pub fn new() -> TileMap {
        TileMap {
            tiles: HashMap::new(),
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
