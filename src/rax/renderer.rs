use glium;
use glium::{Display, Surface, Frame};
use trap::{Vector2};

pub struct Renderer {
    display: Display,
    program: glium::Program,

    vertices: Vec<Vertex>,
    indices: Vec<u32>,

    pub center: Vector2,
    view: (f32, f32, f32, f32),

    pub color: [f64; 4],
    pub line_width: f64,

    frame: Option<Frame>,
}


impl Renderer {
    pub fn new(display: Display) -> Renderer {
        use std::str::from_utf8;
        let program = glium::Program::from_source(
            &display,
            from_utf8(include_bytes!("shader/renderer.vert")).unwrap(),
            from_utf8(include_bytes!("shader/renderer.frag")).unwrap(),
            None,
        ).unwrap();

        Renderer {
            display,
            program,

            vertices: Vec::new(),
            indices: Vec::new(),

            center: Vector2::new(0.0, 0.0),
            view: (-1.0, 1.0, 1.0, -1.0),

            color: [1.0, 0.0, 0.0, 1.0],
            line_width: 1.0,

            frame: None,
        }
    }


    /// Begin a new rendering procedure
    pub fn begin(&mut self) {
        self.frame = Some(self.display.draw());

        if let Some((w, h)) = self.display.gl_window().window().get_inner_size() {
            let w = w as f32;
            let h = h as f32;
            self.view = (-w / 2.0, w / 2.0, -h / 2.0, h / 2.0);
            self.center = Vector2::new(w as f64 / 2.0, h as f64 / 2.0);
        }
    }


    /// Finalize all rendering
    pub fn end(&mut self) {
        self.flush();

        if let Some(frame) = self.frame.take() {
            frame.finish().unwrap();
        }
    }


    /// Submit all commands so far
    pub fn flush(&mut self) {
        if let Some(ref mut frame) = self.frame {
            let vertex_buffer = glium::VertexBuffer::new(
                &self.display,
                self.vertices.as_slice(),
            ).unwrap();

            let index_buffer = glium::IndexBuffer::new(
                &self.display,
                glium::index::PrimitiveType::TrianglesList,
                self.indices.as_slice(),
            ).unwrap();

            let (left, right, top, bottom) = self.view;

            let uniforms = uniform!(
                left: left,
                right: right,
                top: top,
                bottom: bottom,

                translation: [self.center.x as f32, self.center.y as f32]
            );

            frame.draw(&vertex_buffer, &index_buffer, &self.program, &uniforms, &Default::default()).unwrap();

            self.vertices.clear();
            self.indices.clear();
        }
    }


    /// Clear the screen with a color
    pub fn clear(&mut self, r: f64, g: f64, b: f64) {
        if let Some(ref mut frame) = self.frame {
            frame.clear_color(r as f32, g as f32, b as f32, 1.0);
        }
    }


    /// Render a filled rectangle
    pub fn fill_rectangle(&mut self, left: f64, right: f64, top: f64, bottom: f64) {
        let i = self.vertices.len() as u32;

        self.vertices.push(Vertex::pc([left, top], self.color));
        self.vertices.push(Vertex::pc([right, top], self.color));
        self.vertices.push(Vertex::pc([right, bottom], self.color));
        self.vertices.push(Vertex::pc([left, bottom], self.color));

        self.indices.push(i + 0);
        self.indices.push(i + 1);
        self.indices.push(i + 2);
        self.indices.push(i + 2);
        self.indices.push(i + 3);
        self.indices.push(i + 0);
    }


    /// Render the outline of a rectangle
    pub fn draw_rectangle(&mut self, left: f64, right: f64, top: f64, bottom: f64) {
        let width = self.line_width;

        // Top
        self.fill_rectangle(left, right, top, top + width);

        // Right
        self.fill_rectangle(right - width, right, top, bottom);

        // Bottom
        self.fill_rectangle(left, right, bottom - width, bottom);

        // Left
        self.fill_rectangle(left, left + width, top, bottom);
    }


    /// Render a convex polygon
    pub fn fill_convex(&mut self, points: &[Vector2]) {
        let start_index = self.vertices.len() as u32;

        for (index, point) in points.iter().enumerate() {
            self.vertices.push(Vertex::pc((*point).into(), self.color));

            if index < points.len() - 2 {
                self.indices.push(start_index);
                self.indices.push(start_index + index as u32 + 1);
                self.indices.push(start_index + index as u32 + 2);
            }
        }
    }


    /// Render a line from point a to b
    pub fn draw_line(&mut self, a: Vector2, b: Vector2) {
        let start_index = self.vertices.len() as u32;

        let delta = b - a;
        let dir = delta.norm();
        let radius = self.line_width / 2.0;
        let perp = Vector2::new(dir.y, -dir.x) * radius;

        let a_up = a + perp;
        let a_down = a - perp;
        let b_up = b + perp;
        let b_down = b - perp;

        self.vertices.push(Vertex::pc(a_up.into(), self.color));
        self.vertices.push(Vertex::pc(b_up.into(), self.color));
        self.vertices.push(Vertex::pc(b_down.into(), self.color));
        self.vertices.push(Vertex::pc(a_down.into(), self.color));

        self.indices.push(start_index + 0);
        self.indices.push(start_index + 1);
        self.indices.push(start_index + 2);
        self.indices.push(start_index + 2);
        self.indices.push(start_index + 3);
        self.indices.push(start_index + 0);
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4]
}

impl Vertex {
    /// Create a new vector from raw scalars
    pub fn new(x: f64, y: f64, r: f64, g: f64, b: f64, a: f64) -> Vertex {
        Vertex {
            position: [x as f32, y as f32],
            color: [r as f32, g as f32, b as f32, a as f32]
        }
    }

    /// Create a new Vertex with a position
    pub fn p(x: f64, y: f64) -> Vertex {
        Vertex {
            position: [x as f32, y as f32],
            color: [1.0, 0.0, 0.0, 1.0]
        }
    }

    /// Create a new Vertex with a position and color
    pub fn pc(position: [f64; 2], color: [f64; 4]) -> Vertex {
        Vertex {
            position: [position[0] as f32, position[1] as f32],
            color: [color[0] as f32, color[1] as f32, color[2] as f32, color[3] as f32]
        }
    }
}


implement_vertex!(Vertex, position, color);
