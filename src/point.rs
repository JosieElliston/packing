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
        // assert!(r >= 0.0, "not strictly needed but probably a bug");
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

    #[doc(alias = "angle")]
    #[doc(alias = "theta")]
    #[doc(alias = "atan2")]
    #[doc(alias = "phase")]
    pub fn arg(self) -> Coord {
        self.y.atan2(self.x)
    }

    pub fn dot(self, rhs: Self) -> Coord {
        self.x * rhs.x + self.y * rhs.y
    }

    pub fn length_sq(self) -> Coord {
        self.x * self.x + self.y * self.y
    }

    pub fn length(self) -> Coord {
        self.length_sq().sqrt()
    }

    pub fn distance_sq(self, rhs: Self) -> Coord {
        (self - rhs).length_sq()
    }

    pub fn distance(self, rhs: Self) -> Coord {
        (self - rhs).length()
    }

    /// `None` if the length is zero.
    pub fn normalized_checked(self) -> Result<Self, &'static str> {
        let length = self.length();
        if length == 0.0 {
            Err("cannot normalize zero-length vector")
        } else {
            Ok(self / length)
        }
    }

    /// probably won't panic on `length == 0` in release.
    pub fn normalized(self) -> Self {
        let length = self.length();
        debug_assert_ne!(length, 0.0, "cannot normalize zero-length vector");
        self / length
    }

    /// rotated 90 degrees counterclockwise.
    pub fn ccw(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    /// rotated 90 degrees clockwise.
    pub fn cw(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }

    /// scalar projection of `self` onto `rhs`.
    pub fn scalar_proj(self, rhs: Self) -> Coord {
        self.dot(rhs) / rhs.length()
    }

    /// vector projection of `self` onto `rhs`.
    pub fn vector_proj(self, rhs: Self) -> Self {
        rhs * (self.dot(rhs) / rhs.length_sq())
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

impl ops::Mul<Coord> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: Coord) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::MulAssign<Coord> for Vec2 {
    fn mul_assign(&mut self, rhs: Coord) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl ops::Mul<Vec2> for Coord {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl ops::Div<Coord> for Vec2 {
    type Output = Self;

    fn div(self, rhs: Coord) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl ops::DivAssign<Coord> for Vec2 {
    fn div_assign(&mut self, rhs: Coord) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
