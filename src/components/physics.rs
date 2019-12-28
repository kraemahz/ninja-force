/// Accelerate in a direction and return new velocity in that direction.
pub fn accelerate1d(speed: f32, accel: f32, time_step: f32) -> f32 {
    speed + accel * time_step
}

/// Decelerate against the current velocity direction and return the new velocity.
pub fn decelerate1d(speed: f32, decel: f32, time_step: f32) -> f32 {
    let original_sign = speed.signum();
    original_sign * (speed.abs() - decel * time_step).max(0.0)
}

pub type Vector2 = [f32; 2];
pub type Corners = [Vector2; 2];

#[inline]
pub fn x(vec: Vector2) -> f32 {
    vec[0]
}

#[inline]
pub fn y(vec: Vector2) -> f32 {
    vec[1]
}

#[inline]
pub fn left(corners: Corners) -> f32 {
    x(corners[0])
}

#[inline]
pub fn bottom(corners: Corners) -> f32 {
    y(corners[0])
}

#[inline]
pub fn right(corners: Corners) -> f32 {
    x(corners[1])
}

#[inline]
pub fn top(corners: Corners) -> f32 {
    y(corners[1])
}

#[inline]
pub fn bottom_left(corners: Corners) -> Vector2 {
    [left(corners), bottom(corners)]
}

#[inline]
pub fn bottom_right(corners: Corners) -> Vector2 {
    [right(corners), bottom(corners)]
}

#[inline]
pub fn top_left(corners: Corners) -> Vector2 {
    [left(corners), top(corners)]
}

#[inline]
pub fn top_right(corners: Corners) -> Vector2 {
    [right(corners), top(corners)]
}

#[inline]
pub fn x_midpoint(corners: Corners) -> f32 {
    (right(corners) + left(corners)) / 2.0
}

#[inline]
pub fn y_midpoint(corners: Corners) -> f32 {
    (top(corners) + bottom(corners)) / 2.0
}

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox2D {
    pub corners: Corners,
}

#[inline]
fn shortest_from_corner(box_corners: Corners, corner: Vector2) -> Vector2 {
    let x_midpoint = x_midpoint(box_corners);
    let y_midpoint = y_midpoint(box_corners);

    let x_val = if corner[0] <= x_midpoint {
        x(box_corners[0]) - x(corner)
    } else {
        x(box_corners[1]) - x(corner)
    };

    let y_val = if corner[1] <= y_midpoint {
        y(box_corners[0]) - y(corner)
    } else {
        y(box_corners[1]) - y(corner)
    };
    if x_val.abs() < y_val.abs() {
        [x_val, 0.0]
    } else {
        [0.0, y_val]
    }
}

impl BoundingBox2D {
    pub(self) fn shortest_manhattan_one_side(&self, bbox: &BoundingBox2D) -> Option<Vector2> {
        if self.contains_strict(bottom_left(bbox.corners)) {
            Some(shortest_from_corner(
                self.corners,
                bottom_left(bbox.corners),
            ))
        } else if self.contains_strict(top_right(bbox.corners)) {
            Some(shortest_from_corner(self.corners, top_right(bbox.corners)))
        } else if self.contains_strict(top_left(bbox.corners)) {
            Some(shortest_from_corner(self.corners, top_left(bbox.corners)))
        } else if self.contains_strict(bottom_right(bbox.corners)) {
            Some(shortest_from_corner(
                self.corners,
                bottom_right(bbox.corners),
            ))
        } else {
            None
        }
    }

    pub(self) fn intersect_one_box(&self, bbox: &BoundingBox2D) -> bool {
        bbox.contains(bottom_left(self.corners))
            || bbox.contains(top_right(self.corners))
            || bbox.contains(top_left(self.corners))
            || bbox.contains(bottom_right(self.corners))
    }

    pub fn translate(&self, point: Vector2) -> BoundingBox2D {
        BoundingBox2D {
            corners: [
                [
                    x(self.corners[0]) + x(point),
                    y(self.corners[0]) + y(point),
                ],
                [
                    x(self.corners[1]) + x(point),
                    y(self.corners[1]) + y(point),
                ],
            ],
        }
    }

    pub fn shortest_manhattan_move(&self, bbox: &BoundingBox2D) -> Option<Vector2> {
        // Check from this side
        let first_result = self.shortest_manhattan_one_side(bbox);
        // Check from the other side
        let pre_second_result = bbox.shortest_manhattan_one_side(self);
        // Invert the direction of the move as from this side. It should move out the other way.
        let second_result = match pre_second_result {
            Some(result) => Some([-result[0], -result[1]]),
            None => None,
        };

        if first_result.is_none() {
            second_result
        } else if second_result.is_none() {
            first_result
        } else {
            let sum_first: f32 = first_result.unwrap().iter().sum();
            let sum_second: f32 = second_result.unwrap().iter().sum();
            if sum_first > sum_second {
                first_result
            } else {
                second_result
            }
        }
    }

    pub fn intersects(&self, bbox: &BoundingBox2D) -> bool {
        self.intersect_one_box(bbox) || bbox.intersect_one_box(self)
    }

    pub fn contains_strict(&self, point: Vector2) -> bool {
        x(point) > x(self.corners[0])
            && x(point) < x(self.corners[1])
            && y(point) > y(self.corners[0])
            && y(point) < y(self.corners[1])
    }

    pub fn contains(&self, point: Vector2) -> bool {
        x(point) >= x(self.corners[0])
            && x(point) <= x(self.corners[1])
            && y(point) >= y(self.corners[0])
            && y(point) <= y(self.corners[1])
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InverseBoundingBox2D {
    pub corners: Corners,
}

impl InverseBoundingBox2D {
    pub fn contains_strict(&self, point: Vector2) -> bool {
        x(point) < x(self.corners[0])
            || x(point) > x(self.corners[1])
            || y(point) < y(self.corners[0])
            || y(point) > y(self.corners[1])
    }

    pub fn contains(&self, point: Vector2) -> bool {
        x(point) <= x(self.corners[0])
            || x(point) >= x(self.corners[1])
            || y(point) <= y(self.corners[0])
            || y(point) >= y(self.corners[1])
    }

    pub fn shortest_manhattan_move(&self, bbox: &BoundingBox2D) -> Option<Vector2> {
        let point = if self.contains_strict(bottom_left(bbox.corners)) {
            bottom_left(bbox.corners)
        } else if self.contains_strict(top_right(bbox.corners)) {
            top_right(bbox.corners)
        } else if self.contains_strict(bottom_right(bbox.corners)) {
            bottom_right(bbox.corners)
        } else if self.contains_strict(top_left(bbox.corners)) {
            top_left(bbox.corners)
        } else {
            return None;
        };

        let x = if x(point) < x(bottom_left(self.corners)) {
            x(bottom_left(self.corners)) - x(point)
        } else if x(point) > x(top_right(self.corners)) {
            x(top_right(self.corners)) - x(point)
        } else {
            0.0
        };

        let y = if y(point) > y(top_right(self.corners)) {
            y(top_right(self.corners)) - y(point)
        } else if y(point) < y(bottom_left(self.corners)) {
            y(bottom_left(self.corners)) - y(point)
        } else {
            0.0
        };

        Some([x, y])
    }
}
