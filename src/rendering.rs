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
    render_width: u32,
    render_height: u32,
}

pub struct Renderer {
    pub dims: UVec2,
    texture: Texture2D,
    ray_buffer: Vec<u8>,
    upload_buffer: Vec<u8>,
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
            ray_buffer: vec![0; (dims.x * dims.y * 4) as usize],
            upload_buffer: vec![0; (dims.x * dims.y * 4) as usize],
            last_signature: None,
        }
    }

    pub fn draw_scene(&mut self, state: &mut State) {
        update_quality_scale(state);

        let render_width = ((self.dims.x as f32 * state.quality_scale).round() as u32)
            .max(1)
            .min(self.dims.x);
        let render_height = ((self.dims.y as f32 * state.quality_scale).round() as u32)
            .max(1)
            .min(self.dims.y);
        state.render_width = render_width;
        state.render_height = render_height;

        let signature = RenderSignature {
            world_revision: state.world.revision(),
            camera_pos: state.camera.pos,
            camera_dir: state.camera.dir,
            viewplane_size: state.viewplane.size,
            viewplane_distance: state.camera.viewplane_distance,
            draw_distance: state.draw_distance,
            march_step_size: state.march_step_size,
            render_width,
            render_height,
        };

        if self.last_signature == Some(signature) {
            return;
        }

        let ray_len = (render_width as usize)
            .saturating_mul(render_height as usize)
            .saturating_mul(4);
        if self.ray_buffer.len() != ray_len {
            self.ray_buffer.resize(ray_len, 0);
        }

        state.last_render_stats = raymarch::draw_voxels(
            RaymarchInput {
                world: &state.world,
                camera: &state.camera,
                viewplane: &state.viewplane,
                draw_distance: state.draw_distance,
                march_step_size: state.march_step_size,
            },
            &mut self.ray_buffer,
            render_width as i32,
            render_height as i32,
        );

        if render_width == self.dims.x && render_height == self.dims.y {
            self.texture.update_texture(&self.ray_buffer);
        } else {
            upscale_nearest_rgba(
                &self.ray_buffer,
                &mut self.upload_buffer,
                render_width as usize,
                render_height as usize,
                self.dims.x as usize,
                self.dims.y as usize,
            );
            self.texture.update_texture(&self.upload_buffer);
        }

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

#[inline]
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

fn upscale_nearest_rgba(
    src: &[u8],
    dst: &mut [u8],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
) {
    debug_assert_eq!(src.len(), src_w * src_h * 4);
    debug_assert_eq!(dst.len(), dst_w * dst_h * 4);

    for y in 0..dst_h {
        let sy = y * src_h / dst_h;
        for x in 0..dst_w {
            let sx = x * src_w / dst_w;
            let src_idx = (sy * src_w + sx) * 4;
            let dst_idx = (y * dst_w + x) * 4;
            dst[dst_idx] = src[src_idx];
            dst[dst_idx + 1] = src[src_idx + 1];
            dst[dst_idx + 2] = src[src_idx + 2];
            dst[dst_idx + 3] = 255;
        }
    }
}
