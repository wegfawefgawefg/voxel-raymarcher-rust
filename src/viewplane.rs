use crate::camera::Camera;
use glam::{Vec2, Vec3};

#[derive(Debug)]
pub struct Viewplane {
    pub size: Vec2,
}

impl Viewplane {
    pub fn new(size: Vec2, target_aspect_ratio: f32) -> Self {
        let viewplane = Self { size };
        viewplane.validate_aspect_ratio(target_aspect_ratio);
        viewplane
    }

    pub fn top_left_corner_from_perspective_of(&self, camera: &Camera) -> Vec3 {
        let half_size = self.size / 2.0;
        let center = camera.pos + camera.dir * camera.viewplane_distance;
        let right = camera.dir.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(camera.dir).normalize();
        center - right * half_size.x + up * half_size.y
    }

    pub fn get_right_from_perspective_of(&self, camera: &Camera) -> Vec3 {
        camera.dir.cross(Vec3::new(0.0, 1.0, 0.0)).normalize()
    }

    pub fn get_down_from_perspective_of(&self, camera: &Camera) -> Vec3 {
        let right = self.get_right_from_perspective_of(camera);
        right.cross(camera.dir).normalize() * -1.0
    }

    pub fn validate_aspect_ratio(&self, ratio: f32) {
        let aspect_ratio = self.size.x / self.size.y;
        if (aspect_ratio - ratio).abs() > 1e-2 {
            panic!("Aspect ratio {} does not match {}", aspect_ratio, ratio);
        }
    }

    pub fn get_targets(&self, camera: &Camera, resolution: Vec2) -> Vec<Vec3> {
        let mut targets = Vec::new();

        let tl = self.top_left_corner_from_perspective_of(camera);
        let right = self.get_right_from_perspective_of(camera);
        let down = self.get_down_from_perspective_of(camera);
        let pixel_size = self.size / resolution;
        let half_pixel = pixel_size / 2.0;
        let mut t = tl + right * half_pixel.x + down * half_pixel.y;

        for _ in 0..resolution.y as usize {
            let row_start = t;
            for _ in 0..resolution.x as usize {
                targets.push(t);
                t += right * pixel_size.x;
            }
            t = row_start + down * pixel_size.y;
        }

        targets
    }
}
