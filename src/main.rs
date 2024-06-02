use std::collections::HashSet;

use glam::{IVec2, UVec2, UVec3, Vec3};
use raylib::prelude::*;
use raylib::{ffi::SetTraceLogLevel, prelude::TraceLogLevel};

mod camera;
mod sketch;
mod viewplane;
mod world;

const TIMESTEP: f32 = 1.0 / sketch::FRAMES_PER_SECOND as f32;
const DIMS: UVec2 = UVec2::new(240 / 2, 160 / 2);
const NUM_RAY_STEPS: i32 = 128;
const MARCH_STEP_SIZE: f32 = 0.2;
const UP: Vec3 = Vec3::new(0.0, -1.0, 0.0);
const CHUNK_GEN_RADIUS: i32 = 1;

fn main() {
    let mut state = sketch::State::new();
    let (mut rl, rlt) = raylib::init().title("Voxels").build();
    unsafe {
        SetTraceLogLevel(TraceLogLevel::LOG_WARNING as i32);
    }

    let window_dims = UVec2::new(1280, 720);
    let fullscreen = false;
    rl.set_window_size(window_dims.x as i32, window_dims.y as i32);
    if fullscreen {
        rl.toggle_fullscreen();
        rl.set_window_size(rl.get_screen_width(), rl.get_screen_height());
    }

    center_window(&mut rl, window_dims);
    let mouse_scale = DIMS.as_vec2() / window_dims.as_vec2();
    rl.set_mouse_scale(mouse_scale.x, mouse_scale.y);

    let mut render_texture = rl
        .load_render_texture(&rlt, DIMS.x, DIMS.y)
        .unwrap_or_else(|e| {
            println!("Error creating render texture: {}", e);
            std::process::exit(1);
        });

    while state.running && !rl.window_should_close() {
        sketch::process_events_and_input(&mut rl, &mut state);

        let dt = rl.get_frame_time();
        state.time_since_last_update += dt;
        while state.time_since_last_update > TIMESTEP {
            state.time_since_last_update -= TIMESTEP;

            sketch::step(&mut rl, &mut state);

            // if theres any chunks to generate, generate them
            // dont generate duplicates with a set
            let mut already_generated_chunks: HashSet<UVec3> = HashSet::new();
            for chunk_pos in state.chunks_to_generate.iter() {
                if !already_generated_chunks.contains(chunk_pos) {
                    state.world.gen_terrain(*chunk_pos);
                    already_generated_chunks.insert(*chunk_pos);
                }
            }
            state.chunks_to_generate.clear();
        }

        let mut draw_handle = rl.begin_drawing(&rlt);
        {
            let low_res_draw_handle =
                &mut draw_handle.begin_texture_mode(&rlt, &mut render_texture);
            low_res_draw_handle.clear_background(Color::BLACK);
            let chunks_to_generate = sketch::draw(&state, low_res_draw_handle);
            state.chunks_to_generate.clear();
            state.chunks_to_generate.extend(chunks_to_generate);
        }
        scale_and_blit_render_texture_to_window(
            &mut draw_handle,
            &mut render_texture,
            fullscreen,
            window_dims,
        );
    }
}

pub fn center_window(rl: &mut raylib::RaylibHandle, window_dims: UVec2) {
    let screen_dims = IVec2::new(rl.get_screen_width(), rl.get_screen_height());
    let screen_center = screen_dims / 2;
    let window_center = window_dims.as_ivec2() / 2;
    let offset = IVec2::new(screen_center.x, screen_center.y + window_center.y);
    rl.set_window_position(offset.x, offset.y);
    rl.set_target_fps(144);
}

pub fn scale_and_blit_render_texture_to_window(
    draw_handle: &mut RaylibDrawHandle,
    render_texture: &mut RenderTexture2D,
    fullscreen: bool,
    window_dims: UVec2,
) {
    let source_rec = Rectangle::new(
        0.0,
        0.0,
        render_texture.texture.width as f32,
        -render_texture.texture.height as f32,
    );
    // dest rec should be the fullscreen resolution if graphics.fullscreen, otherwise window_dims
    let dest_rec = if fullscreen {
        // get the fullscreen resolution
        let screen_width = draw_handle.get_screen_width();
        let screen_height = draw_handle.get_screen_height();
        Rectangle::new(0.0, 0.0, screen_width as f32, screen_height as f32)
    } else {
        Rectangle::new(0.0, 0.0, window_dims.x as f32, window_dims.y as f32)
    };

    let origin = Vector2::new(0.0, 0.0);

    draw_handle.draw_texture_pro(
        render_texture,
        source_rec,
        dest_rec,
        origin,
        0.0,
        Color::WHITE,
    );
}
