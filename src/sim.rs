use crate::point::*;

/// axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub lo: Vec2,
    pub hi: Vec2,
}

impl AABB {
    pub fn real_rad(self) -> Coord {
        (self.hi.x - self.lo.x) / 2.0
    }

    pub fn width(self) -> Coord {
        self.hi.x - self.lo.x
    }

    pub fn imag_rad(self) -> Coord {
        (self.hi.y - self.lo.y) / 2.0
    }

    pub fn height(self) -> Coord {
        self.hi.y - self.lo.y
    }
}

/// not axis-aligned, not centered at origin, side length `2 * self.normal.length()`.
#[derive(Debug, Clone, Copy)]
pub struct Square {
    pub center: Vec2,
    pub normal: Vec2,
}

impl Square {
    pub fn vertices(self) -> [Vec2; 4] {
        let n = self.normal;
        let t = Vec2 { x: -n.y, y: n.x };
        [
            self.center + n + t,
            self.center + n - t,
            self.center - n - t,
            self.center - n + t,
        ]
    }

    pub fn aabb(self) -> AABB {
        let vertices = self.vertices();
        let mut lo = vertices[0];
        let mut hi = vertices[0];
        for v in &vertices[1..] {
            if v.x < lo.x {
                lo.x = v.x;
            }
            if v.y < lo.y {
                lo.y = v.y;
            }
            if v.x > hi.x {
                hi.x = v.x;
            }
            if v.y > hi.y {
                hi.y = v.y;
            }
        }
        AABB { lo, hi }
    }

    /// at the center
    pub fn drag_handle(self) -> Vec2 {
        self.center
    }

    /// at the middle of the right edge.
    pub fn resize_handle(self) -> Vec2 {
        self.center + self.normal
    }

    /// at the top right corner.
    pub fn rotate_handle(self) -> Vec2 {
        self.center
            + self.normal
            + Vec2 {
                x: -self.normal.y,
                y: self.normal.x,
            }
    }
}

// /// axis-aligned, centered at origin, side length `2 * self.rad`.
// struct BigSquare {
//     rad: Coord,
// }

// /// not axis-aligned, not centered at origin, side length 2.
// /// normal must have length 1.
// struct UnitSquare {
//     center: Vec2,
//     normal: Vec2,
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquareIndex {
    Big,
    Small(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleIndex {
    Dragging(SquareIndex),
    Resizing(SquareIndex),
    Rotating(SquareIndex),
}

pub struct Sim {
    // big_square: BigSquare,
    // unit_squares: Vec<UnitSquare>,
    pub big_square: Square,
    pub small_squares: Vec<Square>,
}

impl Sim {
    pub fn new() -> Self {
        Self {
            big_square: Square {
                center: Vec2::ZERO,
                normal: Vec2::from_r(2.0),
            },
            small_squares: vec![
                Square {
                    center: Vec2 { x: -0.5, y: -0.5 },
                    normal: Vec2::from_theta(0.1),
                },
                Square {
                    center: Vec2 { x: 0.5, y: 0.5 },
                    normal: Vec2::from_theta(0.3),
                },
            ],
        }
    }

    pub fn get(&self, i: SquareIndex) -> &Square {
        match i {
            SquareIndex::Big => &self.big_square,
            SquareIndex::Small(i) => &self.small_squares[i],
        }
    }

    // pub fn enumerate(&self) -> impl Iterator<Item = (SquareIndex, &Square)> {
    //     std::iter::once((SquareIndex::Big, &self.big_square)).chain(
    //         self.small_squares
    //             .iter()
    //             .enumerate()
    //             .map(|(i, square)| (SquareIndex::Small(i), square)),
    //     )
    // }

    pub fn enumerate_handles(&self) -> impl Iterator<Item = (HandleIndex, Vec2)> {
        std::iter::once((SquareIndex::Big, &self.big_square))
            .chain(
                self.small_squares
                    .iter()
                    .enumerate()
                    .map(|(i, square)| (SquareIndex::Small(i), square)),
            )
            .flat_map(|(i, square)| {
                [
                    (HandleIndex::Dragging(i), square.drag_handle()),
                    (HandleIndex::Resizing(i), square.resize_handle()),
                    (HandleIndex::Rotating(i), square.rotate_handle()),
                ]
            })
    }

    fn resolve(&mut self) {}
}

// pub use sim_egui::*;
// /// egui stuff
// mod sim_egui {
//     use eframe::egui;

//     use super::*;

//     impl Sim {
//         fn ui(&self, ui: &mut egui::Ui) {}
//     }
// }
