use trap::{Vector2, Vector2i};

use rax::{Game, Renderer};
use rax::{MouseButton, KeyCode};

use rax::collision::*;

use tile_map::{TileMap, Tile};

use ::TILE_SIZE;

pub struct LevelEditor {
    tile_map: TileMap,

    camera_center: Vector2,
    window_size: Vector2i,

    current_tile: Vector2i,

    selection: Option<[Vector2i; 2]>,

    tile_panel: TilePanel,

    map_area: Rectangle,
    panel_area: Rectangle,
}

impl LevelEditor {
    pub fn new() -> Self {
        LevelEditor {
            tile_map: TileMap::new(TILE_SIZE),
            camera_center: Vector2::new(0.0, 0.0),

            window_size: Vector2i::new(0, 0),
            current_tile: Vector2i::new(0, 0),

            selection: None,

            tile_panel: TilePanel::new(),

            map_area: Rectangle::new(0, 0, 0, 0),
            panel_area: Rectangle::new(0, 0, 0, 0),
        }
    }


    fn screen_to_world(&self, screen: Vector2i) -> Vector2 {
        let width = self.map_area.right - self.map_area.left;
        let height = self.map_area.bottom - self.map_area.top;

        Vector2::new(
            screen.x as f64 + self.camera_center.x - width as f64 / 2.0,
            screen.y as f64 + self.camera_center.y - height as f64 / 2.0,
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
        renderer.set_viewport(
            self.map_area.left as u32,
            self.map_area.right as u32,
            self.map_area.top as u32,
            self.map_area.bottom as u32,
        );

        self.camera_center = renderer.get_center();

        renderer.clear(0.05, 0.05, 0.2);

        self.tile_map.draw(renderer);

        let (left, right, top, bottom) = self.tile_to_rect(self.current_tile);
        renderer.color = [0.0, 0.0, 0.0, 1.0];
        renderer.draw_rectangle(left, right, top, bottom);


        for tile in self.get_selected_tiles() {
            let (left, right, top, bottom) = self.tile_to_rect(tile);

            let mut convex = self.tile_panel.get_current_tile().get_convex_hull(TILE_SIZE);
            convex.translate([left, top].into());

            renderer.color = [0.0, 1.0, 0.0, 0.2];
            renderer.fill_convex(convex.get_points());
        }


        renderer.set_viewport(
            self.panel_area.left as u32,
            self.panel_area.right as u32,
            self.panel_area.top as u32,
            self.panel_area.bottom as u32,
        );
        self.tile_panel.render(renderer);
    }

    fn is_running(&self) -> bool {
        true
    }

    fn on_mouse_press(&mut self, button: MouseButton, x: u64, y: u64) {
        let screen = Vector2i::new(x as i64, y as i64);

        if self.map_area.contains(screen) {
            let tile = self.world_to_tile(self.screen_to_world(Vector2i::new(x as i64, y as i64)));

            self.selection = Some([tile, tile]);
        } else if self.panel_area.contains(screen) {
            self.tile_panel.on_mouse_press(button, x, y);
        }
    }

    fn on_mouse_release(&mut self, button: MouseButton, x: u64, y: u64) {
        let screen = Vector2i::new(x as i64, y as i64);

        if self.map_area.contains(screen) {
            if button == MouseButton::Left {
                for tile in self.get_selected_tiles() {
                    self.tile_map.add_tile(tile, self.tile_panel.get_current_tile())
                }
                self.selection = None;
            } else if button == MouseButton::Right {
                for tile in self.get_selected_tiles() {
                    self.tile_map.remove_tile(tile)
                }
                self.selection = None;
            }
        } else if self.panel_area.contains(screen) {
            self.tile_panel.on_mouse_release(button, x, y);
        }
    }

    fn on_mouse_move(&mut self, x: u64, y: u64) {
        let screen = Vector2i::new(x as i64, y as i64);

        if self.map_area.contains(screen) {
            self.current_tile = self.world_to_tile(self.screen_to_world([x as i64, y as i64].into()));

            if let Some(ref mut selection) = self.selection {
                selection[1] = self.current_tile;
            }
        } else if self.panel_area.contains(screen) {
            self.tile_panel.on_mouse_move(x - self.panel_area.left as u64, y - self.panel_area.top as u64);
        }
    }

    fn on_size_change(&mut self, width: u64, height: u64) {
        self.window_size.x = width as i64;
        self.window_size.y = height as i64;

        self.map_area.left = 0;
        self.map_area.right = width as i64;
        self.map_area.top = 0;
        self.map_area.bottom = height as i64 - (TILE_SIZE + 16.0) as i64;

        self.panel_area.left = 0;
        self.panel_area.right = width as i64;
        self.panel_area.top = self.map_area.bottom;
        self.panel_area.bottom = height as i64;

        self.tile_panel.on_size_change(
            (self.panel_area.right - self.panel_area.left) as u64,
            (self.panel_area.bottom - self.panel_area.top) as u64,
        );
    }
}


struct TilePanel {
    tiles: Vec<Tile>,
    bounding_boxes: Vec<(ConvexHull, Tile)>,

