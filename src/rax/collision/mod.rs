use trap::Vector2;

mod aabb;
pub use self::aabb::AABB;

mod convex_hull;
pub use self::convex_hull::ConvexHull;

pub trait Collide<C>: Bounded {
    // Return the overlap depth and the minimal translation vector of self
    fn overlap(&self, other: &C) -> Option<(f64, Vector2)>;
}


pub trait Bounded {
    // Return the bounding box of this shape
    fn bounding_box(&self) -> AABB;
}