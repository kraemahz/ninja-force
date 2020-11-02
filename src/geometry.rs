use amethyst::core::math::Vector2;
use serde::{Deserialize, Serialize};

#[inline]
pub fn slope_and_offset(line: [Vector2<f32>; 2]) -> (f32, f32) {
    let rise = line[1].y - line[0].y;
    let run = line[1].x - line[0].x;
    let m = rise / run;  // Could be infinite
    let b = line[0].y - m * line[0].x;
    (m, b)
}


pub enum Intersection {
    Point(Vector2<f32>),
    Everywhere,
    None
}

/// Lines can intersect at a point, nowhere, or everywhere.
#[inline]
pub fn line_intersection(line0: (f32, f32), line1: (f32, f32)) -> Intersection {
    let dslope = line0.0 - line1.0;
    let doffset = line1.1 - line0.1;
    match (dslope, doffset) {
        (0.0, 0.0) => Intersection::Everywhere,
        (0.0, _) => Intersection::None,
        (dm, db) => {
            let x = db / dm;
            let y = line0.0 * x + line0.1;
            Intersection::Point(Vector2::new(x, y))
        }
    }
}

pub enum IntersectionMode {
    ParallelIntersects,
    ParallelDoesNotIntersect
}

#[inline]
pub fn segment_intersection(line0: [Vector2<f32>; 2], line1: [Vector2<f32>; 2], mode: IntersectionMode) -> Option<Vector2<f32>> {
    let params0 = slope_and_offset(line0);
    let params1 = slope_and_offset(line1);

    let sort_x = |line: [Vector2<f32>; 2]| -> (f32, f32) {
        if line[0].x < line[1].x {
            (line[0].x, line[1].x)
        } else {
            (line[1].x, line[0].x)
        }
    };

    match line_intersection(params0, params1) {
        Intersection::None => None,
        Intersection::Everywhere => {
            match mode {
                IntersectionMode::ParallelDoesNotIntersect => {
                    None
                },
                IntersectionMode::ParallelIntersects => {
                    // The segments still must overlap
                    
                    let (min_x0, max_x0) = sort_x(line0);

                    if line1[1].x >= min_x0 && line1[1].x <= max_x0 {
                        Some(line1[1])
                    } else if line1[0].x >= min_x0 && line1[0].x <= min_x0 {
                        Some(line1[0])
                    } else {
                        None
                    }
                }
            }
        }
        Intersection::Point(point) => {
            // Point must be within both segments
            let (min_x0, max_x0) = sort_x(line0);
            let (min_x1, max_x1) = sort_x(line1);
            if min_x0 <= point.x && max_x0 >= point.x && min_x1 <= point.x && max_x1 >= point.x {
                Some(point)
            } else {
                None
            }
        }
    }
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
