use crossbeam::channel::{unbounded, Receiver, Sender};
use noise::{NoiseFn, Perlin};
use std::collections::HashSet;

use crate::world::CHUNK_SIZE;

const TERRAIN_BASE_OFFSET: f64 = -18.0;
const TERRAIN_MACRO_SCALE: f64 = 0.035;
const TERRAIN_DETAIL_SCALE: f64 = 0.09;
const TERRAIN_MICRO_SCALE: f64 = 0.22;
const TERRAIN_RIDGE_SCALE: f64 = 0.055;

const TERRAIN_MACRO_AMP: f64 = 12.0;
const TERRAIN_DETAIL_AMP: f64 = 5.0;
const TERRAIN_MICRO_AMP: f64 = 2.0;
const TERRAIN_RIDGE_AMP: f64 = 3.0;

const MIN_SURFACE_OFFSET: i32 = -52;
const MAX_SURFACE_OFFSET: i32 = -2;
const TERRAIN_SEED: u32 = 0;

type TerrainRequest = (u32, u32);
const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

#[derive(Debug)]
pub struct TerrainColumnHeights {
    pub chunk_x: u32,
    pub chunk_z: u32,
    pub surface_y: [i32; CHUNK_AREA],
}

pub struct TerrainGenWorker {
    request_tx: Sender<TerrainRequest>,
    result_rx: Receiver<TerrainColumnHeights>,
    pending: HashSet<(u32, u32)>,
}

impl TerrainGenWorker {
    pub fn new(floor_level: i32) -> Self {
        let (request_tx, request_rx) = unbounded::<TerrainRequest>();
        let (result_tx, result_rx) = unbounded::<TerrainColumnHeights>();

        std::thread::spawn(move || {
            let perlin = Perlin::new(TERRAIN_SEED);
            while let Ok((chunk_x, chunk_z)) = request_rx.recv() {
                let surface_y = build_surface_heights(chunk_x, chunk_z, floor_level, &perlin);
                if result_tx
                    .send(TerrainColumnHeights {
                        chunk_x,
                        chunk_z,
                        surface_y,
                    })
                    .is_err()
                {
                    break;
                }
            }
        });

        Self {
            request_tx,
            result_rx,
            pending: HashSet::new(),
        }
    }

    pub fn is_pending(&self, chunk_x: u32, chunk_z: u32) -> bool {
        self.pending.contains(&(chunk_x, chunk_z))
    }

    pub fn enqueue(&mut self, chunk_x: u32, chunk_z: u32) -> bool {
        if !self.pending.insert((chunk_x, chunk_z)) {
            return false;
        }
        if self.request_tx.send((chunk_x, chunk_z)).is_err() {
            self.pending.remove(&(chunk_x, chunk_z));
            return false;
        }
        true
    }

    pub fn drain_completed(&mut self, max_results: usize, out: &mut Vec<TerrainColumnHeights>) {
        for _ in 0..max_results {
            let Ok(column) = self.result_rx.try_recv() else {
                break;
            };
            self.pending.remove(&(column.chunk_x, column.chunk_z));
            out.push(column);
        }
    }
}

fn build_surface_heights(
    chunk_x: u32,
    chunk_z: u32,
    floor_level: i32,
    perlin: &Perlin,
) -> [i32; CHUNK_AREA] {
    let mut surface_y = [floor_level; CHUNK_AREA];
    let base_x = chunk_x as i32 * CHUNK_SIZE as i32;
    let base_z = chunk_z as i32 * CHUNK_SIZE as i32;

    for local_x in 0..CHUNK_SIZE as i32 {
        for local_z in 0..CHUNK_SIZE as i32 {
            let world_x = base_x + local_x;
            let world_z = base_z + local_z;
            let idx = local_x as usize + local_z as usize * CHUNK_SIZE;
            surface_y[idx] = sample_surface_height(world_x, world_z, floor_level, perlin);
        }
    }

    surface_y
}

pub fn sample_surface_height(world_x: i32, world_z: i32, floor_level: i32, perlin: &Perlin) -> i32 {
    let x = world_x as f64;
    let z = world_z as f64;

    let macro_shape =
        perlin.get([x * TERRAIN_MACRO_SCALE, z * TERRAIN_MACRO_SCALE]) * TERRAIN_MACRO_AMP;
    let detail = perlin.get([
        x * TERRAIN_DETAIL_SCALE + 31.7,
        z * TERRAIN_DETAIL_SCALE - 19.3,
    ]) * TERRAIN_DETAIL_AMP;
    let micro = perlin.get([
        x * TERRAIN_MICRO_SCALE - 87.1,
        z * TERRAIN_MICRO_SCALE + 53.9,
    ]) * TERRAIN_MICRO_AMP;
    let ridge_raw = perlin.get([
        x * TERRAIN_RIDGE_SCALE + 11.0,
        z * TERRAIN_RIDGE_SCALE + 7.0,
    ]);
    let ridge = (1.0 - ridge_raw.abs()) * TERRAIN_RIDGE_AMP;

    let offset = (TERRAIN_BASE_OFFSET + macro_shape + detail + micro + ridge).round() as i32;
    floor_level + offset.clamp(MIN_SURFACE_OFFSET, MAX_SURFACE_OFFSET)
}
