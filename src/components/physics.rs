use amethyst::core::math::Vector2;
use serde::{Deserialize, Serialize};

/// Accelerate in a direction and return new velocity in that direction.
pub fn accelerate1d(speed: f32, accel: f32, time_step: f32) -> f32 {
    speed + accel * time_step
}

/// Decelerate against the current velocity direction and return the new velocity.
pub fn decelerate1d(speed: f32, decel: f32, time_step: f32) -> f32 {
    let original_sign = speed.signum();
    original_sign * (speed.abs() - decel * time_step).max(0.0)
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Corners {
    pub bottom_left: Vector2<f32>,
    pub top_right: Vector2<f32>,
}

impl Corners {
    #[inline]
    pub fn left(&self) -> f32 {
        self.bottom_left.x
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.bottom_left.y
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.top_right.x
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.top_right.y
    }

    #[inline]
    pub fn bottom_left(&self) -> Vector2<f32> {
        self.bottom_left
    }

    #[inline]
    pub fn bottom_right(&self) -> Vector2<f32> {
        Vector2::new(self.top_right.x, self.bottom_left.y)
    }

    #[inline]
    pub fn top_left(&self) -> Vector2<f32> {
        Vector2::new(self.bottom_left.x, self.top_right.y)
    }

    #[inline]
    pub fn top_right(&self) -> Vector2<f32> {
        self.top_right
    }

    #[inline]
    pub fn x_midpoint(&self) -> f32 {
        (self.right() + self.left()) / 2.0
    }

    #[inline]
    pub fn y_midpoint(&self) -> f32 {
        (self.top() + self.bottom()) / 2.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BoundingBox2D {
    pub corners: Corners,
}

pub const MINIMUM_CLIP: f32 = 0.01;

impl BoundingBox2D {
    pub(self) fn manhattan_move(&self, bbox: &BoundingBox2D) -> Option<Vector2<f32>> {
        if !self.intersects(&bbox) {
            return None
        }

        let x_val = {
            let right = bbox.corners.left() - self.corners.right();
            let left = bbox.corners.right() - self.corners.left();
            if right.abs() > left.abs() {
                left
            } else {
                right
            }
        };

        let y_val = {
            let top = bbox.corners.bottom() - self.corners.top();
            let bottom = bbox.corners.top() - self.corners.bottom();
            if top.abs() > bottom.abs() {
                bottom
            } else {
                top
            }
        };

        Some(Vector2::new(x_val, y_val))
    }

    pub fn translate(&self, point: Vector2<f32>) -> BoundingBox2D {
        BoundingBox2D {
            corners: Corners {
                bottom_left: self.corners.bottom_left() + point,
                top_right: self.corners.top_right() + point,
            },
        }
    }

    /// Finds the move which minimizes distance and maximizes for the direction of the velocity.
    pub fn shortest_manhattan_move(
        &self,
        bbox: &BoundingBox2D,
        trajectory: Vector2<f32>,
    ) -> Option<Vector2<f32>> {
        // Check from this side
        let intersection_vector = self.manhattan_move(bbox)?;

        let hypotenuse = (trajectory.x.powf(2.) + trajectory.y.powf(2.)).sqrt();
        let direction_of_impact = if trajectory.x.abs() > hypotenuse * (2./3.) {
            Vector2::new(trajectory.x.signum(), 0.0)
        } else if trajectory.y.abs() > hypotenuse * (2./3.) {
            Vector2::new(0.0, trajectory.y.signum())
        } else {
            Vector2::new(0.0, 0.0)
        };
        println!("vec: {:?}, direction: {:?}", intersection_vector, direction_of_impact);

        // NOTE: this check is complicated because small impacts show up as small intersections,
        // but the other direction might be large if it is near the middle, however large impacts
        // show up as greater intersections in both directions and we must eject the collision in
        // the opposite direction of the impact. We have to heuristically determine whether it was
        // a small impact or a large impact.
        if intersection_vector.x.abs() > intersection_vector.y.abs() {
            // Horizontal intersection
            if direction_of_impact.x != 0.0 {
                Some(Vector2::new(intersection_vector.x, 0.0))
            } else {
                Some(Vector2::new(0.0, intersection_vector.y))
            }
        } else {
            // Vertical intersection
            if direction_of_impact.y != 0.0 {
                Some(Vector2::new(0.0, intersection_vector.y))
            } else {
                Some(Vector2::new(intersection_vector.x, 0.0))
            }
        }
    }

    pub fn intersects(&self, bbox: &BoundingBox2D) -> bool {
        !(self.corners.left() >= bbox.corners.right() ||
          self.corners.right() <= bbox.corners.left() ||
          self.corners.top() <= bbox.corners.bottom() ||
          self.corners.bottom() >= bbox.corners.top())
    }

    pub fn contains(&self, point: Vector2<f32>) -> bool {
        point.x >= self.corners.left()
            && point.x <= self.corners.right()
            && point.y >= self.corners.bottom()
            && point.y <= self.corners.top()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InverseBoundingBox2D {
    pub corners: Corners,
}

impl InverseBoundingBox2D {
    pub fn contains(&self, point: Vector2<f32>) -> bool {
        point.x < self.corners.left()
            || point.x > self.corners.right()
            || point.y < self.corners.bottom()
            || point.y > self.corners.top()
    }

    pub fn shortest_manhattan_move(&self, bbox: &BoundingBox2D) -> Option<Vector2<f32>> {
        let point = if self.contains(bbox.corners.bottom_left()) {
            bbox.corners.bottom_left()
        } else if self.contains(bbox.corners.top_right()) {
            bbox.corners.top_right()
        } else if self.contains(bbox.corners.bottom_right()) {
            bbox.corners.bottom_right()
        } else if self.contains(bbox.corners.top_left()) {
            bbox.corners.top_left()
        } else {
            return None;
        };

        let x = if point.x < self.corners.left() {
            self.corners.left() - point.x
        } else if point.x > self.corners.right() {
            self.corners.right() - point.x
        } else {
            0.0
        };

        let y = if point.y > self.corners.top() {
            self.corners.top() - point.y
        } else if point.y < self.corners.bottom() {
            self.corners.bottom() - point.y
        } else {
            0.0
        };

        Some(Vector2::new(x, y))
    }
}
