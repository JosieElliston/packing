use std::ops;

pub type Coord = f64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: Coord,
    pub y: Coord,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: Coord, y: Coord) -> Self {
        Self { x, y }
    }

    pub fn from_tuple(t: (Coord, Coord)) -> Self {
        Self { x: t.0, y: t.1 }
    }

    pub fn to_tuple(self) -> (Coord, Coord) {
        (self.x, self.y)
    }

    pub fn from_array([x, y]: [Coord; 2]) -> Self {
        Self { x, y }
    }

    pub fn to_array(self) -> [Coord; 2] {
        [self.x, self.y]
    }

    pub fn from_r_theta(r: Coord, theta: Coord) -> Self {
        assert!(r >= 0.0, "not strictly needed but probably a bug");
        use std::f64::consts::PI;
        assert!(
            (-2.0 * PI..=2.0 * PI).contains(&theta),
            "not strictly needed but probably a bug"
        );
        Self {
            x: r * theta.cos(),
            y: r * theta.sin(),
        }
    }

    /// pointing right.
    pub fn from_r(r: Coord) -> Self {
        Self::from_r_theta(r, 0.0)
    }

    /// with length 1.
    pub fn from_theta(theta: Coord) -> Self {
        Self::from_r_theta(1.0, theta)
    }

    pub fn length_sq(self) -> Coord {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> Coord {
        self.length_sq().sqrt()
    }
}

impl ops::Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl ops::Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
