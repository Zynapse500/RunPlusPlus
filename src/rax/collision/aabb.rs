use trap::Vector2;


#[derive(Clone)]
pub struct AABB {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,

    // Determines if an edge can be collided with
    // [left, right, top, bottom]
    pub edges: [bool; 4],
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


    /// Returns true if two boxes intersect
    pub fn intersects(&self, other: &AABB) -> bool {
        self.left <= other.right && other.left <= self.right &&
            self.top <= other.bottom && other.top <= self.bottom
    }


    /// Returns true if this box contains a point
    pub fn contains(&self, point: Vector2) -> bool {
        self.left < point.x && point.x < self.right &&
            self.top < point.y && point.y < self.bottom
    }
}


impl Into<super::ConvexHull> for AABB {
    fn into(self) -> super::ConvexHull {
        super::ConvexHull::from_raw(
            vec![
                Vector2::new(self.left, self.top),
                Vector2::new(self.right, self.top),
                Vector2::new(self.right, self.bottom),
                Vector2::new(self.left, self.bottom)
            ],
            vec![
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 1.0),
            ],
            {
                let mut set = Vec::new();
                if !self.edges[0] { set.push(Vector2::new(1.0, 0.0)) }
                if !self.edges[1] { set.push(Vector2::new(-1.0, 0.0)) }
                if !self.edges[2] { set.push(Vector2::new(0.0, 1.0)) }
                if !self.edges[3] { set.push(Vector2::new(0.0, -1.0)) }

                Some(set)
            }
        )
    }
}


impl super::Bounded for AABB {
    fn bounding_box(&self) -> AABB {
        self.clone()
    }
}


impl super::Collide<AABB> for AABB {
    fn overlap(&self, other: &AABB) -> Option<(f64, Vector2)> {
        // Intersect on x
        if self.left < other.right && other.left < self.right &&
            self.top < other.bottom && other.top < self.bottom {
            let left = other.right - self.left;
            let right = self.right - other.left;

            let top = other.bottom - self.top;
            let bottom = self.bottom - other.top;

            let x = if left < right {
                if self.edges[0] && other.edges[1] { Some(left) } else { None }
            } else {
                if self.edges[1] && other.edges[0] { Some(right) } else { None }
            };

            let y = if top < bottom {
                if self.edges[2] && other.edges[3] { Some(top) } else { None }
            } else {
                if self.edges[3] && other.edges[2] { Some(bottom) } else { None }
            };

            if x.is_some() && y.is_some() {
                let x = x.unwrap();
                let y = y.unwrap();

                if x < y {
                    return Some((x, [if left < right { x } else { -x }, 0.0].into()));
                } else {
                    return Some((y, [0.0, if top < bottom { y } else { -y }].into()));
                }
            } else if x.is_some() && y.is_none() {
                let x = x.unwrap();
                return Some((x, [if left < right { x } else { -x }, 0.0].into()));
            } else if x.is_none() && y.is_some() {
                let y = y.unwrap();
                return Some((y, [0.0, if top < bottom { y } else { -y }].into()));
            }
        }

        return None;
    }
}


impl super::Collide<super::ConvexHull> for AABB {
    fn overlap(&self, other: &super::ConvexHull) -> Option<(f64, Vector2)> {
        let hull: super::ConvexHull = self.clone().into();

        hull.overlap(other)
    }
}

