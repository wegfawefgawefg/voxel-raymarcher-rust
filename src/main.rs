use glam::{UVec2, Vec3};
use raylib::prelude::*;
use raylib::{ffi::SetTraceLogLevel, prelude::TraceLogLevel};

mod camera;
mod controls;
mod raymarch;
mod rendering;
mod simulation;
mod state;
mod ui_overlay;
mod viewplane;
mod world;
mod world_generation;

const TIMESTEP: f32 = 1.0 / state::FRAMES_PER_SECOND as f32;
const DIMS: UVec2 = UVec2::new(240 / 2, 160 / 2);
const MARCH_STEP_SIZE: f32 = 0.2;
const UP: Vec3 = Vec3::new(0.0, -1.0, 0.0);
const WORLD_SIZE: usize = 256;

fn main() {
    let mut state = state::State::new();
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
    if state.mouse_look_locked {
        rl.disable_cursor();
    } else {
        rl.enable_cursor();
    }
    let mut renderer = rendering::Renderer::new(&mut rl, &rlt, DIMS);

    while state.running && !rl.window_should_close() {
        state.fps = rl.get_fps() as i32;

        controls::process_events_and_input(&mut rl, &mut state);

        let dt = rl.get_frame_time();
        state.time_since_last_update += dt;
        while state.time_since_last_update > TIMESTEP {
            state.time_since_last_update -= TIMESTEP;

            simulation::step(&mut rl, &mut state);
        }

        renderer.draw_scene(&mut state);

        let mut draw_handle = rl.begin_drawing(&rlt);
        draw_handle.clear_background(Color::BLACK);
        renderer.draw_to_window(&mut draw_handle, fullscreen, window_dims);
        rendering::draw_ui_overlay(&state, &mut draw_handle);
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
