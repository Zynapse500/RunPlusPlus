use std;

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use trap::{Vector2, Vector2i};

use rax::collision::*;
use rax::Renderer;

use player::Player;

pub struct TileMap {
    tiles: HashMap<Vector2i, (Tile, ConvexHull)>,
    tile_size: f64,

    player_start: Vector2i,
}


#[derive(Copy, Clone, Hash, Eq, PartialEq)]
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
    /// Create a new tile map
    pub fn new(tile_size: f64) -> TileMap {
        TileMap {
            tiles: HashMap::new(),
            tile_size,

            player_start: Vector2i::new(0, 0),
        }
    }


    /// Open a tile map from disk
    pub fn open<P: AsRef<Path>>(path: P, tile_size: f64) -> Option<TileMap> {
        if let Ok(mut file) = File::open(path) {
            use std::io::Read;

            let mut string = String::new();
            if file.read_to_string(&mut string).is_err() {
                return None;
            }

            return TileMap::from_str(&string, tile_size);
        } else {
            None
        }
    }


    /// Save a tile map to disk
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        match std::fs::OpenOptions::new().create(true).write(true).open(path) {
            Ok(mut file) => {
                use std::io::Write;
                let mut text = format!("PLAYER_START {} {}", self.player_start.x, self.player_start.y);

                let mut tiles: HashMap<Tile, Vec<Vector2i>> = HashMap::new();
                for (position, tile) in self.tiles.iter() {
                    if let Some(ref mut positions) = tiles.get_mut(&tile.0) {
                        positions.push(*position);
                        continue;
                    }

                    tiles.insert(tile.0, vec![*position]);
                }

                for (tile, positions) in tiles {
                    text += "\n";
                    if positions.len() > 0 {
                        text.push_str(&format!("TILE \"{}\"", tile));
                        for position in positions {
                            text.push_str(&format!(" {}:{}", position.x, position.y));
                        }
                    }
                }

                if let Err(e) = file.write(text.as_bytes()) { return Err(e); }

                Ok(())
            }
            Err(e) => Err(e)
        }
    }


    /// Parses a string describing a tile map
    fn from_str(text: &str, tile_size: f64) -> Option<TileMap> {
        let mut tile_map = TileMap {
            tiles: HashMap::new(),
            tile_size,
            player_start: Vector2i::new(0, 0),
        };

        let lines = text.lines().map(|l| { l.split_whitespace() });

        for mut line in lines {
            if let Some(word) = line.next() {
                match word {
                    // Sets the start location of the player
                    "PLAYER_START" => {
                        if let Some(x) = line.next() {
                            if let Some(y) = line.next() {
                                tile_map.player_start.x = x.parse().unwrap();
                                tile_map.player_start.y = y.parse().unwrap();
                                continue;
                            }
                        }

                        println!("PLAYER_START: invalid number of arguments!");
                        return None;
                    }


                    // Adds new tiles to the map
                    "TILE" => {
                        if let Some(id) = line.next() {
                            let tile = Tile::from(id.trim_matches('\"'));
                            while let Some(coordinate) = line.next() {
                                let mut numbers = coordinate.split(':');

                                if let Some(x) = numbers.next() {
                                    if let Some(y) = numbers.next() {
                                        tile_map.add_tile(Vector2i::new(x.parse().unwrap(), y.parse().unwrap()), tile);
                                        continue;
                                    }
                                }

                                println!("Invalid coordinate format: '{}'", coordinate);
                                return None;
                            }
                        }
                    }

                    word => {
                        println!("Failed to load tile map: invalid command '{}'", word);
                        return None;
                    }
                }
            }
        }

        Some(tile_map)
    }


    /// Returns the size, in pixels, of a single tile
    pub fn get_tile_size(&self) -> f64 {
        self.tile_size
    }


    /// Returns a new player located in this map
    pub fn spawn_player(&self) -> Player {
        Player::new(self.tile_size * Vector2::from(self.player_start) + Vector2::new(self.tile_size / 2.0, self.tile_size / 2.0))
    }


    /// Adds a tile to the map
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


    /// Removes a tile from the map
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


    /// Updates a singular tile
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


    /// Renders the entire map
    pub fn draw(&self, renderer: &mut Renderer) {
        // Tiles
        for (_, &(_, ref obstacle)) in self.tiles.iter() {
            renderer.color = [1.0, 0.0, 0.0, 0.2];
            renderer.fill_convex(obstacle.get_points());
        }

        // Normals
        /*for (_, &(_, ref obstacle)) in self.tiles.iter() {
            renderer.color = [0.0, 1.0, 1.0, 0.2];
            for line in obstacle.get_normals_as_lines(24.0) {
                renderer.draw_line(line.0, line.1);
            }
        }*/
    }


    /// Renders shadows casted from a singular point
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

impl<C> Collide<C> for TileMap
    where C: Collide<ConvexHull>
{
    fn overlap(&self, other: &C) -> Option<(f64, Vector2)> {
        let smallest = std::f64::INFINITY;
        let mut best = None;

        let bounding_box = other.bounding_box();

        for (_, &(_, ref obstacle)) in self.tiles.iter() {
            // Broad phase
            if bounding_box.intersects(&obstacle.bounding_box()) {
                // Narrow phase
                if let Some((overlap, resolve)) = other.overlap(obstacle) {
                    if overlap < smallest {
                        best = Some(-resolve);
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

impl<'a> From<&'a str> for Tile {
    fn from(id: &'a str) -> Self {
        match id as &str {
            "Square" => Tile::Square,
            "WedgeUpLeft" => Tile::WedgeUpLeft,
            "WedgeUpRight" => Tile::WedgeUpRight,
            "WedgeDownLeft" => Tile::WedgeDownLeft,
            "WedgeDownRight" => Tile::WedgeDownRight,

            id => panic!("Did not recognize '{}' as a tile name", id)
        }
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match *self {
            Tile::Square => "Square",
            Tile::WedgeUpLeft => "WedgeUpLeft",
            Tile::WedgeUpRight => "WedgeUpRight",
            Tile::WedgeDownLeft => "WedgeDownLeft",
            Tile::WedgeDownRight => "WedgeDownRight",
        })
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
