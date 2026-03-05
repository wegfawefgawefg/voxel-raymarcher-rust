use glam::{Vec2, Vec3};

use crate::camera::Camera;
use crate::raymarch::{self, RenderStats};
use crate::viewplane::Viewplane;
use crate::world::{Block, World};
use crate::{DIMS, VOXEL_STEP_BUDGET, WORLD_SIZE};

pub const FRAMES_PER_SECOND: u32 = 60;
pub const DEFAULT_DRAW_DISTANCE: f32 = 128.0;

const MIN_DRAW_DISTANCE: f32 = 2.0;
const MAX_DRAW_DISTANCE: f32 = 2000.0;
const MIN_FOV_Y_DEG: f32 = 25.0;
const MAX_FOV_Y_DEG: f32 = 120.0;

#[derive(Debug, Eq, PartialEq)]
pub enum Mode {
    Orbit,
    Fly,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ResolutionScale {
    X1,
    XHalf,
    XQuarter,
    XEighth,
    XSixteenth,
    XThirtySecond,
}

impl ResolutionScale {
    #[inline]
    pub fn label(self) -> &'static str {
        match self {
            Self::X1 => "1x",
            Self::XHalf => "1/2x",
            Self::XQuarter => "1/4x",
            Self::XEighth => "1/8x",
            Self::XSixteenth => "1/16x",
            Self::XThirtySecond => "1/32x",
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct FrameTimings {
    pub simulation_ms: f32,
    pub raymarch_ms: f32,
    pub upload_ms: f32,
    pub frame_ms: f32,
    pub reused_render: bool,
}

pub struct State {
    pub running: bool,
    pub time_since_last_update: f32,

    pub world: Box<World>,
    pub camera: Box<Camera>,
    pub viewplane: Box<Viewplane>,

    pub mode: Mode,
    pub draw_distance: f32,
    pub voxel_step_budget: f32,
    pub fov_y_deg: f32,
    pub fps: i32,
    pub resolution_scale: ResolutionScale,
    pub render_width: u32,
    pub render_height: u32,
    pub chunk_gen_budget_per_step: usize,
    pub mouse_look_locked: bool,
    pub last_render_stats: RenderStats,
    pub last_frame_timings: FrameTimings,
}

impl State {
    pub fn new() -> Self {
        let mut world = Box::new(World::new(WORLD_SIZE));
        world.gen_floor(Block::new(255, 255, 255, 255));
        world.gen_cube(
            Vec3::new(1.0, world.get_above_floor_level() as f32 - 1.0, 1.0),
            Vec3::new(1.0, 2.0, 1.0),
            Block::new(255, 0, 0, 255),
        );

        let cube_color = Block::new(255, 255, 255, 255);
        world.gen_cube(
            Vec3::new(0.0, world.get_above_floor_level() as f32 - 1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            cube_color,
        );
        world.gen_cube(
            Vec3::new(
                0.0,
                world.get_above_floor_level() as f32 - 1.0,
                world.dim as f32 - 1.0,
            ),
            Vec3::new(1.0, 1.0, 1.0),
            cube_color,
        );
        world.gen_cube(
            Vec3::new(
                world.dim as f32 - 1.0,
                world.get_above_floor_level() as f32 - 1.0,
                0.0,
            ),
            Vec3::new(1.0, 1.0, 1.0),
            cube_color,
        );
        world.gen_cube(
            Vec3::new(
                world.dim as f32 - 1.0,
                world.get_above_floor_level() as f32 - 1.0,
                world.dim as f32 - 1.0,
            ),
            Vec3::new(1.0, 1.0, 1.0),
            cube_color,
        );

        let glass = Block::new(180, 220, 255, 85);
        world.gen_cube(
            Vec3::new(8.0, world.get_above_floor_level() as f32 - 10.0, 8.0),
            Vec3::new(1.0, 10.0, 8.0),
            glass,
        );

        let camera_pos = world.get_center();
        let camera_dir = Vec3::new(0.0, 0.2, 1.0).normalize();
        let camera = Box::new(Camera::new(camera_pos, camera_dir, 3.0));
        let viewplane = Box::new(Viewplane::new(Vec2::new(4.0, 3.0), 4.0 / 3.0));

        let fov_y_deg =
            (2.0 * ((viewplane.size.y * 0.5) / camera.viewplane_distance).atan()).to_degrees();

        Self {
            running: true,
            time_since_last_update: 0.0,
            world,
            camera,
            viewplane,
            mode: Mode::Fly,
            draw_distance: DEFAULT_DRAW_DISTANCE,
            voxel_step_budget: VOXEL_STEP_BUDGET,
            fov_y_deg,
            fps: 0,
            resolution_scale: ResolutionScale::XQuarter,
            render_width: DIMS.x,
            render_height: DIMS.y,
            chunk_gen_budget_per_step: 2,
            mouse_look_locked: true,
            last_render_stats: RenderStats::default(),
            last_frame_timings: FrameTimings::default(),
        }
    }

    pub fn current_fov_y_deg(&self) -> f32 {
        let distance = self.camera.viewplane_distance.max(0.001);
        (2.0 * ((self.viewplane.size.y * 0.5) / distance).atan()).to_degrees()
    }

    pub fn sync_fov_y_from_viewplane(&mut self) {
        self.fov_y_deg = self.current_fov_y_deg();
    }

    pub fn apply_fov_y_deg(&mut self, new_fov_y_deg: f32) {
        self.fov_y_deg = new_fov_y_deg.max(MIN_FOV_Y_DEG).min(MAX_FOV_Y_DEG);
        let distance = self.camera.viewplane_distance.max(0.001);
        let aspect = DIMS.x as f32 / DIMS.y as f32;
        let half_height = (self.fov_y_deg.to_radians() * 0.5).tan() * distance;
        let viewplane_height = half_height * 2.0;
        self.viewplane.size = Vec2::new(viewplane_height * aspect, viewplane_height);
    }

    pub fn clamp_render_budget(&mut self) {
        self.draw_distance = self
            .draw_distance
            .max(MIN_DRAW_DISTANCE)
            .min(MAX_DRAW_DISTANCE);
        self.voxel_step_budget = self
            .voxel_step_budget
            .max(raymarch::MIN_STEP_BUDGET)
            .min(raymarch::MAX_STEP_BUDGET);
        self.fov_y_deg = self.fov_y_deg.max(MIN_FOV_Y_DEG).min(MAX_FOV_Y_DEG);
    }
}
