use std::ops;

use eframe::egui;

use std::fmt;

use crate::point::{self, Coord};

type Real = Coord;
type Imag = Coord;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Window {
    real_lo: Real,
    real_hi: Real,
    imag_lo: Imag,
    imag_hi: Imag,
}
impl Default for Window {
    fn default() -> Self {
        Self {
            real_lo: -2.0,
            real_hi: 2.0,
            imag_lo: -2.0,
            imag_hi: 2.0,
        }
    }
}
impl Window {
    /// fails if the window would be empty,
    /// ie if it would have zero width or height.
    /// also fails if the window is too big, to avoid later overflow issues.
    pub fn from_lo_hi(real_lo: Real, real_hi: Real, imag_lo: Imag, imag_hi: Imag) -> Option<Self> {
        if !(real_lo < real_hi && imag_lo < imag_hi) {
            return None;
        }
        Some(Self {
            real_lo,
            real_hi,
            imag_lo,
            imag_hi,
        })
    }

    /// fails if the window would be empty,
    /// ie if it would have zero width or height.
    /// also fails if the window is too big, to avoid overflow issues.
    pub fn from_mid_rad(
        real_mid: Real,
        imag_mid: Imag,
        real_rad: Real,
        imag_rad: Imag,
    ) -> Option<Self> {
        assert!(real_rad > 0.0);
        assert!(imag_rad > 0.0);
        let real_lo = real_mid - real_rad;
        let real_hi = real_mid + real_rad;
        let imag_lo = imag_mid - imag_rad;
        let imag_hi = imag_mid + imag_rad;
        Self::from_lo_hi(real_lo, real_hi, imag_lo, imag_hi)
    }

    pub fn real_lo(self) -> Real {
        self.real_lo
    }
    pub fn real_hi(self) -> Real {
        self.real_hi
    }
    pub fn real_mid(self) -> Real {
        (self.real_hi + self.real_lo) / 2.0
    }
    pub fn real_rad(self) -> Real {
        (self.real_hi - self.real_lo) / 2.0
    }
    // pub fn real_mid_checked(self) -> Option<Real> {
    //     Some(self.real_hi.add_checked(self.real_lo)? / 2.0)
    // }
    // pub fn real_rad_checked(self) -> Option<Real> {
    //     Some(self.real_hi.sub_checked(self.real_lo)? / 2.0)
    // }

    pub fn imag_lo(self) -> Imag {
        self.imag_lo
    }
    pub fn imag_hi(self) -> Imag {
        self.imag_hi
    }
    pub fn imag_mid(self) -> Imag {
        (self.imag_hi + self.imag_lo) / 2.0
    }
    pub fn imag_rad(self) -> Imag {
        (self.imag_hi - self.imag_lo) / 2.0
    }
    // pub fn imag_mid_checked(self) -> Option<Imag> {
    //     Some(self.imag_hi.add_checked(self.imag_lo)? / 2.0)
    // }
    // pub fn imag_rad_checked(self) -> Option<Imag> {
    //     Some(self.imag_hi.sub_checked(self.imag_lo)? / 2.0)
    // }

    pub fn mid(self) -> point::Vec2 {
        point::Vec2 {
            x: self.real_mid(),
            y: self.imag_mid(),
        }
    }

    // pub fn area(self) -> f32 {
    //     (self.real_hi - self.real_lo) * (self.imag_hi - self.imag_lo)
    // }

    pub fn intersect(self, other: impl Into<Self>) -> Option<Self> {
        let other = other.into();
        let real_lo = Coord::max(self.real_lo, other.real_lo);
        let real_hi = Coord::min(self.real_hi, other.real_hi);
        let imag_lo = Coord::max(self.imag_lo, other.imag_lo);
        let imag_hi = Coord::min(self.imag_hi, other.imag_hi);
        Self::from_lo_hi(real_lo, real_hi, imag_lo, imag_hi)
    }

