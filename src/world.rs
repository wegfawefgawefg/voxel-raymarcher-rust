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

type Chunk = Vec<Block>;
pub const AIR: Block = Block::new(0, 0, 0, 0);
pub const CHUNK_SIZE: usize = 16;
const CHUNK_SHIFT: usize = 4;
const CHUNK_MASK: i32 = CHUNK_SIZE as i32 - 1;
const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Debug)]
pub struct World {
    pub dim: usize,
    pub chunk_dim: usize,
    pub chunks: Vec<Chunk>,
    pub generated_chunks: Vec<bool>,
    pub genned_objects: Vec<Object>,
}

pub fn gen_air_chunk() -> Chunk {
    vec![AIR; CHUNK_VOLUME]
}

impl World {
    pub fn new(dim: usize) -> Self {
        debug_assert_eq!(1usize << CHUNK_SHIFT, CHUNK_SIZE);
        let chunk_dim = dim / CHUNK_SIZE;
        let chunk_count = chunk_dim * chunk_dim * chunk_dim;
        Self {
            dim,
            chunk_dim,
            chunks: (0..chunk_count).map(|_| gen_air_chunk()).collect(),
            generated_chunks: vec![false; chunk_count],
            genned_objects: Vec::new(),
        }
    }

    pub fn is_chunk_genned(&self, chunk_pos: UVec3) -> bool {
        self.generated_chunks[self.chunk_index(chunk_pos.x as usize, chunk_pos.y as usize, chunk_pos.z as usize)]
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

    pub fn get_voxel(&self, pos: Vec3) -> Block {
        self.get_voxel_i32(pos.x as i32, pos.y as i32, pos.z as i32)
    }

    #[inline]
    pub fn get_voxel_i32(&self, x: i32, y: i32, z: i32) -> Block {
        if x < 0
            || y < 0
            || z < 0
            || x >= self.dim as i32
            || y >= self.dim as i32
            || z >= self.dim as i32
        {
            return AIR;
        }

        let chunk_x = (x as usize) >> CHUNK_SHIFT;
        let chunk_y = (y as usize) >> CHUNK_SHIFT;
        let chunk_z = (z as usize) >> CHUNK_SHIFT;
        let chunk_index = self.chunk_index(chunk_x, chunk_y, chunk_z);

        let in_chunk_x = (x & CHUNK_MASK) as usize;
        let in_chunk_y = (y & CHUNK_MASK) as usize;
        let in_chunk_z = (z & CHUNK_MASK) as usize;
        let voxel_index = self.voxel_index(in_chunk_x, in_chunk_y, in_chunk_z);

        self.chunks[chunk_index][voxel_index]
    }

    pub fn set_voxel(&mut self, pos: Vec3, block: Block) {
        let x = pos.x as i32;
        let y = pos.y as i32;
        let z = pos.z as i32;
        if x < 0
            || y < 0
            || z < 0
            || x >= self.dim as i32
            || y >= self.dim as i32
            || z >= self.dim as i32
        {
            return;
        }

        let chunk_x = (x as usize) >> CHUNK_SHIFT;
        let chunk_y = (y as usize) >> CHUNK_SHIFT;
        let chunk_z = (z as usize) >> CHUNK_SHIFT;
        let chunk_index = self.chunk_index(chunk_x, chunk_y, chunk_z);

        let in_chunk_x = (x & CHUNK_MASK) as usize;
        let in_chunk_y = (y & CHUNK_MASK) as usize;
        let in_chunk_z = (z & CHUNK_MASK) as usize;
        let voxel_index = self.voxel_index(in_chunk_x, in_chunk_y, in_chunk_z);

        self.chunks[chunk_index][voxel_index] = block;
    }

    pub fn is_in_bounds(&self, pos: Vec3) -> bool {
        pos.x >= 0.0
            && pos.x < self.dim as f32
            && pos.y >= 0.0
            && pos.y < self.dim as f32
            && pos.z >= 0.0
            && pos.z < self.dim as f32
    }

    pub fn is_in_chunk_bounds(&self, chunk_pos: UVec3) -> bool {
        chunk_pos.x < self.chunk_dim as u32
            && chunk_pos.y < self.chunk_dim as u32
            && chunk_pos.z < self.chunk_dim as u32
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
        for chunk in self.chunks.iter_mut() {
            chunk.fill(AIR);
        }
        self.generated_chunks.fill(false);
    }

    pub fn gen_empty_voxel_array(dim: usize) -> Vec<Block> {
        vec![AIR; dim * dim * dim]
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
        // bail if already generated
        let chunk_index = self.chunk_index(chunk_pos.x as usize, chunk_pos.y as usize, chunk_pos.z as usize);
        if self.generated_chunks[chunk_index] {
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

        self.generated_chunks[chunk_index] = true;
    }

    pub fn gen_terrain(&mut self, chunk_pos: UVec3) {
        // bail if already generated
        let chunk_index = self.chunk_index(chunk_pos.x as usize, chunk_pos.y as usize, chunk_pos.z as usize);
        if self.generated_chunks[chunk_index] {
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
                let gentle_green = Color::new(56, 183, 100, 100);
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

        self.generated_chunks[chunk_index] = true;
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

    #[inline]
    fn chunk_index(&self, chunk_x: usize, chunk_y: usize, chunk_z: usize) -> usize {
        chunk_x + chunk_y * self.chunk_dim + chunk_z * self.chunk_dim * self.chunk_dim
    }

    #[inline]
    fn voxel_index(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }
}
