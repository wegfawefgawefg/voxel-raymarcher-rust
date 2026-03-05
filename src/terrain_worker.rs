use crossbeam::channel::{unbounded, Receiver, Sender};
use noise::{NoiseFn, Perlin};
use std::collections::HashSet;

use crate::world::CHUNK_SIZE;

const TERRAIN_NOISE_SCALE: f64 = 0.1;
const TERRAIN_HEIGHT_AMPLITUDE: f64 = 10.0;
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
            let y_offset = perlin.get([
                world_x as f64 * TERRAIN_NOISE_SCALE,
                world_z as f64 * TERRAIN_NOISE_SCALE,
            ]) * TERRAIN_HEIGHT_AMPLITUDE;
            let idx = local_x as usize + local_z as usize * CHUNK_SIZE;
            surface_y[idx] = floor_level + y_offset as i32;
        }
    }

    surface_y
}