    render_size: Vector2i,
    mouse_position: Vector2i,

    current_tile: Tile,
}


impl TilePanel {
    pub fn new() -> TilePanel {
        let tiles = vec![
            Tile::Square,

            Tile::WedgeUpLeft,
            Tile::WedgeUpRight,
            Tile::WedgeDownLeft,
            Tile::WedgeDownRight,

            Tile::SlantedWedgeUpLeft,
            Tile::SlantUpLeft,

            Tile::SlantUpRight,
            Tile::SlantedWedgeUpRight,

            Tile::SlantedWedgeDownLeft,
            Tile::SlantDownLeft,

            Tile::SlantDownRight,
            Tile::SlantedWedgeDownRight,
        ];

        let bounding_boxes = tiles.iter().enumerate().map(|(i, tile)|{
            let x = 8.0 + i as f64 * 56.0;
            let y = 8.0;

            let mut convex = tile.get_convex_hull(TILE_SIZE);
            convex.translate([x, y].into());
            (convex, *tile)
        }).collect();


        TilePanel {
            tiles,

            bounding_boxes,

            render_size: Vector2i::new(0, 0),
            mouse_position: Vector2i::new(0, 0),

            current_tile: Tile::Square,
        }
    }


    pub fn get_current_tile(&self) -> Tile {
        self.current_tile
    }
}

impl Game for TilePanel {
    fn update(&mut self, dt: f64) {}

    fn render(&mut self, renderer: &mut Renderer) {
        renderer.set_center(Vector2::from(self.render_size) / 2.0);

        renderer.color = [0.05, 0.05, 0.05, 1.0];
        renderer.fill_rectangle(0.0, self.render_size.x as f64, 0.0, self.render_size.y as f64);

        for &(ref convex, ref tile) in self.bounding_boxes.iter() {
            if convex.bounding_box().contains(self.mouse_position.into()) {
                if *tile == self.current_tile {
                    renderer.color = [1.0, 0.5, 0.0, 1.0];
                } else {
                    renderer.color = [1.0, 0.0, 0.0, 1.0];
                }

            } else {
                if *tile == self.current_tile {
                    renderer.color = [0.4, 0.5, 0.0, 1.0];
                } else {
                    renderer.color = [0.4, 0.0, 0.0, 1.0];
                }
            }

            renderer.fill_convex(convex.get_points());
        }

        renderer.flush();
    }

    fn is_running(&self) -> bool {
        true
    }


    fn on_mouse_press(&mut self, button: MouseButton, x: u64, y: u64) {
        println!("Pressed!");

        if button == MouseButton::Left {
            for &(ref convex, ref tile) in self.bounding_boxes.iter() {
                if convex.bounding_box().contains(self.mouse_position.into()) {
                    self.current_tile = *tile;
                    return;
                }
            }
        }
    }

    fn on_mouse_move(&mut self, x: u64, y: u64) {
        self.mouse_position.x = x as i64;
        self.mouse_position.y = y as i64;
    }

    fn on_size_change(&mut self, width: u64, height: u64) {
        self.render_size.x = width as i64;
        self.render_size.y = height as i64;
    }
}
