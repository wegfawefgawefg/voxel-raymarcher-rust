use std::collections::HashMap;

use glam::UVec3;
use glam::Vec3;
use noise::{NoiseFn, Perlin};
use raylib::color::Color;

pub type Block = Color;

#[derive(Debug, Clone)]
pub struct Object {
    pub pos: Vec3,
    pub size: Vec3,
    pub color: Block,
}

// define chunk type
type Chunk = Vec<Vec<Vec<Option<Block>>>>;
pub const CHUNK_SIZE: usize = 16;

#[derive(Debug)]
pub struct World {
    pub dim: usize,
    pub chunk_dim: usize,
    pub chunks: HashMap<UVec3, Option<Chunk>>,
    pub genned_objects: Vec<Object>,
}

pub enum GetVoxelResult {
    Voxel { block: Block },
    NoVoxel,
    ChunkNotGenerated,
}

impl World {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            chunk_dim: dim / CHUNK_SIZE,
            chunks: HashMap::new(),
            genned_objects: Vec::new(),
        }
    }

    pub fn to_chunk_pos(&self, pos: Vec3) -> UVec3 {
        UVec3::new(
            (pos.x / CHUNK_SIZE as f32).floor() as u32,
            (pos.y / CHUNK_SIZE as f32).floor() as u32,
            (pos.z / CHUNK_SIZE as f32).floor() as u32,
        )
    }

    pub fn get_chunk_world_pos(&self, chunk_pos: UVec3) -> Vec3 {
        Vec3::new(
            chunk_pos.x as f32 * CHUNK_SIZE as f32,
            chunk_pos.y as f32 * CHUNK_SIZE as f32,
            chunk_pos.z as f32 * CHUNK_SIZE as f32,
        )
    }

    pub fn get_voxel(&self, pos: Vec3) -> GetVoxelResult {
        if !self.is_in_bounds(pos) {
            return GetVoxelResult::NoVoxel;
        }
        let chunk_pos = self.to_chunk_pos(pos);

        // Check if there's an entry in the dictionary for the chunk
        if let Some(chunk) = self.chunks.get(&chunk_pos) {
            let pos_in_chunk = pos.as_uvec3() - self.get_chunk_world_pos(chunk_pos).as_uvec3();
            if let Some(chunk) = chunk {
                if let Some(block) =
                    chunk[pos_in_chunk.x as usize][pos_in_chunk.y as usize][pos_in_chunk.z as usize]
                {
                    return GetVoxelResult::Voxel { block };
                } else {
                    return GetVoxelResult::NoVoxel;
                }
            }
        } else {
            return GetVoxelResult::ChunkNotGenerated;
        }
        GetVoxelResult::NoVoxel
    }

    pub fn set_voxel(&mut self, pos: Vec3, color: Color) {
        if !self.is_in_bounds(pos) {
            return;
        }
        let chunk_pos = self.to_chunk_pos(pos);
        let pos_in_chunk = pos.as_uvec3()
            - chunk_pos * UVec3::new(CHUNK_SIZE as u32, CHUNK_SIZE as u32, CHUNK_SIZE as u32);

        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            if let Some(chunk) = chunk {
                // If the chunk exists, set the voxel
                chunk[pos_in_chunk.x as usize][pos_in_chunk.y as usize][pos_in_chunk.z as usize] =
                    Some(color);
            }
        } else {
            // If the chunk does not exist, create it and set the voxel
            let mut new_chunk = Self::gen_empty_voxel_array(CHUNK_SIZE);
            new_chunk[pos_in_chunk.x as usize][pos_in_chunk.y as usize][pos_in_chunk.z as usize] =
                Some(color);
            self.chunks.insert(chunk_pos, Some(new_chunk));
        }
    }

    pub fn is_in_bounds(&self, pos: Vec3) -> bool {
        pos.x >= 0.0
            && pos.x < self.dim as f32
            && pos.y >= 0.0
            && pos.y < self.dim as f32
            && pos.z >= 0.0
            && pos.z < self.dim as f32
    }

    pub fn get_center(&self) -> Vec3 {
        Vec3::new(
            self.dim as f32 / 2.0,
            self.dim as f32 / 2.0,
            self.dim as f32 / 2.0,
        )
    }

    pub fn reset(&mut self) {
        self.genned_objects.clear();
        self.chunks.clear();
    }

    pub fn gen_empty_voxel_array(dim: usize) -> Vec<Vec<Vec<Option<Color>>>> {
        vec![vec![vec![None; dim]; dim]; dim]
    }

    pub fn gen_cube(&mut self, pos: Vec3, size: Vec3, block: Color) {
        for x in 0..size.x as usize {
            for y in 0..size.y as usize {
                for z in 0..size.z as usize {
                    let voxel_pos = pos + Vec3::new(x as f32, y as f32, z as f32);
                    self.set_voxel(voxel_pos, block);
                }
            }
        }
        self.genned_objects.push(Object {
            pos,
            size,
            color: block,
        });
    }

    pub fn gen_sphere(&mut self, pos: Vec3, radius: f32, block: Color) {
        let radius = radius as i32;
        for x in -radius..=radius {
            for y in -radius..=radius {
                for z in -radius..=radius {
                    if Vec3::new(x as f32, y as f32, z as f32).length() <= radius as f32 {
                        let voxel_pos = pos + Vec3::new(x as f32, y as f32, z as f32);
                        if self.is_in_bounds(voxel_pos) {
                            self.set_voxel(voxel_pos, block);
                        }
                    }
                }
            }
        }
        self.genned_objects.push(Object {
            pos,
            size: Vec3::new(
                radius as f32 * 2.0,
                radius as f32 * 2.0,
                radius as f32 * 2.0,
            ),
            color: block,
        });
    }

    pub fn get_lower_void(&self) -> usize {
        self.dim - 1
    }

    pub fn gen_sin_terrain(&mut self, chunk_pos: UVec3) {
        // skip if the chunk is already generated
        if self.chunks.contains_key(&chunk_pos) {
            return;
        }

        // Convert chunk position to block position
        let base_pos = Vec3::new(
            chunk_pos.x as f32 * CHUNK_SIZE as f32,
            self.get_floor_level() as f32, // Start from the floor level
            chunk_pos.z as f32 * CHUNK_SIZE as f32,
        );
        // let mut rng = rand::thread_rng();
        // Generate terrain within the chunk
        let frequency = std::f32::consts::PI * 2.0 / CHUNK_SIZE as f32; // one period per chunk
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // World position of the current voxel
                let world_x = base_pos.x + x as f32;
                let world_z = base_pos.z + z as f32;

                let y_offset =
                    (world_x * frequency).sin() * 2.0 + (world_z * frequency).cos() * 2.0;
                let mut world_y = base_pos.y + y_offset;

                // set the block green, and all blocks down to the floor brown
                let gentle_green = Block::new(56, 183, 100, 255);
                // const RANDOM_DELTA_MAG: f32 = 10.0;
                // let random_delta_r = rng.gen_range(-1.0..1.0) * RANDOM_DELTA_MAG;
                // let random_delta_g = rng.gen_range(-1.0..1.0) * RANDOM_DELTA_MAG;
                // let random_delta_b = rng.gen_range(-1.0..1.0) * RANDOM_DELTA_MAG;
                // let grass_color =
                //     gentle_green + Vec3::new(random_delta_r, random_delta_g, random_delta_b);
                self.set_voxel(Vec3::new(world_x, world_y, world_z), gentle_green);

                // fill to the floor with brown
                let brown = Block::new(122, 72, 65, 255);
                // let random_delta_r = rng.gen_range(-1.0..1.0) * RANDOM_DELTA_MAG;
                // let random_delta_g = rng.gen_range(-1.0..1.0) * RANDOM_DELTA_MAG;
                // let random_delta_b = rng.gen_range(-1.0..1.0) * RANDOM_DELTA_MAG;
                // let dirt_color = brown + Vec3::new(random_delta_r, random_delta_g, random_delta_b);
                world_y += 1.0;
                while world_y < self.get_lower_void() as f32 {
                    self.set_voxel(Vec3::new(world_x, world_y, world_z), brown);
                    world_y += 1.0;
                }
            }
        }
    }

    pub fn gen_terrain(&mut self, chunk_pos: UVec3) {
        // Skip if the chunk is already generated
        if self.chunks.contains_key(&chunk_pos) {
            return;
        }

        // Convert chunk position to block position
        let base_pos = Vec3::new(
            chunk_pos.x as f32 * CHUNK_SIZE as f32,
            self.get_floor_level() as f32, // Start from the floor level
            chunk_pos.z as f32 * CHUNK_SIZE as f32,
        );

        // Initialize Perlin noise generator
        let perlin = Perlin::new(0);

        // Random number generator

        // Generate terrain within the chunk
        let scale = 0.1; // Scale factor for Perlin noise
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // World position of the current voxel
                let world_x = base_pos.x + x as f32;
                let world_z = base_pos.z + z as f32;

                // Calculate height using Perlin noise
                let y_offset = perlin.get([world_x as f64 * scale, world_z as f64 * scale]) * 10.0;
                let world_y = base_pos.y + y_offset as f32;

                // Set the block green, and all blocks down to the floor brown
                let gentle_green = Color::new(56, 183, 100, 255);
                self.set_voxel(Vec3::new(world_x, world_y, world_z), gentle_green);

                // Fill to the floor with brown
                let dirt_color = Color::new(122, 72, 65, 255);
                let mut y = world_y + 1.0;
                while y < self.get_lower_void() as f32 {
                    self.set_voxel(Vec3::new(world_x, y, world_z), dirt_color);
                    y += 1.0;
                }
            }
        }
    }

    pub fn get_floor_level(&self) -> usize {
        self.dim - 1
    }

    pub fn get_above_floor_level(&self) -> usize {
        self.dim - 2
    }

    pub fn gen_floor(&mut self, block: Block) {
        let floor_level = self.get_floor_level();
        for x in 0..self.dim {
            for z in 0..self.dim {
                self.set_voxel(Vec3::new(x as f32, floor_level as f32, z as f32), block);
            }
        }
    }
}
