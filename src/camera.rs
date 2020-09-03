use glam::{vec2, vec3, vec4, Mat4, Vec2};
pub const MAX_ZOOM: f32 = 100000.0;
pub const INIT_ZOOM: f32 = 0.1;
pub const MIN_ZOOM: f32 = 0.001;

use std::ops::{Add, Mul};

fn lerp<T: Mul<f32, Output = T> + Add<T, Output = T> + Clone>(a: T, b: T, t: f32) -> T {
    a * t + b * (1. - t)
}

pub struct Camera {
    pub desired_position: Vec2,
    pub position2d: Vec2,
    pub desired_zoom: f32,
    pub zoom: f32,
}

impl Camera {
    pub fn new(init_zoom: f32, min_zoom: f32) -> Self {
        Camera {
            desired_zoom: init_zoom,
            zoom: min_zoom,
            ..Default::default()
        }
    }

    pub fn get_mvp(&self, aspect_ratio: f32) -> Mat4 {
        let w = 1. / self.zoom;
        let h = aspect_ratio / self.zoom;
        let proj = Mat4::orthographic_rh_gl(
            -w / 2., // left
            w / 2.,  // right
            -h / 2., // bottom
            h / 2.,  // top
            1.,      // near
            0.,      // far
        );
        let eye = vec3(self.position2d.x(), self.position2d.y(), 1.);
        let center = vec3(self.position2d.x(), self.position2d.y(), 0.0);
        let up = vec3(0.0, 1.0, 0.0);
        let view = Mat4::look_at_rh(eye, center, up);
        proj * view
    }

    pub fn update(&mut self) {
        self.zoom = lerp(self.zoom, self.desired_zoom, 0.8);
        self.position2d = lerp(self.position2d, self.desired_position, 0.4);
    }

    fn position_restrictions(&mut self, map_size: f32) {
        *self.desired_position.x_mut() = self
            .desired_position
            .x()
            .min(map_size as f32)
            .max(-map_size as f32);
        *self.desired_position.y_mut() = self
            .desired_position
            .y()
            .min(map_size as f32)
            .max(-map_size as f32);
    }

    pub fn position_set(&mut self, value: Vec2, map_size: f32) {
        self.desired_position = value;
        self.position_restrictions(map_size);
    }

    pub fn position_add(&mut self, delta: Vec2, map_size: f32) {
        self.desired_position += delta;
        self.position_restrictions(map_size);
    }

    pub fn zoom_set(&mut self, zoom: f32) {
        self.desired_zoom = zoom;
        self.desired_zoom = self.desired_zoom.min(MAX_ZOOM).max(MIN_ZOOM);
    }

    pub fn zoom_wheel(&mut self, y: f32) {
        self.desired_zoom *= f32::powf(1.2, y);
        self.desired_zoom = self.desired_zoom.min(MAX_ZOOM).max(MIN_ZOOM);
    }

    /// use only if it's needed once, cause it creates project matrix inside
    pub fn project(&self, point: Vec2, width: f32, height: f32) -> Vec2 {
        let mvp = self.get_mvp(height / width);
        let projected = mvp * vec4(point.x(), point.y(), 0., 1.);
        vec2(
            (projected.x() + 1.) * width / 2.,
            (1. - projected.y()) * height / 2.,
        )
    }

    /// x, y -- screen coordinates in pixels
    pub fn unproject(&self, x: f32, y: f32, width: f32, height: f32) -> Vec2 {
        // coords are in cube with corners [-1, -1, -1], [1, 1, 1] after orthographic projection
        let sx = -1. + 2. * x / width;
        let sy = 1. - 2. * y / height;
        // apply inverse matrix to point on a surface
        let unproject_pos = self.get_mvp(height / width).inverse() * vec4(sx, sy, 0., 1.);
        vec2(unproject_pos.x(), unproject_pos.y())
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            desired_position: vec2(0., 0.),
            position2d: vec2(0., 0.),
            desired_zoom: INIT_ZOOM,
            zoom: MIN_ZOOM,
        }
    }
}
