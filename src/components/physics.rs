/// Accelerate in a direction and return new velocity in that direction.
pub fn accelerate1d(speed: f32, accel: f32, time_step: f32) -> f32 {
    speed + accel * time_step
}

/// Decelerate against the current velocity direction and return the new velocity.
pub fn decelerate1d(speed: f32, decel: f32, time_step: f32) -> f32 {
    let original_sign = if speed > 0.0 {
        1.0
    } else if speed < 0.0 {
        -1.0
    } else {
        0.0
    };
    original_sign * (speed.abs() - decel * time_step).max(0.0)
}

pub type Point = [f32; 2];

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox2D {
    pub corners: [Point; 2],
}

impl BoundingBox2D {
    pub fn translate(&self, point: Point) -> BoundingBox2D {
        BoundingBox2D {
            corners: [
                [self.corners[0][0] + point[0], self.corners[0][1] + point[1]],
                [self.corners[1][0] + point[0], self.corners[1][1] + point[1]],
            ],
        }
    }

    pub fn intersects(&self, bbox: &BoundingBox2D) -> bool {
        self.intersects_one_side(bbox) || bbox.intersects_one_side(self)
    }

    pub(self) fn intersect_one_side(&self, bbox: &BoundingBox2D) -> bool {
        bbox.contains(self.corners[0])
            || bbox.contains(self.corners[1])
            || bbox.contains([self.corners[0][0], self.corners[1][1]])
            || bbox.contains([self.corners[1][0], self.corners[0][1]])
    }

    pub fn contains(&self, point: Point) -> bool {
        point[0] >= self.corners[0][0]
            && point[0] <= self.corners[1][0]
            && point[1] >= self.corners[0][1]
            && point[1] <= self.corners[1][1]
    }
}
