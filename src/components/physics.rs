use amethyst::{
    core::{math::Vector2, Time, Transform},
    ecs::{
        Component, DenseVecStorage, Join, Read, ReadStorage, System, SystemData, WriteStorage,
    },
};
use crate::geometry::{segment_intersection, Corners, IntersectionMode};

/// Accelerate in a direction and return new velocity in that direction.
pub fn accelerate1d(speed: f32, accel: f32, time_step: f32) -> f32 {
    speed + accel * time_step
}

/// Decelerate against the current velocity direction and return the new velocity.
pub fn decelerate1d(speed: f32, decel: f32, time_step: f32) -> f32 {
    let original_sign = speed.signum();
    original_sign * (speed.abs() - decel * time_step).max(0.0)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BoundingBox2D {
    pub corners: Corners,
}

pub const MINIMUM_CLIP: f32 = 0.01;

impl BoundingBox2D {
    pub fn new(corner: Vector2<f32>, width:f32, height: f32) -> Self {
        Self{
            corners: Corners {
                bottom_left: corner,
                top_right: Vector2::new(corner.x + width, corner.y + height),
            },
        }
    }

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

    pub fn intersects_with_segment(&self, line_segment: [Vector2<f32>; 2]) -> Option<Vector2<f32>> {
        let sides: [[Vector2<f32>; 2]; 4] = [
            [Vector2::new(self.corners.left(), self.corners.bottom()),
             Vector2::new(self.corners.right(), self.corners.bottom())],
            [Vector2::new(self.corners.left(), self.corners.bottom()),
             Vector2::new(self.corners.left(), self.corners.top())],
            [Vector2::new(self.corners.right(), self.corners.bottom()),
             Vector2::new(self.corners.right(), self.corners.top())],
            [Vector2::new(self.corners.left(), self.corners.top()),
             Vector2::new(self.corners.right(), self.corners.top())]
        ];

        for &side in sides.into_iter() {
            if let Some(intersection) = segment_intersection(line_segment, side, IntersectionMode::ParallelDoesNotIntersect) {
                return Some(intersection);
            }
        }
        None
    }

    pub fn super_bounding_box(&self, bbox: BoundingBox2D) -> BoundingBox2D {
        let left_min = self.corners.left().min(bbox.corners.left());
        let right_max = self.corners.right().max(bbox.corners.right());
        let bottom_min = self.corners.bottom().min(bbox.corners.bottom());
        let top_max = self.corners.top().max(bbox.corners.top());
        BoundingBox2D{corners:
            Corners{bottom_left: Vector2::new(left_min, bottom_min),
                    top_right: Vector2::new(right_max, top_max)}
        }
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

impl Component for BoundingBox2D {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PhysicsBox {
    pub bbox: BoundingBox2D,
    pub velocity: Vector2<f32>,
}

impl PhysicsBox {
    pub fn new(bbox: BoundingBox2D) -> Self {
        Self{bbox, velocity: Vector2::new(0.0, 0.0)}
    }
}

impl Component for PhysicsBox {
    type Storage = DenseVecStorage<Self>;
}

pub fn euclidean_distance(line_segment: [Vector2<f32>; 2]) -> f32 {
    ((line_segment[1].y - line_segment[0].y).powf(2.0) +
     (line_segment[1].x - line_segment[0].x).powf(2.0)).sqrt()
}

#[derive(SystemDesc)]
pub struct MoveExecutionSystem;

impl<'s> System<'s> for MoveExecutionSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, BoundingBox2D>,
        WriteStorage<'s, PhysicsBox>,
        Read<'s, Time>
    );

    fn run(&mut self, (mut transforms, bounding_boxes, mut physics_boxes, time): Self::SystemData) {
        let time_step = time.delta_seconds();
        let mut physix = Vec::new();

        for (phys_transform, physics) in (&transforms, &physics_boxes).join() {
            let phys_pos = {let vec = phys_transform.translation(); Vector2::new(vec.x, vec.y)};
            let phys_box = physics.bbox.translate(phys_pos);
            let direction = physics.velocity * time_step;
            debug!("Phys: {:?} {:?}", phys_box, direction);
            physix.push((phys_box, direction));
        }

        let mut moves: Vec<Option<(Vector2<f32>, Vector2<f32>)>> = Vec::new();

        for (phys_box, direction) in physix {
            let expected_position = BoundingBox2D{corners:
                Corners{bottom_left: phys_box.corners.bottom_left() + direction,
                        top_right: phys_box.corners.top_right() + direction}
            };
            let covered_space = phys_box.super_bounding_box(expected_position);

            let ray_casts: Vec<[Vector2<f32>; 2]> = vec![
                [phys_box.corners.bottom_left(), phys_box.corners.bottom_left() + direction],
                [phys_box.corners.top_right(), phys_box.corners.top_right() + direction],
                [phys_box.corners.bottom_right(), phys_box.corners.bottom_right() + direction],
                [phys_box.corners.top_left(), phys_box.corners.top_left() + direction],
            ];

            'statics: for (static_transform, bbox) in (&transforms, &bounding_boxes).join() {
                let static_box = bbox.translate(
                    {let vec = static_transform.translation(); Vector2::new(vec.x, vec.y)});
                if static_box.intersects(&covered_space) {
                    for segment in &ray_casts {
                        if let Some(intersection_point) = static_box.intersects_with_segment(*segment) {
                            // 1. Find how far along the segment the intersection point is.
                            let total_distance = euclidean_distance(*segment);
                            let travelled_segment = [segment[0], segment[0] + intersection_point];
                            let travelled_distance = euclidean_distance(travelled_segment);
                            let remaining_distance_fraction = (total_distance - travelled_distance) / total_distance;
                            let remaining_time = remaining_distance_fraction * time_step;
                            // 2. Set a new velocity which is reversed and halved from the old.
                            // NOTE: This makes assumptions about the static box's orientation.
                            let new_velocity = -direction / 2.0;

                            // 3. Move the remaining distance at the new velocity.
                            let new_position = intersection_point + remaining_time * new_velocity;
                            moves.push(Some((new_velocity, new_position))); 
                            break 'statics;
                        }
                    }
                    // If we didn't break, add a nothing move.
                    moves.push(None);
                }
            }
        }

        for ((phys_transform, physics), phys_move) in (&mut transforms, &mut physics_boxes).join().zip(moves) {
            if let Some((new_velocity, new_position)) = phys_move {
                let phys_pos = {let vec = phys_transform.translation(); Vector2::new(vec.x, vec.y)};
                let difference = new_position - phys_pos;
                phys_transform.prepend_translation_x(difference.x);
                phys_transform.prepend_translation_y(difference.y);
                physics.velocity = new_velocity;
            } else {
                // Now move by the current velocity.
                phys_transform.prepend_translation_x(physics.velocity.x * time_step);
                phys_transform.prepend_translation_y(physics.velocity.y * time_step);
            }
        }
    }
}
