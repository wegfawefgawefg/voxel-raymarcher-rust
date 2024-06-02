use glam::{Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Camera {
    pub pos: Vec3,
    pub dir: Vec3,
    pub original_pos: Vec3,
    pub original_dir: Vec3,
    pub viewplane_distance: f32,
}

const UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

impl Camera {
    pub fn new(pos: Vec3, dir: Vec3, viewplane_distance: f32) -> Self {
        let normalized_dir = dir.normalize();
        Self {
            pos,
            dir: normalized_dir,
            original_pos: pos,
            original_dir: normalized_dir,
            viewplane_distance,
        }
    }

    pub fn reset(&mut self) {
        self.pos = self.original_pos;
        self.dir = self.original_dir;
    }

    pub fn get_right(&self) -> Vec3 {
        self.dir.cross(Vec3::new(0.0, 1.0, 0.0)).normalize()
    }

    pub fn get_down(&self) -> Vec3 {
        let right = self.get_right();
        right.cross(self.dir).normalize() * -1.0
    }

    pub fn get_up(&self) -> Vec3 {
        self.get_down() * -1.0
    }

    pub fn get_left(&self) -> Vec3 {
        self.get_right() * -1.0
    }

    pub fn rotate(&mut self, axis: Vec3, angle: f32) {
        let rotation = Quat::from_axis_angle(axis.normalize(), angle);
        self.dir = rotation * self.dir;
    }
}
