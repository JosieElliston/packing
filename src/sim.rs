use std::ops;

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

/// a oriented edge.
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    from: Vec2,
    to: Vec2,
}

impl Edge {
    /// `None` if the nearest point is outside the edge segment.
    pub fn nearest_point(self, p: Vec2) -> Option<Vec2> {
        let edge_vec = self.to - self.from;
        let to_p = p - self.from;
        let t = to_p.dot(edge_vec) / edge_vec.length_sq();
        if (0.0..=1.0).contains(&t) {
            Some(self.from + edge_vec * t)
        } else {
            None
        }
    }

    /// negative if `p` is on the left side of the edge,
    /// positive if `p` is on the right side of the edge,
    /// 0 if `p` is on the edge.
    fn signed_distance(self, p: Vec2) -> Coord {
        let edge_vec = self.to - self.from;
        let out = edge_vec.cw().normalized();
        let to_p = p - self.from;
        to_p.dot(out)
    }

    /// basically the nearest point, but can be outside the edge segment.
    fn perpendicular_foot(self, p: Vec2) -> Vec2 {
        let edge_vec = self.to - self.from;
        let t = (p - self.from).dot(edge_vec) / edge_vec.length_sq();
        self.from + edge_vec * t
    }

    /// whether the perpendicular foot of `p` is in the edge segment.
    fn perpendicular_foot_in_segment(self, p: Vec2) -> bool {
        let edge_vec = self.to - self.from;
        let t = (p - self.from).dot(edge_vec) / edge_vec.length_sq();
        (0.0..=1.0).contains(&t)
    }

    fn distance_sq_if_perpendicular_foot_in_segment(self, p: Vec2) -> Option<Coord> {
        let edge_vec = self.to - self.from;
        let t = (p - self.from).dot(edge_vec);
        let length_sq = edge_vec.length_sq();
        // note that we don't include the endpoints, since those are handled by the vertex block.
        if t <= 0.0 || t >= length_sq {
            return None;
        }
        let t = t / length_sq;
        let foot = self.from + edge_vec * t;
        Some((p - foot).length_sq())
    }

    /// note that we include the endpoints.
    // TODO: be better.
    fn contains(self, p: Vec2, epsilon: Coord) -> bool {
        let edge_vec = self.to - self.from;
        let t = (p - self.from).dot(edge_vec);
        let length_sq = edge_vec.length_sq();
        if !(0.0..=length_sq).contains(&t) {
            return false;
        }
        let t = t / length_sq;
        let foot = self.from + edge_vec * t;
        (p - foot).length_sq() <= epsilon * epsilon
    }

    fn distance(self, p: Vec2) -> Coord {
        let edge_vec = self.to - self.from;
        let t = (p - self.from).dot(edge_vec);
        let length_sq = edge_vec.length_sq();
        let t = t / length_sq;
        let foot = self.from + edge_vec * t;
        (p - foot).length()
    }
}

/// the closest point on the square to a given point,
/// and the signed distance to that point.
/// the signed distance is always non-negative for `Vertex`.
pub enum NearestPoint {
    Vertex((Vec2, Coord)),
    Edge((Vec2, Coord)),
}

