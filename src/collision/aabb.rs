use trap::Vector2;

pub struct AABB {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,

    // Determines if an edge can be collided with
    pub edges: [bool; 4]
}


impl AABB {
    pub fn new(left: f64, right: f64, top: f64, bottom: f64) -> AABB {
        AABB {
            left,
            right,
            top,
            bottom,
            edges: [true; 4],
        }
    }

    pub fn translate(&mut self, amount: Vector2) {
        self.left += amount.x;
        self.right += amount.x;
        self.top += amount.y;
        self.bottom += amount.y;
    }
}


impl super::Collide<AABB> for AABB {
    fn overlap(&self, other: &AABB) -> Option<(f64, Vector2)> {
        // Intersect on x
        if self.left < other.right && other.left < self.right &&
            self.top < other.bottom && other.top < self.bottom {
            let left = self.right - other.left;
            let right = other.right - self.left;

            let top = self.bottom - other.top;
            let bottom = other.bottom - self.top;

            let x = if left < right { left } else { right };
            let y = if top < bottom { top } else { bottom };

            if x < y {
                Some((x, (if left < right {[-x, 0.0]} else {[x, 0.0]}).into()))
            } else {
                Some((x, (if top < bottom {[0.0, -y]} else {[0.0, y]}).into()))
            }
        } else {
            None
        }
    }
}
