use crate::{camera::Camera, UP};
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
        let right = self.get_right_from_perspective_of(camera);
        let up = self.get_up_from_perspective_of(camera);
        center - right * half_size.x + up * half_size.y
    }

    pub fn get_up_from_perspective_of(&self, camera: &Camera) -> Vec3 {
        let right = self.get_right_from_perspective_of(camera);
        right.cross(camera.dir).normalize()
    }

    pub fn get_right_from_perspective_of(&self, camera: &Camera) -> Vec3 {
        camera.dir.cross(UP).normalize()
    }

    pub fn get_down_from_perspective_of(&self, camera: &Camera) -> Vec3 {
        -self.get_up_from_perspective_of(camera)
    }

    pub fn validate_aspect_ratio(&self, ratio: f32) {
        let aspect_ratio = self.size.x / self.size.y;
        if (aspect_ratio - ratio).abs() > 1e-2 {
            panic!("Aspect ratio {} does not match {}", aspect_ratio, ratio);
        }
    }
}