impl NearestPoint {
    pub fn inner(self) -> (Vec2, Coord) {
        match self {
            NearestPoint::Vertex(v) | NearestPoint::Edge(v) => v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementIndex {
    Vertex(usize),
    Edge(usize),
}

/// not axis-aligned, not centered at origin, side length `2 * self.normal.length()`.
#[derive(Debug, Clone, Copy)]
pub struct Square {
    pub center: Vec2,
    pub normal: Vec2,
    pub ccw: bool,
}

impl Square {
    pub fn vertices(self) -> [Vec2; 4] {
        let n = self.normal;
        let t = self.tangent();
        [
            self.center + n + t,
            self.center + n - t,
            self.center - n - t,
            self.center - n + t,
        ]
    }

    pub fn edges(self) -> [Edge; 4] {
        let vertices = self.vertices();
        [
            Edge {
                from: vertices[0],
                to: vertices[1],
            },
            Edge {
                from: vertices[1],
                to: vertices[2],
            },
            Edge {
                from: vertices[2],
                to: vertices[3],
            },
            Edge {
                from: vertices[3],
                to: vertices[0],
            },
        ]
    }

    pub fn enumerate_elements(self) -> impl Iterator<Item = ElementIndex> {
        (0..4)
            .map(ElementIndex::Vertex)
            .chain((0..4).map(ElementIndex::Edge))
    }

    pub fn get_vertex(self, i: usize) -> Vec2 {
        self.vertices()[i]
    }

    pub fn get_edge(self, i: usize) -> Edge {
        self.edges()[i]
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

    pub fn rad(self) -> Coord {
        self.normal.length()
    }

    pub fn tangent(self) -> Vec2 {
        if self.ccw {
            self.normal.ccw()
        } else {
            self.normal.cw()
        }
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
        self.center + self.normal + self.tangent()
    }

    // TODO: do this faster
    pub fn contains(self, p: Vec2) -> bool {
        let vertices = self.vertices();
        let mut sign = 0;
        for i in 0..4 {
            let a = vertices[i];
            let b = vertices[(i + 1) % 4];
            let edge = b - a;
            let to_p = p - a;
            let cross = edge.x * to_p.y - edge.y * to_p.x;
            if cross != 0.0 {
                if sign == 0 {
                    sign = if cross > 0.0 { 1 } else { -1 };
                } else if (cross > 0.0 && sign < 0) || (cross < 0.0 && sign > 0) {
                    return false;
                }
            }
        }
        true
    }

    pub fn nearest_point(self, p: Vec2) -> NearestPoint {
        let (closest_vertex, closest_vertex_distance_sq) = self
            .vertices()
            .iter()
            .map(|&v| (v, (v - p).length_sq()))
            .min_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .expect("distance squared should never be NaN")
            })
            .unwrap();

        if let Some((closest_edge_point, closest_edge_signed_distance)) = self
            .edges()
            .iter()
            .filter_map(|e| {
                let nearest_point = e.nearest_point(p)?;
                Some((nearest_point, (e.signed_distance(p))))
            })
            .min_by(|a, b| {
                a.1.abs()
                    .partial_cmp(&b.1.abs())
                    .expect("distance squared should never be NaN")
            })
        {
            if closest_vertex_distance_sq
                <= closest_edge_signed_distance * closest_edge_signed_distance
            {
                NearestPoint::Vertex((closest_vertex, closest_vertex_distance_sq.sqrt()))
            } else {
                NearestPoint::Edge((closest_edge_point, closest_edge_signed_distance))
            }
        } else {
            NearestPoint::Vertex((closest_vertex, closest_vertex_distance_sq.sqrt()))
        }
    }

    /// the normal vector for `p`.
    /// `None` if `p` is too far away.
    /// `p` doesn't need to lie on the boundary.
    /// `p` probably shouldn't lie on a vertex.
    /// ret has length TODO.
    // TODO: debug visualization of these normals.
    // TODO: debug visualization of the next step as a ghost.
    // TODO: what if the offset radius is larger than the square's radius? can probably just forbid that ig.
    pub fn gauss_map_offset_radius(self, p: Vec2, offset_radius: Coord) -> Option<Vec2> {
        let (nearest_point, signed_distance) = self.nearest_point(p).inner();
        if signed_distance == 0.0 {
            todo!();
            return None;
        }
        if signed_distance.abs() > offset_radius {
            return None;
        }
        let to_p = p - nearest_point;
        let normal = to_p / signed_distance;
        Some(normal)
    }

    /// whether `p` in the block of the `i`th vertex.
    /// note that this set is closed.
    /// note that if `p == vertex`, it's still in the block.
    pub fn vertex_block_contains(self, offset_radius: Coord, i: usize, p: Vec2) -> bool {
        let vertices = self.vertices();
        let vertex = vertices[i];
        let to_p = p - vertex;
        let distance_sq = to_p.length_sq();
        if distance_sq > offset_radius * offset_radius {
            return false;
        }
        for neighbor in [vertices[(i + 3) % 4], vertices[(i + 1) % 4]] {
            if to_p.dot(vertex - neighbor) < 0.0 {
                return false;
            }
        }
        true
    }

    /// whether `p` in the block of the `i`th edge.
    /// note that this set is not closed, since the endpoints are handled by the vertex blocks.
    /// note that if `p` lies on the interior of the edge, it's still in the block.
    pub fn edge_block_contains(self, offset_radius: Coord, i: usize, p: Vec2) -> bool {
        let edges = self.edges();
        let edge = edges[i];
        let Some(distance_sq) = edge.distance_sq_if_perpendicular_foot_in_segment(p) else {
            return false;
        };
        distance_sq <= offset_radius * offset_radius
    }

    fn after_impulse_builders(
        self,
        builders: impl IntoIterator<Item = SquareImpulseBuilder>,
    ) -> Self {
        let mut delta = SquareDelta::ZERO;
        for builder in builders {
            let (point, impulse) = match builder {
                SquareImpulseBuilder::Vertex {
                    vertex_index,
                    impulse,
                } => (self.get_vertex(vertex_index), impulse),
                SquareImpulseBuilder::Edge {
                    edge_index,
                    point,
                    impulse,
                } => (
                    self.get_edge(edge_index).nearest_point(point).unwrap(),
                    impulse,
                ),
            };
            let r = (point - self.center).normalized();
            // // if |impulse \cdot r| is large, the impulse is mostly linear.
            // // if |impulse \cdot r| is small, the impulse is mostly rotational.
            // delta += SquareDelta {
            //     center_delta: impulse.vector_proj(r),
            //     normal_delta: impulse.vector_proj(r.ccw()),
            // };
            let normal_delta = impulse.vector_proj(r.ccw());
            let new_normal = (self.normal + normal_delta).normalized() * self.normal.length();
            let normal_delta = new_normal - self.normal;
            delta += SquareDelta {
                // center_delta: impulse - normal_delta,
                center_delta: impulse,
                normal_delta,
            };
            // todo!("here");
        }
        self + delta
    }

    // fn impulse_on_vertex(self, i: usize, impulse: Vec2) -> SquareImpulse {
    //     let vertex = self.get_vertex(i);
    //     let r = vertex - self.center;
    //     SquareImpulse {
    //         linear: impulse,
    //         angular: r.x * impulse.y - r.y * impulse.x,
    //     }
    // }

    // /// point isn't on the edge.
    // fn impulse_on_edge(self, i: usize, point: Vec2, impulse: Vec2) -> SquareImpulse {
    //     let edge = self.get_edge(i);
    //     let nearest_point = edge.nearest_point(point).unwrap();
    //     let r = nearest_point - self.center;
    //     SquareImpulse {
    //         linear: impulse,
    //         angular: r.x * impulse.y - r.y * impulse.x,
    //     }
    // }
}

// #[derive(Debug, Clone, Copy)]
// struct SquareImpulse {
//     linear: Vec2,
//     angular: Coord,
// }

// impl ops::AddAssign for SquareImpulse {
//     fn add_assign(&mut self, rhs: Self) {
//         self.linear += rhs.linear;
//         self.angular += rhs.angular;
//     }
// }

// impl ops::Add<SquareImpulse> for SquareImpulse {
//     type Output = Self;

//     fn add(mut self, rhs: Self) -> Self {
//         self += rhs;
//         self
//     }
// }

#[derive(Debug, Clone, Copy)]
pub enum SquareImpulseBuilder {
    Vertex {
        vertex_index: usize,
        impulse: Vec2,
    },
    Edge {
        edge_index: usize,
        point: Vec2,
        impulse: Vec2,
    },
}

struct SquareDelta {
    center_delta: Vec2,
    normal_delta: Vec2,
}

impl SquareDelta {
    pub const ZERO: Self = Self {
        center_delta: Vec2::ZERO,
        normal_delta: Vec2::ZERO,
    };
}

impl ops::AddAssign<SquareDelta> for Square {
    fn add_assign(&mut self, rhs: SquareDelta) {
        self.center += rhs.center_delta;
        self.normal += rhs.normal_delta;
    }
}

impl ops::Add<SquareDelta> for Square {
    type Output = Self;

    fn add(mut self, rhs: SquareDelta) -> Self {
        self += rhs;
        self
    }
}

impl ops::AddAssign<SquareDelta> for SquareDelta {
    fn add_assign(&mut self, rhs: SquareDelta) {
        self.center_delta += rhs.center_delta;
        self.normal_delta += rhs.normal_delta;
    }
}

impl ops::Add<SquareDelta> for SquareDelta {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SquareIndex {
    Big,
    Small(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandleIndex {
    Dragging(SquareIndex),
    Resizing(SquareIndex),
    Rotating(SquareIndex),
}

impl HandleIndex {
    pub fn square_index(self) -> SquareIndex {
        match self {
            HandleIndex::Dragging(i) | HandleIndex::Resizing(i) | HandleIndex::Rotating(i) => i,
        }
    }
}

#[derive(Debug, Clone)]
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
                normal: Vec2::from_r(3.0),
                ccw: false,
            },
            small_squares: vec![
                Square {
                    center: Vec2 { x: -0.6, y: -0.6 },
                    normal: Vec2::from_theta(0.1),
                    ccw: true,
                },
                Square {
                    center: Vec2 { x: 0.6, y: 0.6 },
                    normal: Vec2::from_theta(0.3),
                    ccw: true,
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

    pub fn get_mut(&mut self, i: SquareIndex) -> &mut Square {
        match i {
            SquareIndex::Big => &mut self.big_square,
            SquareIndex::Small(i) => &mut self.small_squares[i],
        }
    }

    pub fn enumerate_squares(&self) -> impl Iterator<Item = (SquareIndex, &Square)> {
        std::iter::once((SquareIndex::Big, &self.big_square)).chain(
            self.small_squares
                .iter()
                .enumerate()
                .map(|(i, square)| (SquareIndex::Small(i), square)),
        )
    }

    pub fn enumerate_handles(&self) -> impl Iterator<Item = (HandleIndex, Vec2)> {
        self.enumerate_squares().flat_map(|(i, square)| {
            [
                (HandleIndex::Dragging(i), square.drag_handle()),
                (HandleIndex::Resizing(i), square.resize_handle()),
                (HandleIndex::Rotating(i), square.rotate_handle()),
            ]
        })
    }

    /// they call this a contact face set,
    /// but it's actually general elements.
    /// note that we skip the element if it contains the point,
    /// even if the block contains the point.
    // TODO: iterating over the enum indexes is really bad.
    fn contact_element_set(
        &self,
        contact_radius: Coord,
        v: Vec2,
    ) -> impl Iterator<Item = (SquareIndex, ElementIndex)> {
        self.enumerate_squares()
            .flat_map(move |(square_index, square)| {
                square
                    .enumerate_elements()
                    .filter(move |&element_index| match element_index {
                        ElementIndex::Vertex(i) => {
                            const EPSILON: Coord = 1e-6;
                            (square.get_vertex(i) - v).length_sq() > EPSILON * EPSILON
                                && square.vertex_block_contains(contact_radius, i, v)
                        }
                        ElementIndex::Edge(i) => {
                            const EPSILON: Coord = 1e-6;
                            !square.get_edge(i).contains(v, EPSILON)
                                && square.edge_block_contains(contact_radius, i, v)
                        }
                    })
                    .map(move |element_index| (square_index, element_index))
            })
    }

    // TODO: is contact_radius different from offset_radius?
    // TODO: optimize float operation ordering.
    // TODO: factor out into `struct Activation` to cache stuff.
    // inline to cache parameters during loops.
    #[inline(always)]
    fn activation(k_c: Coord, distance: Coord, contact_radius: Coord) -> Coord {
        if distance <= 0.0 {
            panic!("distance <= 0.0");
        }
        if distance > contact_radius {
            panic!("distance > contact_radius");
        }

        let tau = contact_radius / 2.0;
        if tau <= distance {
            let diff = contact_radius - distance;
            diff * diff * k_c / 2.0
        } else {
            let tau_sq = tau * tau;
            let k_c_prime = tau * k_c * tau_sq;
            let b = k_c * tau_sq / 2.0 + k_c_prime * tau.ln();
            -k_c_prime * distance.ln() + b
        }
    }

    fn normal_contact_energy(&self, k_c: Coord, contact_radius: Coord, v: Vec2) -> Coord {
        self.contact_element_set(contact_radius, v)
            .map(|(square_index, element_index)| {
                let square = self.get(square_index);
                let distance = match element_index {
                    ElementIndex::Vertex(i) => square.get_vertex(i).distance(v),
                    ElementIndex::Edge(i) => square.get_edge(i).distance(v),
                };
                Self::activation(k_c, distance, contact_radius)
            })
            .sum()
    }

    /// [OGC](https://graphics.cs.utah.edu/research/projects/ogc/Offset_Geometric_Contact-SIGGRAPH2025.pdf)
    /// https://ankachan.github.io/Projects/VertexBlockDescent/index.html
    #[cfg(false)]
    pub fn step(&mut self, k_c: Coord, contact_radius: Coord) {
        let mut impulses =
            vec![SquareImpulse::default(); self.small_squares.len()].into_boxed_slice();

        for (i, impulse) in impulses.into_iter().enumerate() {
            self.small_squares[i] = self.small_squares[i].after_impulse(impulse);
        }
    }

    pub fn get_impulse_builders(&self) -> Box<[Vec<SquareImpulseBuilder>]> {
        let mut builders = (0..self.small_squares.len())
            .map(|_| Vec::new())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        for (square_index, square) in self.enumerate_squares() {
            for (vertex_i, vertex) in square.vertices().into_iter().enumerate() {
                for (other_square_index, element_index) in self.contact_element_set(0.5, vertex) {
                    let other_square = self.get(other_square_index);
                    let signed_distance = match element_index {
                        ElementIndex::Vertex(i) => other_square.get_vertex(i).distance(vertex),
                        ElementIndex::Edge(i) => -other_square.get_edge(i).signed_distance(vertex),
                    };
                    let normal = (match element_index {
                        ElementIndex::Vertex(i) => other_square.get_vertex(i),
                        ElementIndex::Edge(i) => {
                            other_square.get_edge(i).perpendicular_foot(vertex)
                        }
                    } - vertex)
                        .normalized();
                    if let SquareIndex::Small(i) = square_index {
                        builders[i].push(SquareImpulseBuilder::Vertex {
                            vertex_index: vertex_i,
                            impulse: -normal * signed_distance,
                        });
                    }
                    if let SquareIndex::Small(i) = other_square_index {
                        match element_index {
                            ElementIndex::Vertex(vertex_i) => {
                                builders[i].push(SquareImpulseBuilder::Vertex {
                                    vertex_index: vertex_i,
                                    impulse: normal * signed_distance,
                                });
                            }
                            ElementIndex::Edge(edge_i) => {
                                // builders[i].push(SquareImpulseBuilder::Edge {
                                //     i: edge_i,
                                //     point: vertex,
                                //     impulse: normal * signed_distance,
                                // });
                            }
                        }
                    }
                }
            }
        }

        builders
    }

    #[cfg(false)]
    pub fn build_impulses(&self, builders: &[Vec<SquareImpulseBuilder>]) -> Box<[SquareImpulse]> {
        (self.small_squares.iter().zip(builders.iter()))
            .map(|(square, builders)| {
                let mut impulse = SquareImpulse::default();
                for &builder in builders {
                    match builder {
                        SquareImpulseBuilder::Vertex {
                            i,
                            impulse: vertex_impulse,
                        } => {
                            impulse += square.impulse_on_vertex(i, vertex_impulse);
                        }
                        SquareImpulseBuilder::Edge {
                            i,
                            point,
                            impulse: edge_impulse,
                        } => {
                            impulse += square.impulse_on_edge(i, point, edge_impulse);
                        }
                    }
                }
                impulse
            })
            .collect()
    }

    pub fn apply_impulse_builders(&mut self, builders: &[Vec<SquareImpulseBuilder>]) {
        assert_eq!(self.small_squares.len(), builders.len());
        for (square, square_builders) in self.small_squares.iter_mut().zip(builders.iter()) {
            // let mut impulse = SquareImpulse::ZERO;
            // for &builder in square_builders {
            //     match builder {
            //         SquareImpulseBuilder::Vertex {
            //             vertex_index: i,
            //             impulse: vertex_impulse,
            //         } => {
            //             impulse += square.impulse_on_vertex(i, vertex_impulse);
            //         }
            //         SquareImpulseBuilder::Edge {
            //             edge_index: i,
            //             point,
            //             impulse: edge_impulse,
            //         } => {
            //             impulse += square.impulse_on_edge(i, point, edge_impulse);
            //         }
            //     }
            // }
            // *square = square.after_impulse_builders(impulse);
            *square = square.after_impulse_builders(square_builders.iter().cloned());
        }
    }
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
