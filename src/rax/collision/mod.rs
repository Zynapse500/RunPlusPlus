use trap::Vector2;

mod aabb;
pub use self::aabb::AABB;

mod convex_hull;
pub use self::convex_hull::ConvexHull;

mod circle;
pub use self::circle::Circle;


pub trait Collide<C>: Bounded {
    // Return the overlap depth and the minimal translation vector of self
    fn overlap(&self, other: &C) -> Option<(f64, Vector2)>;
}


pub trait Bounded {
    // Return the bounding box of this shape
    fn bounding_box(&self) -> AABB;
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
