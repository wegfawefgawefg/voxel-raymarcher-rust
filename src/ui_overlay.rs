use glam::Vec2;
use raylib::prelude::*;

use crate::raymarch::{MAX_RAY_STEPS, MAX_STEP_SIZE, MIN_STEP_SIZE};
use crate::state::{Mode, State};
use crate::DIMS;

pub struct UiLayout {
    pub panel: Rectangle,
    pub dist_dec: Rectangle,
    pub dist_inc: Rectangle,
    pub step_dec: Rectangle,
    pub step_inc: Rectangle,
    pub fov_dec: Rectangle,
    pub fov_inc: Rectangle,
}

pub fn ui_layout(screen_width: i32, _screen_height: i32) -> UiLayout {
    let screen_w = screen_width.max(1) as f32;
    let screen_h = _screen_height.max(1) as f32;

    let margin = (screen_w * 0.012).clamp(10.0, 24.0);
    let panel_width = (screen_w * 0.23).clamp(280.0, 420.0);
    let panel_height = (screen_h * 0.42).clamp(280.0, 380.0);
    let panel_x = screen_w - panel_width - margin;
    let panel_y = margin;

    let padding = (panel_width * 0.04).clamp(10.0, 18.0);
    let button_w = (panel_width * 0.10).clamp(26.0, 36.0);
    let button_h = (panel_height * 0.075).clamp(20.0, 28.0);

    let header_h = (panel_height * 0.10).clamp(20.0, 32.0);
    let status_h = (panel_height * 0.10).clamp(20.0, 30.0);
    let controls_top = panel_y + padding + header_h + status_h + 8.0;
    let row_gap = (panel_height * 0.11).clamp(30.0, 42.0);

    let dist_row_y = controls_top;
    let step_row_y = dist_row_y + row_gap;
    let fov_row_y = step_row_y + row_gap;
    let inc_x = panel_x + panel_width - padding - button_w;
    let dec_x = inc_x - button_w - 6.0;

    UiLayout {
        panel: Rectangle::new(panel_x, panel_y, panel_width, panel_height),
        dist_dec: Rectangle::new(dec_x, dist_row_y, button_w, button_h),
        dist_inc: Rectangle::new(inc_x, dist_row_y, button_w, button_h),
        step_dec: Rectangle::new(dec_x, step_row_y, button_w, button_h),
        step_inc: Rectangle::new(inc_x, step_row_y, button_w, button_h),
        fov_dec: Rectangle::new(dec_x, fov_row_y, button_w, button_h),
        fov_inc: Rectangle::new(inc_x, fov_row_y, button_w, button_h),
    }
}

#[inline]
pub fn point_in_rect(p: Vec2, r: Rectangle) -> bool {
    p.x >= r.x && p.x <= r.x + r.width && p.y >= r.y && p.y <= r.y + r.height
}

fn current_fov_y_deg(state: &State) -> f32 {
    let distance = state.camera.viewplane_distance.max(0.001);
    (2.0 * ((state.viewplane.size.y * 0.5) / distance).atan()).to_degrees()
}

fn draw_button(d: &mut RaylibDrawHandle, rect: Rectangle, label: &str) {
    d.draw_rectangle(
        rect.x as i32,
        rect.y as i32,
        rect.width as i32,
        rect.height as i32,
        Color::new(40, 40, 40, 220),
    );
    d.draw_rectangle_lines(
        rect.x as i32,
        rect.y as i32,
        rect.width as i32,
        rect.height as i32,
        Color::new(200, 200, 200, 255),
    );
    d.draw_text(
        label,
        rect.x as i32 + 9,
        rect.y as i32 + 4,
        16,
        Color::WHITE,
    );
}

