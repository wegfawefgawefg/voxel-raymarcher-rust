use glam::Vec2;
use raylib::prelude::*;

use crate::state::{Mode, State};
use crate::ui_overlay;
use crate::{MARCH_STEP_SIZE, NUM_RAY_STEPS, UP};

const DISTANCE_FACTOR: f32 = 1.1;
const STEP_FACTOR: f32 = 1.1;
const FOV_FACTOR: f32 = 1.05;
const QUALITY_FACTOR: f32 = 1.05;
const MOUSE_LOOK_SENSITIVITY: f32 = 0.0015;
const MAX_VIEW_ALIGNMENT_WITH_UP: f32 = 0.995;
const HIGH_SPEED_MULTIPLIER: f32 = 4.0;
const MIN_QUALITY_SCALE: f32 = 0.35;
const MAX_QUALITY_SCALE: f32 = 1.0;

pub fn process_events_and_input(rl: &mut RaylibHandle, state: &mut State) {
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE) {
        state.running = false;
    }

    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_M) {
        state.mode = match state.mode {
            Mode::Orbit => Mode::Fly,
            Mode::Fly => Mode::Orbit,
        };
        match state.mode {
            Mode::Fly => {
                state.mouse_look_locked = true;
                rl.disable_cursor();
            }
            Mode::Orbit => {
                state.mouse_look_locked = false;
                rl.enable_cursor();
            }
        }
    }

    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_R) {
        state.camera.reset();
    }

    if state.mode == Mode::Fly && rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_TAB) {
        state.mouse_look_locked = !state.mouse_look_locked;
        if state.mouse_look_locked {
            rl.disable_cursor();
        } else {
            rl.enable_cursor();
        }
    }

    let mut cam_speed = 0.1;
    let mut rotation_speed = 0.02;
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT_SHIFT) {
        cam_speed *= HIGH_SPEED_MULTIPLIER;
        rotation_speed *= HIGH_SPEED_MULTIPLIER;
    }

    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_W) {
        state.camera.pos += state.camera.dir * cam_speed;
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_S) {
        state.camera.pos -= state.camera.dir * cam_speed;
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_A) {
        state.camera.pos -= state.camera.get_right() * cam_speed;
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_D) {
        state.camera.pos += state.camera.get_right() * cam_speed;
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_SPACE) {
        state.camera.pos += state.camera.get_up() * cam_speed;
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT_CONTROL) {
        state.camera.pos -= state.camera.get_up() * cam_speed;
    }

    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_Q) {
        state.camera.rotate(UP, rotation_speed);
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_E) {
        state.camera.rotate(UP, -rotation_speed);
    }

    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_Y) {
        state.camera.dir.y += rotation_speed;
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_H) {
        state.camera.dir.y -= rotation_speed;
    }

    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_T) {
        state.camera.viewplane_distance -= cam_speed;
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_G) {
        state.camera.viewplane_distance += cam_speed;
    }

    state.sync_fov_y_from_viewplane();

    if state.mode == Mode::Fly && state.mouse_look_locked {
        let mouse_delta = rl.get_mouse_delta();
        let yaw = -mouse_delta.x * MOUSE_LOOK_SENSITIVITY;
        let pitch = -mouse_delta.y * MOUSE_LOOK_SENSITIVITY;

        if yaw != 0.0 {
            state.camera.rotate(UP, yaw);
        }
        if pitch != 0.0 {
            let old_dir = state.camera.dir;
            state.camera.rotate(state.camera.get_right(), pitch);
            if state.camera.dir.dot(UP).abs() > MAX_VIEW_ALIGNMENT_WITH_UP {
                state.camera.dir = old_dir;
            }
        }
        state.camera.dir = state.camera.dir.normalize();
    }

    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_MINUS) {
        state.draw_distance /= DISTANCE_FACTOR;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_EQUAL) {
        state.draw_distance *= DISTANCE_FACTOR;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_COMMA) {
        state.march_step_size /= STEP_FACTOR;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_PERIOD) {
        state.march_step_size *= STEP_FACTOR;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_BACKSPACE) {
        state.draw_distance = NUM_RAY_STEPS as f32 * MARCH_STEP_SIZE;
        state.march_step_size = MARCH_STEP_SIZE;
        state.apply_fov_y_deg(53.130104);
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_LEFT_BRACKET) {
        state.apply_fov_y_deg(state.fov_y_deg / FOV_FACTOR);
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_RIGHT_BRACKET) {
        state.apply_fov_y_deg(state.fov_y_deg * FOV_FACTOR);
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_F1) {
        state.auto_quality = !state.auto_quality;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_F2) {
        state.quality_scale /= QUALITY_FACTOR;
        state.auto_quality = false;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_F3) {
        state.quality_scale *= QUALITY_FACTOR;
        state.auto_quality = false;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_F4) {
        state.chunk_gen_budget_per_step = state.chunk_gen_budget_per_step.saturating_sub(1).max(1);
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_F5) {
        state.chunk_gen_budget_per_step = (state.chunk_gen_budget_per_step + 1).min(32);
    }
    state.clamp_render_budget();
    state.quality_scale = state
        .quality_scale
        .clamp(MIN_QUALITY_SCALE, MAX_QUALITY_SCALE);

    let screen_width = rl.get_screen_width();
    let screen_height = rl.get_screen_height();
    let mouse_pos = rl.get_mouse_position();
    let ui_mouse = Vec2::new(mouse_pos.x, mouse_pos.y);

    if !state.mouse_look_locked && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        let layout = ui_overlay::ui_layout(screen_width, screen_height);
        if ui_overlay::point_in_rect(ui_mouse, layout.dist_dec) {
            state.draw_distance /= DISTANCE_FACTOR;
        }
        if ui_overlay::point_in_rect(ui_mouse, layout.dist_inc) {
            state.draw_distance *= DISTANCE_FACTOR;
        }
        if ui_overlay::point_in_rect(ui_mouse, layout.step_dec) {
            state.march_step_size /= STEP_FACTOR;
        }
        if ui_overlay::point_in_rect(ui_mouse, layout.step_inc) {
            state.march_step_size *= STEP_FACTOR;
        }
        if ui_overlay::point_in_rect(ui_mouse, layout.fov_dec) {
            state.apply_fov_y_deg(state.fov_y_deg / FOV_FACTOR);
        }
        if ui_overlay::point_in_rect(ui_mouse, layout.fov_inc) {
            state.apply_fov_y_deg(state.fov_y_deg * FOV_FACTOR);
        }
        state.clamp_render_budget();
    }
}
