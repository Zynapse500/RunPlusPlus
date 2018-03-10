use trap::Vector2;

pub struct Circle {
    pub center: Vector2,
    pub radius: f64
}


impl Circle {
    pub fn new(center: Vector2, radius: f64) -> Circle {
        Circle {
            center,
            radius,
        }
    }


    pub fn translate(&mut self, amount: Vector2) {
        self.center += amount;
    }
}



impl super::Bounded for Circle {
    fn bounding_box(&self) -> super::AABB {
        super::AABB {
            left: self.center.x - self.radius,
            right: self.center.x + self.radius,
            top: self.center.y - self.radius,
            bottom: self.center.y + self.radius,
            edges: [true; 4],
        }
    }
}


impl super::Collide<super::ConvexHull> for Circle {
    fn overlap(&self, other: &super::ConvexHull) -> Option<(f64, Vector2)> {
        match other.overlap(self) {
            Some((overlap, resolve)) => Some((overlap, -resolve)),
            n => n,
        }
    }
}