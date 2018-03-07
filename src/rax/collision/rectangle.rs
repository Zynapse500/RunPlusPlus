use trap::Vector2i;

pub struct Rectangle {
    pub left: i64,
    pub right: i64,
    pub top: i64,
    pub bottom: i64,
}


impl Rectangle {
    pub fn new(left: i64, right: i64, top: i64, bottom: i64) -> Rectangle {
        Rectangle {
            left,
            right,
            top,
            bottom,
        }
    }


    pub fn contains(&self, point: Vector2i) -> bool {
        self.left <= point.x && point.x <= self.right &&
            self.top <= point.y && point.y <= self.bottom
    }
}