    pub fn overlaps(self, other: impl Into<Self>) -> bool {
        let other = other.into();
        let real_lo = Coord::max(self.real_lo, other.real_lo);
        let real_hi = Coord::min(self.real_hi, other.real_hi);
        let imag_lo = Coord::max(self.imag_lo, other.imag_lo);
        let imag_hi = Coord::min(self.imag_hi, other.imag_hi);
        real_lo <= real_hi && imag_lo <= imag_hi
    }

    pub fn contains(self, other: impl Into<Self>) -> bool {
        let other = other.into();
        self.real_lo <= other.real_lo
            && other.real_hi <= self.real_hi
            && self.imag_lo <= other.imag_lo
            && other.imag_hi <= self.imag_hi
    }

    pub fn contains_point(self, point::Vec2 { x: real, y: imag }: point::Vec2) -> bool {
        (self.real_lo..=self.real_hi).contains(&real)
            && (self.imag_lo..=self.imag_hi).contains(&imag)
    }
}
impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Window(real: [{}, {}], imag: [{}, {}])",
            self.real_lo, self.real_hi, self.imag_lo, self.imag_hi
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub real_mid: f64,
    pub imag_mid: f64,
    pub real_rad: f64,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            real_mid: 0.0,
            imag_mid: 0.0,
            real_rad: 2.0,
        }
    }
}
impl Camera {
    /// panics if `real_rad` is not positive
    // pub fn new(real_mid: Real, imag_mid: Imag, real_rad: Real) -> Self {
    pub fn new(real_mid: f64, imag_mid: f64, real_rad: f64) -> Self {
        // assert!(real_rad > 0.0);
        assert!(real_rad > 0.0);
        Self {
            real_mid,
            imag_mid,
            real_rad,
        }
    }

    pub fn real_lo(self) -> f64 {
        self.real_mid - self.real_rad
    }
    pub fn real_hi(self) -> f64 {
        self.real_mid + self.real_rad
    }
    pub fn imag_mid(self) -> f64 {
        self.imag_mid
    }
    pub fn real_mid(self) -> f64 {
        self.real_mid
    }
    pub fn real_rad(self) -> f64 {
        self.real_rad
    }
    pub fn real_rad_mut(&mut self) -> &mut f64 {
        &mut self.real_rad
    }
    pub fn mid(self) -> (f64, f64) {
        (self.real_mid, self.imag_mid)
    }
}
impl ops::AddAssign<(f64, f64)> for Camera {
    fn add_assign(&mut self, (real, imag): (f64, f64)) {
        self.real_mid += real;
        self.imag_mid += imag;
    }
}
impl ops::SubAssign<(f64, f64)> for Camera {
    fn sub_assign(&mut self, (real, imag): (f64, f64)) {
        self.real_mid -= real;
        self.imag_mid -= imag;
    }
}

#[derive(Debug, Clone)]
pub struct CameraMap {
    rect: egui::Rect,
    camera: Camera,
}
impl CameraMap {
    pub fn new(rect: egui::Rect, camera: Camera) -> Self {
        assert!(rect.min.x < rect.max.x);
        assert!(rect.min.y < rect.max.y);
        Self { rect, camera }
    }

    pub fn rect(&self) -> egui::Rect {
        self.rect
    }
    pub fn camera(&self) -> Camera {
        self.camera
    }
    /// equivalent to `self.rect_to_window(self.rect())`
    pub fn window(&self) -> Option<Window> {
        Window::from_lo_hi(
            self.camera.real_lo().try_into().ok()?,
            self.camera.real_hi().try_into().ok()?,
            self.imag_lo().try_into().ok()?,
            self.imag_hi().try_into().ok()?,
        )
    }

