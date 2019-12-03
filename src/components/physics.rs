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

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox2D {
    pub corners: [Vector2; 2],
}

impl BoundingBox2D {
    pub fn translate(&self, point: Vector2) -> BoundingBox2D {
        BoundingBox2D {
            corners: [
                [self.corners[0][0] + point[0], self.corners[0][1] + point[1]],
                [self.corners[1][0] + point[0], self.corners[1][1] + point[1]],
            ],
        }
    }

    pub fn shortest_from_corner(&self, corner: Vector2) -> Vector2 {
        let x_midpoint = (self.corners[1][0] + self.corners[0][0]) / 2.0;
        let y_midpoint = (self.corners[1][1] + self.corners[0][1]) / 2.0;

        let x_val = if corner[0] <= x_midpoint {
            self.corners[0][0] - corner[0]
        } else {
            self.corners[1][0] - corner[0]
        };

        let y_val = if corner[1] <= y_midpoint {
            self.corners[0][1] - corner[1]
        } else {
            self.corners[1][1] - corner[1]
        };
        if x_val.abs() < y_val.abs() { [x_val, 0.0] } else { [0.0, y_val] }
    }

    pub fn shortest_manhattan_move(&self, bbox: &BoundingBox2D) -> Option<Vector2> {
        // Check from this side
        let first_result = self.shortest_manhattan_one_side(bbox);
        // Check from the other side
        let pre_second_result = bbox.shortest_manhattan_one_side(self);
        // Invert the direction of the move as from this side it should move out the other way.
        let second_result = match pre_second_result {
            Some(result) => Some([-result[0], -result[1]]),
            None => None
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

    pub(self) fn shortest_manhattan_one_side(&self, bbox: &BoundingBox2D) -> Option<Vector2> {
        let bbox_top_left = [bbox.corners[0][0], bbox.corners[1][1]];
        let bbox_bottom_right = [bbox.corners[1][0], bbox.corners[0][1]];
        if self.contains_strict(bbox.corners[0]) {
            Some(self.shortest_from_corner(bbox.corners[0]))
        } else if self.contains_strict(bbox.corners[1]) {
            Some(self.shortest_from_corner(bbox.corners[1]))
        } else if self.contains_strict(bbox_top_left) {
            Some(self.shortest_from_corner(bbox_top_left))
        } else if self.contains_strict(bbox_bottom_right) {
            Some(self.shortest_from_corner(bbox_bottom_right))
        } else {
            None
        }
    }

    pub fn intersects(&self, bbox: &BoundingBox2D) -> bool {
        self.intersect_one_box(bbox) || bbox.intersect_one_box(self)
    }

    pub(self) fn intersect_one_box(&self, bbox: &BoundingBox2D) -> bool {
        bbox.contains(self.corners[0])
            || bbox.contains(self.corners[1])
            || bbox.contains([self.corners[0][0], self.corners[1][1]])
            || bbox.contains([self.corners[1][0], self.corners[0][1]])
    }

    pub fn contains_strict(&self, point: Vector2) -> bool {
        point[0] > self.corners[0][0]
            && point[0] < self.corners[1][0]
            && point[1] > self.corners[0][1]
            && point[1] < self.corners[1][1]
    }

    pub fn contains(&self, point: Vector2) -> bool {
        point[0] >= self.corners[0][0]
            && point[0] <= self.corners[1][0]
            && point[1] >= self.corners[0][1]
            && point[1] <= self.corners[1][1]
    }
}
