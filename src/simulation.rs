use glam::Vec3;
use raylib::prelude::*;

use crate::state::{Mode, State};
use crate::world::CHUNK_SIZE;

const MIN_CHUNK_GEN_RADIUS: i32 = 1;
const MAX_CHUNK_GEN_BUDGET: usize = 32;

pub fn step(rl: &mut RaylibHandle, state: &mut State) {
    let cam_chunk_x = (state.camera.pos.x / CHUNK_SIZE as f32).floor() as i32;
    let cam_chunk_z = (state.camera.pos.z / CHUNK_SIZE as f32).floor() as i32;
    let chunk_dim = state.world.chunk_dim as i32;
    let mut desired_chunk_radius = (state.draw_distance / CHUNK_SIZE as f32).ceil() as i32 + 1;
    desired_chunk_radius = desired_chunk_radius.max(MIN_CHUNK_GEN_RADIUS);
    desired_chunk_radius = desired_chunk_radius.min(chunk_dim.saturating_sub(1));

    let scaled_budget = ((desired_chunk_radius as usize) + 1).saturating_mul(2);
    let generation_budget = state
        .chunk_gen_budget_per_step
        .max(scaled_budget.min(MAX_CHUNK_GEN_BUDGET));

    let mut completed_columns = Vec::with_capacity(generation_budget);
    state
        .terrain_worker
        .drain_completed(generation_budget, &mut completed_columns);
    for column in completed_columns {
        if state
            .world
            .is_terrain_column_generated(column.chunk_x, column.chunk_z)
        {
            continue;
        }
        state
            .world
            .apply_terrain_column_heights(column.chunk_x, column.chunk_z, &column.surface_y);
    }

    let mut candidates: Vec<(i32, u32, u32)> = Vec::new();
    candidates.reserve(
        ((desired_chunk_radius * 2 + 1) as usize)
            .saturating_mul((desired_chunk_radius * 2 + 1) as usize),
    );

    for x in -desired_chunk_radius..=desired_chunk_radius {
        for z in -desired_chunk_radius..=desired_chunk_radius {
            let chunk_x = cam_chunk_x + x;
            let chunk_z = cam_chunk_z + z;
            if chunk_x < 0 || chunk_x >= chunk_dim || chunk_z < 0 || chunk_z >= chunk_dim {
                continue;
            }
            if state
                .world
                .is_terrain_column_generated(chunk_x as u32, chunk_z as u32)
            {
                continue;
            }
            let dist_sq = x * x + z * z;
            candidates.push((dist_sq, chunk_x as u32, chunk_z as u32));
        }
    }
    candidates.sort_unstable_by_key(|c| c.0);

    let mut queued = 0usize;
    for (_, chunk_x, chunk_z) in candidates {
        if queued >= generation_budget {
            break;
        }
        if state.terrain_worker.is_pending(chunk_x, chunk_z) {
            continue;
        }
        if state.terrain_worker.enqueue(chunk_x, chunk_z) {
            queued += 1;
            continue;
        }
        state.world.gen_terrain_column(chunk_x, chunk_z);
        queued += 1;
    }

    if state.mode == Mode::Orbit {
        let t = rl.get_time();
        let orbit_radius: f32 = 10.0;
        let orbit_center = state.world.get_center();
        let cam_height = state.camera.pos.y;
        state.camera.pos = Vec3::new(
            t.sin() as f32 * orbit_radius,
            0.0,
            t.cos() as f32 * orbit_radius,
        ) + orbit_center;
        state.camera.dir = (state.world.get_center() - state.camera.pos).normalize();
        state.camera.pos.y = cam_height;
    }
}