    pub fn imag_lo(&self) -> f64 {
        self.camera.imag_mid - self.imag_rad()
    }
    pub fn imag_hi(&self) -> f64 {
        self.camera.imag_mid + self.imag_rad()
    }
    pub fn imag_rad(&self) -> f64 {
        // self.camera
        //     .real_rad
        //     .mul_f64(self.rect.height() as f64 / self.rect.width() as f64)
        self.camera.real_rad * (self.rect.height() as f64 / self.rect.width() as f64)
    }

    /// returns `None` if it would be out of the fixed point domain
    pub fn x_to_real(&self, x: f32) -> Real {
        super::lerp_f64(
            self.camera.real_lo(),
            self.camera.real_hi(),
            super::inv_lerp_f64(self.rect.min.x as f64, self.rect.max.x as f64, x as f64),
        )
    }
    /// returns `None` if it would be out of the fixed point domain
    pub fn y_to_imag(&self, y: f32) -> Imag {
        super::lerp_f64(
            self.imag_lo(),
            self.imag_hi(),
            1.0 - super::inv_lerp_f64(self.rect.min.y as f64, self.rect.max.y as f64, y as f64),
        )
    }
    pub fn real_to_x(&self, real: Real) -> f32 {
        super::lerp_f64(
            self.rect.min.x as f64,
            self.rect.max.x as f64,
            super::inv_lerp_f64(self.camera.real_lo(), self.camera.real_hi(), real.into()),
        ) as f32
    }
    pub fn imag_to_y(&self, imag: Imag) -> f32 {
        super::lerp_f64(
            self.rect.min.y as f64,
            self.rect.max.y as f64,
            1.0 - super::inv_lerp_f64(self.imag_lo(), self.imag_hi(), imag.into()),
        ) as f32
    }

    pub fn screen_to_world(&self, pos: egui::Pos2) -> point::Vec2 {
        point::Vec2 {
            x: self.x_to_real(pos.x),
            y: self.y_to_imag(pos.y),
        }
    }
    pub fn world_to_screen(&self, point::Vec2 { x: real, y: imag }: point::Vec2) -> egui::Pos2 {
        egui::Pos2::new(self.real_to_x(real), self.imag_to_y(imag))
    }

    // pub fn vec1_to_delta_real(&self, vec1: f32) -> Option<Real> {
    //     (super::lerp_f64(
    //         0.0,
    //         2.0 * self.camera.real_rad,
    //         super::inv_lerp_f64(0.0, self.rect.width() as f64, vec1 as f64),
    //     ))
    // }
    // pub fn vec1_to_delta_imag(&self, vec1: f32) -> Option<Imag> {
    //     self.vec1_to_delta_real(-vec1)
    // }
    // pub fn vec2_to_delta_complex(&self, egui::vec2: egui::Vec2) -> Option<point::Vec2> {
    //     Some((
    //         self.vec1_to_delta_real(egui::vec2.x)?,
    //         self.vec1_to_delta_imag(egui::vec2.y)?,
    //     ))
    // }
    pub fn vec1_to_delta_real(&self, vec1: f32) -> f64 {
        super::lerp_f64(
            0.0,
            2.0 * self.camera.real_rad,
            super::inv_lerp_f64(0.0, self.rect.width() as f64, vec1 as f64),
        )
    }
    pub fn vec1_to_delta_imag(&self, vec1: f32) -> f64 {
        self.vec1_to_delta_real(-vec1)
    }
    pub fn vec2_to_delta_complex(&self, egui_vec2: egui::Vec2) -> (f64, f64) {
        (
            self.vec1_to_delta_real(egui_vec2.x),
            self.vec1_to_delta_imag(egui_vec2.y),
        )
    }
    /// equivalent to `self.real_to_x(fixed) - self.real_to_x(0.0)`
    /// and to `self.imag_to_y(0.0) - self.imag_to_y(fixed)`
    /// keywords: displacement, delta, difference, rad_to_vec1
    pub fn delta_real_to_vec1(&self, real: Real) -> f32 {
        super::lerp_f64(
            0.0,
            self.rect.width() as f64,
            super::inv_lerp_f64(0.0, 2.0 * self.camera.real_rad, real.into()),
        ) as f32
    }
    pub fn delta_imag_to_vec1(&self, imag: Imag) -> f32 {
        self.delta_real_to_vec1(-imag)
    }
    pub fn delta_complex_to_vec2(
        &self,
        point::Vec2 { x: real, y: imag }: point::Vec2,
    ) -> egui::Vec2 {
        egui::Vec2::new(self.delta_real_to_vec1(real), self.delta_imag_to_vec1(imag))
    }

