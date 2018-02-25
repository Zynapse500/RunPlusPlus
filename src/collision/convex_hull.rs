use trap::Vector2;

#[derive(Clone)]
pub struct ConvexHull {
    points: Vec<Vector2>,
    axes: Vec<Vector2>,

    ignored_normals: Vec<Vector2>
}

impl ConvexHull {
    /// Create a new convex hull from raw points and normals
    pub fn from_raw(points: Vec<Vector2>, normals: Vec<Vector2>, ignored_normals: Option<Vec<Vector2>>) -> ConvexHull {
        ConvexHull {
            points,
            axes: normals,

            ignored_normals: if let Some(normals) = ignored_normals {normals} else {Vec::new()}
        }
    }


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
            axes: normals,

            ignored_normals: Vec::new()
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

            if !self.ignored_normals.contains(&normal) {
                let middle = (start + end) * 0.5;
                lines.push((middle, middle + normal * length));
            }
        }

        lines
    }


    /// Add a normal to the ignored list
    pub fn ignore_normal(&mut self, normal: Vector2) {
        self.ignored_normals.push(normal);
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
        let axes = self.axes.iter().chain(other.axes.iter());

        let mut min_overlap = None;
        let mut min_axis = None;

        for axis in axes {
            let (self_min, self_max) = projected_range(self.points.as_slice(), *axis);
            let (other_min, other_max) = projected_range(other.points.as_slice(), *axis);

            if self_min < other_max && other_min < self_max {
                let left = self_max - other_min;
                let right = other_max - self_min;

                let (overlap, normal) = if left < right {(left, -*axis)} else {(right, *axis)};

                // Ignore any overlap generated by ignored normals
                if self.ignored_normals.contains(&(-normal)) ||
                    other.ignored_normals.contains(&normal) {
                    continue;
                }

                if min_overlap.is_none() {
                    min_overlap = Some(overlap);
                    min_axis = Some(normal);
                } else {
                    if overlap < min_overlap.unwrap() {
                        min_overlap = Some(overlap);
                        min_axis = Some(normal);
                    }
                }
            } else {
                return None;
            }
        }

        if min_overlap.is_some() && min_axis.is_some() {
            let overlap = min_overlap.unwrap();
            return Some((overlap, overlap * min_axis.unwrap()));
        } else {
            None
        }
    }
}


/// Return the min and max projected values onto an axis
fn projected_range(points: &[Vector2], axis: Vector2) -> (f64, f64) {
    let mut min = None;
    let mut max = None;

    for point in points {
        let projection = axis.dot(point);
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
