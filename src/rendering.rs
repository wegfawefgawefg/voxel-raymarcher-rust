use glam::{UVec2, Vec2, Vec3};
use raylib::prelude::*;

use crate::raymarch::{self, RaymarchInput};
use crate::state::{State, TARGET_FPS};
use crate::ui_overlay;

const MIN_QUALITY_SCALE: f32 = 0.35;
const MAX_QUALITY_SCALE: f32 = 1.0;

#[derive(Copy, Clone, PartialEq)]
struct RenderSignature {
    world_revision: u64,
    camera_pos: Vec3,
    camera_dir: Vec3,
    viewplane_size: Vec2,
    viewplane_distance: f32,
    draw_distance: f32,
    march_step_size: f32,
}

pub struct Renderer {
    pub dims: UVec2,
    texture: Texture2D,
    framebuffer: Vec<u8>,
    last_signature: Option<RenderSignature>,
}

impl Renderer {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, dims: UVec2) -> Self {
        let image = Image::gen_image_color(dims.x as i32, dims.y as i32, Color::BLACK);
        let texture = rl
            .load_texture_from_image(thread, &image)
            .unwrap_or_else(|e| {
                println!("Error creating render texture: {}", e);
                std::process::exit(1);
            });

        Self {
            dims,
            texture,
            framebuffer: vec![0; (dims.x * dims.y * 4) as usize],
            last_signature: None,
        }
    }

    pub fn draw_scene(&mut self, state: &mut State) {
        update_quality_scale(state);

        let effective_draw_distance = state.draw_distance * state.quality_scale;
        let signature = RenderSignature {
            world_revision: state.world.revision(),
            camera_pos: state.camera.pos,
            camera_dir: state.camera.dir,
            viewplane_size: state.viewplane.size,
            viewplane_distance: state.camera.viewplane_distance,
            draw_distance: effective_draw_distance,
            march_step_size: state.march_step_size,
        };

        if self.last_signature == Some(signature) {
            return;
        }

        state.last_render_stats = raymarch::draw_voxels(
            RaymarchInput {
                world: &state.world,
                camera: &state.camera,
                viewplane: &state.viewplane,
                draw_distance: effective_draw_distance,
                march_step_size: state.march_step_size,
            },
            &mut self.framebuffer,
            self.dims.x as i32,
            self.dims.y as i32,
        );
        self.texture.update_texture(&self.framebuffer);
        self.last_signature = Some(signature);
    }

    pub fn draw_to_window(
        &self,
        draw_handle: &mut RaylibDrawHandle,
        fullscreen: bool,
        window_dims: UVec2,
    ) {
        let source_rec = Rectangle::new(
            0.0,
            0.0,
            self.texture.width() as f32,
            self.texture.height() as f32,
        );

        let dest_rec = if fullscreen {
            let screen_width = draw_handle.get_screen_width();
            let screen_height = draw_handle.get_screen_height();
            Rectangle::new(0.0, 0.0, screen_width as f32, screen_height as f32)
        } else {
            Rectangle::new(0.0, 0.0, window_dims.x as f32, window_dims.y as f32)
        };

        draw_handle.draw_texture_pro(
            &self.texture,
            source_rec,
            dest_rec,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
    }
}

pub fn draw_ui_overlay(state: &State, d: &mut RaylibDrawHandle) {
    ui_overlay::draw_ui_overlay(state, d);
}

fn update_quality_scale(state: &mut State) {
    if !state.auto_quality {
        state.quality_scale = state
            .quality_scale
            .clamp(MIN_QUALITY_SCALE, MAX_QUALITY_SCALE);
        return;
    }

    if state.fps < TARGET_FPS - 2 {
        state.quality_scale *= 0.97;
    } else if state.fps > TARGET_FPS + 8 {
        state.quality_scale *= 1.01;
    }

    state.quality_scale = state
        .quality_scale
        .clamp(MIN_QUALITY_SCALE, MAX_QUALITY_SCALE);
}