pub fn draw_ui_overlay(state: &State, d: &mut RaylibDrawHandle) {
    let screen_width = d.get_screen_width();
    let screen_height = d.get_screen_height();
    let layout = ui_layout(screen_width, screen_height);

    let step_size = state.march_step_size.max(MIN_STEP_SIZE).min(MAX_STEP_SIZE);
    let mut num_ray_steps = (state.draw_distance / step_size).ceil() as i32;
    num_ray_steps = num_ray_steps.max(1).min(MAX_RAY_STEPS);
    let draw_distance = num_ray_steps as f32 * step_size;
    let pixel_budget =
        state.render_width as i64 * state.render_height as i64 * num_ray_steps as i64;
    let fov_y_deg = current_fov_y_deg(state);
    let stats = state.last_render_stats;
    let avg_steps_per_ray = if stats.rays_cast > 0 {
        stats.voxel_steps as f32 / stats.rays_cast as f32
    } else {
        0.0
    };
    let panel_x = layout.panel.x as i32;
    let panel_y = layout.panel.y as i32;
    let pad = (layout.panel.width * 0.04).clamp(10.0, 18.0) as i32;
    let text_x = panel_x + pad;
    let metric_start_y = (layout.fov_dec.y + layout.fov_dec.height + (pad as f32 * 0.5)) as i32;

    d.draw_rectangle(
        panel_x,
        panel_y,
        layout.panel.width as i32,
        layout.panel.height as i32,
        Color::new(0, 0, 0, 170),
    );
    d.draw_rectangle_lines(
        panel_x,
        panel_y,
        layout.panel.width as i32,
        layout.panel.height as i32,
        Color::new(180, 180, 180, 255),
    );

    let mode_label = match state.mode {
        Mode::Orbit => "Orbit",
        Mode::Fly => "Fly",
    };

    d.draw_text("Perf Overlay", text_x, panel_y + pad - 2, 18, Color::WHITE);
    d.draw_text(
        &format!("FPS: {}", state.fps),
        text_x,
        panel_y + pad + 22,
        18,
        Color::GREEN,
    );
    d.draw_text(
        &format!("Scale: {}", state.resolution_scale.label()),
        text_x + (layout.panel.width as i32 / 2) - 8,
        panel_y + pad + 44,
        18,
        Color::WHITE,
    );
    d.draw_text(
        &format!("Mode: {}", mode_label),
        text_x + (layout.panel.width as i32 / 2) - 8,
        panel_y + pad + 22,
        18,
        Color::WHITE,
    );

    d.draw_text(
        &format!("Draw Dist: {:>6.2}", draw_distance),
        text_x,
        layout.dist_dec.y as i32 + 2,
        18,
        Color::WHITE,
    );
    d.draw_text(
        &format!("Step Size: {:>6.3}", step_size),
        text_x,
        layout.step_dec.y as i32 + 2,
        18,
        Color::WHITE,
    );
    d.draw_text(
        &format!("FOV Y: {:>6.2} deg", fov_y_deg),
        text_x,
        layout.fov_dec.y as i32 + 2,
        18,
        Color::WHITE,
    );
    d.draw_text(
        &format!("Render Scale: {}", state.resolution_scale.label()),
        text_x,
        metric_start_y,
        16,
        Color::new(220, 220, 220, 255),
    );
    d.draw_text(
        &format!(
            "Render Res: {}x{} / {}x{}",
            state.render_width, state.render_height, DIMS.x, DIMS.y
        ),
        text_x,
        metric_start_y + 20,
        16,
        Color::new(220, 220, 220, 255),
    );
    d.draw_text(
        &format!("Chunk Gen Budget: {}", state.chunk_gen_budget_per_step),
        text_x,
        metric_start_y + 40,
        16,
        Color::new(220, 220, 220, 255),
    );
    d.draw_text(
        &format!("Steps: {}  Budget: {}", num_ray_steps, pixel_budget),
        text_x,
        metric_start_y + 60,
        16,
        Color::new(200, 200, 200, 255),
    );
    d.draw_text(
        &format!("Rays: {}  Hits: {}", stats.rays_cast, stats.rays_hit),
        text_x,
        metric_start_y + 80,
        16,
        Color::new(200, 200, 200, 255),
    );
    d.draw_text(
        &format!(
            "Voxel Steps: {}  Avg/Ray: {:.2}",
            stats.voxel_steps, avg_steps_per_ray
        ),
        text_x,
        metric_start_y + 100,
        16,
        Color::new(200, 200, 200, 255),
    );
    d.draw_text(
        &format!("Empty Chunk Skips: {}", stats.empty_chunk_skips),
        text_x,
        metric_start_y + 120,
        16,
        Color::new(200, 200, 200, 255),
    );

    draw_button(d, layout.dist_dec, "-");
    draw_button(d, layout.dist_inc, "+");
    draw_button(d, layout.step_dec, "-");
    draw_button(d, layout.step_inc, "+");
    draw_button(d, layout.fov_dec, "-");
    draw_button(d, layout.fov_inc, "+");

    d.draw_text(
        "Keys: Tab, [-]/[+], [,]/[.], [[/]], F1..F6 Scale, F7/F8 Gen, Backspace",
        16,
        screen_height - 28,
        18,
        Color::new(220, 220, 220, 255),
    );

    d.draw_text(
        if state.mouse_look_locked {
            "Mouse Look: ON"
        } else {
            "Mouse Look: OFF"
        },
        text_x + (layout.panel.width as i32 / 2) - 8,
        panel_y + pad - 2,
        18,
        if state.mouse_look_locked {
            Color::GREEN
        } else {
            Color::WHITE
        },
    );

    let cx = screen_width / 2;
    let cy = screen_height / 2;
    d.draw_line(cx - 8, cy, cx + 8, cy, Color::GREEN);
    d.draw_line(cx, cy - 8, cx, cy + 8, Color::GREEN);
}