    pub fn rect_to_window(&self, rect: egui::Rect) -> Option<Window> {
        Window::from_lo_hi(
            self.x_to_real(rect.min.x),
            self.x_to_real(rect.max.x),
            self.y_to_imag(rect.max.y),
            self.y_to_imag(rect.min.y),
        )
    }
    pub fn window_to_rect(&self, window: impl Into<Window>) -> egui::Rect {
        let window = window.into();
        egui::Rect {
            min: self.world_to_screen(point::Vec2 {
                x: window.real_lo(),
                y: window.imag_hi(),
            }),
            max: self.world_to_screen(point::Vec2 {
                x: window.real_hi(),
                y: window.imag_lo(),
            }),
        }
    }

    // pub fn pixels_width(&self) -> usize {
    //     let stride = self.stride.unwrap().get();
    //     let ret = (self.rect.width() as usize).div_ceil(stride);
    //     #[cfg(debug_assertions)]
    //     if let Some(line) = self.pixels().next() {
    //         debug_assert_eq!(ret, line.count());
    //     }
    //     ret
    // }
    // pub fn pixels_height(&self) -> usize {
    //     let stride = self.stride.unwrap().get();
    //     let ret = (self.rect.height() as usize).div_ceil(stride);
    //     debug_assert_eq!(ret, self.pixels().count());
    //     ret
    // }

    // pub fn rect_at(&self, row: usize, col: usize) -> egui::Rect {
    //     let stride = self.stride.unwrap().get();
    //     egui::Rect::from_min_size(
    //         egui::Pos2::new(col as f32, row as f32) * stride as f32 + self.rect.min.to_egui::vec2(),
    //         egui::Vec2::new(stride as f32, stride as f32),
    //     )
    // }
    // pub fn pixel_at(&self, row: usize, col: usize) -> Option<Pixel> {
    //     let stride = self.stride.unwrap().get();
    //     Pixel::from_lo_hi(
    //         self.x_to_real(col as f32)?,
    //         self.x_to_real((col + stride) as f32)?,
    //         self.y_to_imag((row + stride) as f32)?,
    //         self.y_to_imag(row as f32)?,
    //     )
    // }

    // /// pixel is None if it couldn't be constructed,
    // /// so it would be too small or outside the fixed point domain
    // pub fn pixels(&self) -> impl Iterator<Item = impl Iterator<Item = (egui::Rect, Option<Pixel>)>> {
    //     let stride = self.stride.unwrap().get();
    //     (0..self.rect.size().y as usize)
    //         .step_by(stride)
    //         .map(move |row| {
    //             (0..self.rect.size().x as usize)
    //                 .step_by(stride)
    //                 .map(move |col| (self.rect_at(row, col), self.pixel_at(row, col)))
    //         })
    // }

