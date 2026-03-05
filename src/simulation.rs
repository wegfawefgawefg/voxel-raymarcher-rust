use glam::Vec3;
use raylib::prelude::*;

use crate::state::{Mode, State};
use crate::world::CHUNK_SIZE;
use crate::CHUNK_GEN_RADIUS;

pub fn step(rl: &mut RaylibHandle, state: &mut State) {
    let cam_chunk_x = (state.camera.pos.x / CHUNK_SIZE as f32).floor() as i32;
    let cam_chunk_z = (state.camera.pos.z / CHUNK_SIZE as f32).floor() as i32;
    let chunk_dim = state.world.chunk_dim as i32;

    for x in -CHUNK_GEN_RADIUS..=CHUNK_GEN_RADIUS {
        for z in -CHUNK_GEN_RADIUS..=CHUNK_GEN_RADIUS {
            let chunk_x = cam_chunk_x + x;
            let chunk_z = cam_chunk_z + z;
            if chunk_x < 0 || chunk_x >= chunk_dim || chunk_z < 0 || chunk_z >= chunk_dim {
                continue;
            }
            state
                .world
                .gen_terrain_column(chunk_x as u32, chunk_z as u32);
        }
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
