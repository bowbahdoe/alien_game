/// Rectangle for checking collisions
pub trait CollisionRect {
    fn top_left_x(&self) -> f32;
    fn top_left_y(&self) -> f32;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
}

pub fn are_colliding<Rect1: CollisionRect, Rect2: CollisionRect>(
    rect1: &Rect1,
    rect2: &Rect2,
) -> bool {
    // https://developer.mozilla.org/en-US/docs/Games/Techniques/2D_collision_detection
    rect1.top_left_x() < rect2.top_left_x() + rect2.width()
        && rect1.top_left_x() + rect1.width() > rect2.top_left_x()
        && rect1.top_left_y() < rect2.top_left_y() + rect2.height()
        && rect1.top_left_y() + rect1.height() > rect2.top_left_y()
}
