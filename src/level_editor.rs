use trap::{Vector2, Vector2i};

use rax::{Game, Renderer};
use rax::{MouseButton, KeyCode};

use tile_map::{TileMap, Tile};

use ::TILE_SIZE;

pub struct LevelEditor {
    tile_map: TileMap,

    camera_center: Vector2,
    window_size: Vector2i,

    current_tile: Vector2i,

    selection: Option<[Vector2i; 2]>,
}

impl LevelEditor {
    pub fn new() -> Self {
        LevelEditor {
            tile_map: TileMap::new(TILE_SIZE),
            camera_center: Vector2::new(0.0, 0.0),

            window_size: Vector2i::new(0, 0),
            current_tile: Vector2i::new(0, 0),

            selection: None,
        }
    }


    fn screen_to_world(&self, screen: Vector2i) -> Vector2 {
        Vector2::new(
            screen.x as f64 + self.camera_center.x - self.window_size.x as f64 / 2.0,
            screen.y as f64 + self.camera_center.y - self.window_size.y as f64 / 2.0,
        )
    }

    fn world_to_tile(&self, world: Vector2) -> Vector2i {
        let tile_size = self.tile_map.get_tile_size();

        Vector2i {
            x: (world.x / tile_size).floor() as i64,
            y: (world.y / tile_size).floor() as i64,
        }
    }


    fn get_selected_tiles(&self) -> Vec<Vector2i> {
        let mut tiles = Vec::new();

        if let Some(selection) = self.selection {
            let start = selection[0];
            let end = selection[1];
            let (left, right) = if start.x < end.x { (start.x, end.x) } else { (end.x, start.x) };
            let (top, bottom) = if start.y < end.y { (start.y, end.y) } else { (end.y, start.y) };

            for x in left..right + 1 {
                for y in top..bottom + 1 {
                    tiles.push([x, y].into());
                }
            }
        }

        tiles
    }


    fn tile_to_rect(&self, tile: Vector2i) -> (f64, f64, f64, f64) {
        let left = tile.x as f64 * TILE_SIZE;
        let right = tile.x as f64 * TILE_SIZE + TILE_SIZE;
        let top = tile.y as f64 * TILE_SIZE;
        let bottom = tile.y as f64 * TILE_SIZE + TILE_SIZE;

        (left, right, top, bottom)
    }
}

impl Game for LevelEditor {
    fn update(&mut self, _dt: f64) {}

    fn render(&mut self, renderer: &mut Renderer) {
        self.camera_center = renderer.get_center();

        renderer.clear(0.05, 0.05, 0.2);
        self.tile_map.draw(renderer);

        renderer.color = [0.0, 0.0, 0.0, 1.0];
        let (left, right, top, bottom) = self.tile_to_rect(self.current_tile);
        renderer.draw_rectangle(left, right, top, bottom);


        for tile in self.get_selected_tiles() {
            renderer.color = [0.0, 1.0, 0.0, 0.2];

            let (left, right, top, bottom) = self.tile_to_rect(tile);
            renderer.fill_rectangle(left, right, top, bottom);
        }

    }

    fn is_running(&self) -> bool {
        true
    }

    fn on_mouse_press(&mut self, button: MouseButton, x: u64, y: u64) {
        println!("Camera: {:?}", self.camera_center);

        let tile = self.world_to_tile(self.screen_to_world(Vector2i::new(x as i64, y as i64)));

        self.selection = Some([tile, tile]);
    }

    fn on_mouse_release(&mut self, button: MouseButton, x: u64, y: u64) {
        if button == MouseButton::Left {
            for tile in self.get_selected_tiles() {
                self.tile_map.add_tile(tile, Tile::Square)
            }
            self.selection = None;
        } else if button == MouseButton::Right {
            for tile in self.get_selected_tiles() {
                self.tile_map.remove_tile(tile)
            }
            self.selection = None;
        }
    }

    fn on_mouse_move(&mut self, x: u64, y: u64) {
        self.current_tile = self.world_to_tile(self.screen_to_world([x as i64, y as i64].into()));

        if let Some(ref mut selection) = self.selection {
            selection[1] = self.current_tile;
        }
    }

    fn on_size_change(&mut self, width: u64, height: u64) {
        self.window_size.x = width as i64;
        self.window_size.y = height as i64;
    }
}
