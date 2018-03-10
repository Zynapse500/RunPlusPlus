#[macro_use]
extern crate glium;
extern crate trap;

use trap::Vector2;

mod rax;

use rax::MouseButton;
use rax::collision::*;

fn main() {
    print!("Collision tests");

    let settings = rax::GameBuilder::new()
        .with_size(1280, 720)
        .with_fullscreen(false)
        .with_vsync(true);

    let game = settings.run(CollisionTests::new());
}

pub struct CollisionTests {
    rectangle: ConvexHull,
    obstacles: Vec<ConvexHull>,
}

impl CollisionTests {
    pub fn new() -> CollisionTests {
        CollisionTests {
            rectangle: ConvexHull::from_points(&[
                Vector2::new(200.0, 200.0),
                Vector2::new(300.0, 200.0),
                Vector2::new(300.0, 300.0),
                Vector2::new(200.0, 300.0),
            ]),
            obstacles: vec![
                ConvexHull::from_points(&[
                    Vector2::new(400.0, 300.0),
                    Vector2::new(600.0, 300.0),
                    Vector2::new(750.0, 500.0),
                    Vector2::new(400.0, 500.0),
                ]),
                ConvexHull::from_points(&[
                    Vector2::new(400.0, 300.0),
                    Vector2::new(400.0, 100.0),
                    Vector2::new(600.0, 100.0),
                    Vector2::new(600.0, 300.0),
                ]),
            ],
        }
    }
}

impl rax::Game for CollisionTests {
    fn update(&mut self, dt: f64) {}

    fn render(&mut self, renderer: &mut rax::Renderer) {
        renderer.clear(0.01, 0.01, 0.01);

        renderer.color = [0.0, 0.0, 1.0, 1.0];
        renderer.draw_convex(self.rectangle.get_points());

        renderer.color = [1.0, 0.0, 0.0, 1.0];
        for obstacle in self.obstacles.iter() {
            renderer.draw_convex(obstacle.get_points());
        }

        //for obstacle in self.obstacles.iter() {
        if let Some((_, resolve)) = self.obstacles.as_slice().overlap(&self.rectangle) {
            let mut copy = self.rectangle.clone();
            copy.translate(-resolve);

            renderer.color = [1.0, 1.0, 1.0, 1.0];
            renderer.draw_convex(copy.get_points());
        }
        //}
    }

    fn is_running(&self) -> bool {
        true
    }

    fn on_mouse_move(&mut self, x: u64, y: u64) {
        let center = self.rectangle.average();
        self.rectangle.translate(-center);
        self.rectangle.translate([x as f64, y as f64].into());
    }
}


impl<'a> Bounded for &'a [ConvexHull] {
    fn bounding_box(&self) -> AABB {
        use std::f64::INFINITY;

        let mut total = AABB::new(INFINITY, -INFINITY, INFINITY, -INFINITY);

        for convex in self.iter() {
            let rect = convex.bounding_box();
            if rect.left < total.left { total.left = rect.left };
            if rect.right > total.right { total.right = rect.right };
            if rect.top < total.top { total.top = rect.top };
            if rect.bottom > total.bottom { total.bottom = rect.bottom };
        }

        total
    }
}

/// Return the min and max projected values onto an axis
fn projected_range(points: &[Vector2], axis: Vector2) -> (f64, f64) {
    let mut min = None;
    let mut max = None;

    for point in points {
        let projection = axis.dot(*point);
        if min.is_none() {
            min = Some(projection);
        } else {
            if projection < min.unwrap() {
                min = Some(projection);
            }
        }

        if max.is_none() {
            max = Some(projection);
        } else {
            if projection > max.unwrap() {
                max = Some(projection);
            }
        }
    }

    (min.unwrap(), max.unwrap())
}


impl<'a> Collide<ConvexHull> for &'a [ConvexHull] {
    fn overlap(&self, other: &ConvexHull) -> Option<(f64, Vector2)> {
        let mut other = other.clone();

        let mut total_resolve = Vector2::new(0.0, 0.0);
        let mut remaining_iterations = 10;
        loop {
            let first = {
                let overlaps = self.iter().filter_map(|c| {
                    c.overlap(&other)
                });
                overlaps.min_by(|a, b| {
                    a.0.partial_cmp(&b.0).unwrap()
                })
            };

            if let Some((_, resolve)) = first {
                total_resolve += resolve;
                other.translate(-resolve);

                remaining_iterations -= 1;

                if remaining_iterations == 0 {
                    break;
                }
            } else {
                break;
            }
        }

        let overlap = total_resolve.len();
        if overlap != 0.0 {
            Some((overlap, total_resolve))
        } else {
            None
        }
    }
    /*fn overlap(&self, other: &ConvexHull) -> Option<(f64, Vector2)> {
            let bounding_box = other.bounding_box();
            let mut overlaps = Vec::new();

            'convex: for convex in self.iter() {

                let mut self_overlaps = Vec::new();

                let axes = convex.get_axes().iter().chain(other.get_axes().iter());

                for axis in axes {
                    let (self_min, self_max) = projected_range(convex.get_points(), *axis);
                    let (other_min, other_max) = projected_range(other.get_points(), *axis);

                    if self_min < other_max && other_min < self_max {
                        let left = self_max - other_min;
                        let right = other_max - self_min;

                        let (overlap, normal) = if left < right {(left, -*axis)} else {(right, *axis)};

                        self_overlaps.push((overlap, overlap * normal));
                    } else {
                        continue 'convex;
                    }
                }

                overlaps.extend_from_slice(self_overlaps.as_slice());
            }


            overlaps.sort_by(|a, b| { a.0.partial_cmp(&b.0).unwrap() });

            println!("Overlaps: {:?}", overlaps);

            for &(ref overlap, ref resolve) in overlaps.iter() {
                let mut clone = other.clone();

                clone.translate(-*resolve);

                let bounding_box = clone.bounding_box();
                if !self.iter().any(|c|{
                    if c.bounding_box().intersects(&bounding_box) {
                        if let Some((overlap, _)) = c.overlap(&clone) {
                            overlap > 1e-4
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }) {
                    return Some((*overlap, *resolve));
                }
            }

            None
        }*/
}

