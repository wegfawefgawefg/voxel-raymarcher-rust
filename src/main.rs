use glam::{UVec2, Vec3};
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
const WORLD_SIZE: usize = 256;

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
    state.mouse_scale = mouse_scale;
    if state.mouse_look_locked {
        rl.disable_cursor();
    } else {
        rl.enable_cursor();
    }

    let mut render_texture = rl
        .load_render_texture(&rlt, DIMS.x, DIMS.y)
        .unwrap_or_else(|e| {
            println!("Error creating render texture: {}", e);
            std::process::exit(1);
        });

    while state.running && !rl.window_should_close() {
        let current_screen_dims = UVec2::new(rl.get_screen_width() as u32, rl.get_screen_height() as u32);
        state.mouse_scale = DIMS.as_vec2() / current_screen_dims.as_vec2();
        rl.set_mouse_scale(state.mouse_scale.x, state.mouse_scale.y);
        state.fps = rl.get_fps() as i32;

        sketch::process_events_and_input(&mut rl, &mut state);

        let dt = rl.get_frame_time();
        state.time_since_last_update += dt;
        while state.time_since_last_update > TIMESTEP {
            state.time_since_last_update -= TIMESTEP;

            sketch::step(&mut rl, &mut state);
        }

        let mut draw_handle = rl.begin_drawing(&rlt);
        {
            let low_res_draw_handle =
                &mut draw_handle.begin_texture_mode(&rlt, &mut render_texture);
            low_res_draw_handle.clear_background(Color::BLACK);
            sketch::draw(&state, low_res_draw_handle);
        }
        scale_and_blit_render_texture_to_window(
            &mut draw_handle,
            &mut render_texture,
            fullscreen,
            window_dims,
        );
        sketch::draw_ui_overlay(&state, &mut draw_handle);
    }
}

pub fn center_window(rl: &mut raylib::RaylibHandle, window_dims: UVec2) {
    // Prefer the left-most monitor in a multi-monitor setup.
    let monitor_count = raylib::core::window::get_monitor_count();
    let mut target_monitor = raylib::core::window::get_current_monitor();
    let mut best_x = i32::MAX;
    for monitor in 0..monitor_count {
        let pos = raylib::core::window::get_monitor_position(monitor);
        let px = pos.x as i32;
        if px < best_x {
            best_x = px;
            target_monitor = monitor;
        }
    }

    let monitor_pos = raylib::core::window::get_monitor_position(target_monitor);
    let monitor_width = raylib::core::window::get_monitor_width(target_monitor);
    let monitor_height = raylib::core::window::get_monitor_height(target_monitor);
    let x = monitor_pos.x as i32 + (monitor_width - window_dims.x as i32) / 2;
    let y = monitor_pos.y as i32 + (monitor_height - window_dims.y as i32) / 2;
    rl.set_window_position(x, y);
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
