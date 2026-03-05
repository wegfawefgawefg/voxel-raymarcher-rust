use raylib::prelude::*;

use crate::app_state::State;
use crate::raymarch::{self, RaymarchInput};
use crate::ui_overlay;

pub fn draw_scene(state: &mut State, d: &mut RaylibTextureMode<RaylibDrawHandle>) {
    state.last_render_stats = raymarch::draw_voxels(
        RaymarchInput {
            world: &state.world,
            camera: &state.camera,
            viewplane: &state.viewplane,
            draw_distance: state.draw_distance,
            march_step_size: state.march_step_size,
        },
        d,
    );
}

pub fn draw_ui_overlay(state: &State, d: &mut RaylibDrawHandle) {
    ui_overlay::draw_ui_overlay(state, d);
}
