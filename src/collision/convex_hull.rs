use trap::Vector2;

#[derive(Clone)]
pub struct ConvexHull {
    points: Vec<Vector2>,
    normals: Vec<Vector2>
}

impl ConvexHull {
    /// Create a new convex hull from points in a clockwise order
    pub fn from_points(points: &[Vector2]) -> ConvexHull {
        let points: Vec<Vector2> = points.iter().map(|p|{p.clone()}).collect();
        let mut normals = Vec::new();

        for i in 0..points.len() {
            let start: Vector2 = points[i];
            let end: Vector2 = points[(i + 1) % points.len()];

            let direction = (end - start).norm();
            let normal = Vector2::from([direction.y, -direction.x]);

            let mut duplicate = false;
            for n in normals.iter() {
                if normal == *n || -normal == *n {
                    duplicate = true;
                }
            }

            if !duplicate {
                normals.push(normal);
            }
        }

        ConvexHull {
            points,
            normals
        }
    }


    /// Return the points of this hull
    pub fn get_points<'a>(&'a self) -> &'a [Vector2] {
        self.points.as_slice()
    }

    /// Return the normals of this hull as lines
    pub fn get_normals_as_lines(&self, length: f64) -> Vec<(Vector2, Vector2)> {
        let mut lines = Vec::new();

        for i in 0..self.points.len() {
            let start: Vector2 = self.points[i];
            let end: Vector2 = self.points[(i + 1) % self.points.len()];

            let direction = (end - start).norm();
            let normal = Vector2::from([direction.y, -direction.x]);

            let middle = (start + end) * 0.5;
            lines.push((middle, middle + normal * length));
        }

        lines
    }


    /// Translate the convex hull
    pub fn translate(&mut self, amount: Vector2) {
        for point in self.points.iter_mut() {
            *point += amount;
        }
    }
}


impl super::Collide<ConvexHull> for ConvexHull {
    fn overlap(&self, other: &ConvexHull) -> Option<(f64, Vector2)> {
        let mut normals = self.normals.clone();
        normals.extend_from_slice(other.normals.as_slice());

        let mut min_overlap = None;
        let mut axis = None;

        for normal in normals {
            let mut self_min = None;
            let mut self_max = None;

            for point in self.points.iter() {
                let projection = normal.dot(point);
                if self_min.is_none() {
                    self_min = Some(projection);
                } else {
                    if projection < self_min.unwrap() {
                        self_min = Some(projection);
                    }
                }

                if self_max.is_none() {
                    self_max = Some(projection);
                } else {
                    if projection > self_max.unwrap() {
                        self_max = Some(projection);
                    }
                }
            }

            let mut other_min = None;
            let mut other_max = None;

            for point in other.points.iter() {
                let projection = normal.dot(point);
                if other_min.is_none() {
                    other_min = Some(projection);
                } else {
                    if projection < other_min.unwrap() {
                        other_min = Some(projection);
                    }
                }

                if other_max.is_none() {
                    other_max = Some(projection);
                } else {
                    if projection > other_max.unwrap() {
                        other_max = Some(projection);
                    }
                }
            }


            // Check for overlap
            let self_min = self_min.unwrap();
            let self_max = self_max.unwrap();
            let other_min = other_min.unwrap();
            let other_max = other_max.unwrap();

            if self_min < other_max && other_min < self_max {
                let left = self_max - other_min;
                let right = other_max - self_min;

                let overlap = if left < right {left} else {right};
                if min_overlap.is_none() {
                    min_overlap = Some(overlap);
                    axis = Some(if left < right {-normal} else {normal});
                } else {
                    if overlap < min_overlap.unwrap() {
                        min_overlap = Some(overlap);
                        axis = Some(if left < right {-normal} else {normal});
                    }
                }
            } else {
                return None;
            }
        }

        if min_overlap.is_some() && axis.is_some() {
            let overlap = min_overlap.unwrap();
            return Some((overlap, overlap * axis.unwrap()));
        } else {
            None
        }
    }
}