    pub fn pan_zoom(
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        camera: &mut Camera,
        velocity: &mut egui::Vec2,
    ) {
        let rect = ui.max_rect();
        let r = ui.allocate_rect(rect, egui::Sense::drag());
        let dt = ctx.input(|i| i.stable_dt);
        let camera_map = CameraMap::new(rect, *camera);

        // pan
        if r.is_pointer_button_down_on() && ctx.input(|i| i.pointer.primary_down()) {
            *camera += camera_map.vec2_to_delta_complex(-r.drag_delta());
            *velocity = -r.drag_delta() / dt;
        } else {
            const VELOCITY_DAMPING: f32 = 0.9999;
            *camera += camera_map.vec2_to_delta_complex(*velocity * dt);
            *velocity *= (1.0 - VELOCITY_DAMPING).powf(dt);
        }
        if velocity.length_sq() < 0.0001 {
            *velocity = egui::Vec2::ZERO;
        }

        // zoom
        if r.hovered()
            && let Some(mouse_pos) = r.hover_pos()
        {
            let mouse = mouse_pos - rect.center();
            let zoom = ctx.input(|i| (i.smooth_scroll_delta.y / 300.0).exp()) as f64;
            *camera += camera_map.vec2_to_delta_complex(mouse);
            // *camera.real_rad_mut() = camera_map
            //     .camera
            //     .real_rad()
            //     .mul_f64_saturating(zoom.recip());
            *camera.real_rad_mut() /= zoom;
            let camera_map = CameraMap::new(rect, *camera);
            *camera -= camera_map.vec2_to_delta_complex(mouse);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_rect_camera() -> (egui::Rect, Camera) {
        let rect =
            egui::Rect::from_min_max(egui::Pos2::new(0.0, 30.0), egui::Pos2::new(10.0, 50.0));
        let camera = Camera::new(1.0, 2.0, 1.0);
        (rect, camera)
    }

    #[test]
    fn test_bounds() {
        let (rect, camera) = get_rect_camera();
        let camera_map = CameraMap::new(rect, camera);

        assert_eq!(camera_map.camera.real_lo(), 0.0);
        assert_eq!(camera_map.camera.real_hi(), 2.0);
        assert_eq!(camera_map.imag_lo(), 0.0);
        assert_eq!(camera_map.imag_hi(), 4.0);

        assert!((0.0 - camera_map.real_to_x(0.0.try_into().unwrap())).abs() < 1e-4);
        assert!((10.0 - camera_map.real_to_x(2.0.try_into().unwrap())).abs() < 1e-4);
        assert!((30.0 - camera_map.imag_to_y(4.0.try_into().unwrap())).abs() < 1e-4);
        assert!((50.0 - camera_map.imag_to_y(0.0.try_into().unwrap())).abs() < 1e-4);

        assert_eq!(rect.min.x, camera_map.real_to_x(camera.real_lo()));
        assert_eq!(rect.max.x, camera_map.real_to_x(camera.real_hi()));
        assert_eq!(rect.max.y, camera_map.imag_to_y(camera_map.imag_lo()));
        assert_eq!(rect.min.y, camera_map.imag_to_y(camera_map.imag_hi()));

        for (p, c) in [
            (
                egui::Pos2::new(0.0, 30.0),
                (0.0.try_into().unwrap(), 4.0.try_into().unwrap()),
            ),
            (
                egui::Pos2::new(0.0, 50.0),
                (0.0.try_into().unwrap(), 0.0.try_into().unwrap()),
            ),
            (
                egui::Pos2::new(10.0, 30.0),
                (2.0.try_into().unwrap(), 4.0.try_into().unwrap()),
            ),
            (
                egui::Pos2::new(10.0, 50.0),
                (2.0.try_into().unwrap(), 0.0.try_into().unwrap()),
            ),
        ] {
            let c = point::Vec2 { x: c.0, y: c.1 };
            let c_actual = camera_map.screen_to_world(p);
            // assert!((c.0 - c_actual.0).abs() + (c.1 - c_actual.1).abs() < 1e-4);
            assert_eq!(c, c_actual);
            let p_actual = camera_map.world_to_screen(c);
            assert!((p - p_actual).length() < 1e-4);
        }
    }

    #[test]
    fn test_map_pos2() {
        let (rect, camera) = get_rect_camera();
        let camera_map = CameraMap::new(rect, camera);

        for pos in [
            egui::Pos2::new(1.0, 30.0),
            egui::Pos2::new(1.0, 50.0),
            egui::Pos2::new(10.0, 30.0),
            egui::Pos2::new(10.0, 50.0),
            egui::Pos2::new(9.871, 38.635),
            egui::Pos2::new(1.248, 45.656),
            egui::Pos2::new(3.463, 48.559),
            egui::Pos2::new(1.684, 32.323),
            egui::Pos2::new(2.809, 31.250),
            egui::Pos2::new(8.142, 36.146),
            egui::Pos2::new(3.938, 48.579),
            egui::Pos2::new(5.761, 42.575),
            egui::Pos2::new(9.691, 42.933),
            egui::Pos2::new(2.457, 30.097),
        ] {
            assert!(
                (pos - camera_map.world_to_screen(camera_map.screen_to_world(pos))).length() < 1e-4
            );
        }
        for c in [
            (-2.0, -1.0),
            (-2.0, 5.0),
            (4.0, -1.0),
            (4.0, 5.0),
            (-1.885, -0.978),
            (0.254, 0.793),
            (3.634, 3.274),
            (3.332, 1.716),
            (0.063, 3.933),
            (2.132, 1.927),
            (1.848, 4.781),
            (2.971, 4.047),
            (0.194, 2.966),
            (1.173, -0.435),
        ]
        .map(|(real, imag)| point::Vec2 { x: real, y: imag })
        {
            let c = point::Vec2 { x: c.x, y: c.y };
            let actual = camera_map.screen_to_world(camera_map.world_to_screen(c));
            // assert!((c.0 - actual.0).abs() + (c.1 - actual.1).abs() < 1e-4);
            // it's not precise enough for this to pass
            // assert_eq!(c, actual);
            assert!((c.x - actual.x).abs() < 1e-4);
            assert!((c.y - actual.y).abs() < 1e-4);
        }
    }

    #[test]
    fn test_window() {
        let (rect, camera) = get_rect_camera();
        let camera_map = CameraMap::new(rect, camera);

        let window = Window::from_lo_hi(
            camera_map.camera.real_lo().try_into().unwrap(),
            camera_map.camera.real_hi().try_into().unwrap(),
            camera_map.imag_lo().try_into().unwrap(),
            camera_map.imag_hi().try_into().unwrap(),
        )
        .unwrap();
        assert_eq!(camera_map.rect, camera_map.window_to_rect(window));
        assert_eq!(window, camera_map.rect_to_window(rect).unwrap());
    }

    #[test]
    fn test_map_vec1() {
        let (rect, camera) = get_rect_camera();
        let camera_map = CameraMap::new(rect, camera);

        for fixed in [
            -2.0, -1.0, -2.0, 5.0, 4.0, -1.0, 4.0, 5.0, -1.885, -0.978, 0.254, 0.793, 3.634, 3.274,
            3.332, 1.716, 0.063, 3.933, 2.132, 1.927, 1.848, 4.781, 2.971, 4.047, 0.194, 2.966,
        ]
        .map(|fixed| fixed.try_into().unwrap())
        {
            let vec1_fixed = camera_map.delta_real_to_vec1(fixed);
            let vec1_real = camera_map.real_to_x(fixed) - camera_map.real_to_x(0.0);
            let vec1_imag = camera_map.imag_to_y(0.0) - camera_map.imag_to_y(fixed);
            assert!((vec1_fixed - vec1_real).abs() < 1e-4);
            assert!((vec1_fixed - vec1_imag).abs() < 1e-4);
        }
    }

    // #[test]
    // fn test_pixels() {
    //     let (rect, camera) = get_rect_camera();
    //     let camera_map = CameraMap::new(rect, camera, 2);

    //     assert_eq!(camera_map.pixels_width(), 5);
    //     assert_eq!(camera_map.pixels_height(), 10);

    //     assert_eq!(camera_map.pixels().count(), camera_map.pixels_height());
    //     assert_eq!(
    //         camera_map.pixels().next().unwrap().count(),
    //         camera_map.pixels_width()
    //     );
    // }
}
