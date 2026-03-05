use glam::{UVec3, Vec3};
use noise::{NoiseFn, Perlin};
use raylib::color::Color;

use crate::world::{Block, MaterialId, Object, World, CHUNK_SIZE};

impl World {
    pub fn gen_empty_voxel_array(dim: usize) -> Vec<MaterialId> {
        vec![0; dim * dim * dim]
    }

    pub fn gen_cube(&mut self, pos: Vec3, size: Vec3, color: Color) {
        let material = self.intern_material(color);
        for x in 0..size.x as usize {
            for y in 0..size.y as usize {
                for z in 0..size.z as usize {
                    let voxel_pos = pos + Vec3::new(x as f32, y as f32, z as f32);
                    self.set_voxel_material_i32(
                        voxel_pos.x as i32,
                        voxel_pos.y as i32,
                        voxel_pos.z as i32,
                        material,
                    );
                }
            }
        }
        self.genned_objects.push(Object { pos, size, color });
    }

    pub fn gen_sphere(&mut self, pos: Vec3, radius: f32, color: Color) {
        let material = self.intern_material(color);
        let radius = radius as i32;
        for x in -radius..=radius {
            for y in -radius..=radius {
                for z in -radius..=radius {
                    if Vec3::new(x as f32, y as f32, z as f32).length() <= radius as f32 {
                        let voxel_pos = pos + Vec3::new(x as f32, y as f32, z as f32);
                        self.set_voxel_material_i32(
                            voxel_pos.x as i32,
                            voxel_pos.y as i32,
                            voxel_pos.z as i32,
                            material,
                        );
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
            color,
        });
    }

    pub fn get_lower_void(&self) -> usize {
        self.dim - 1
    }

    pub fn get_floor_level(&self) -> usize {
        self.dim - 1
    }

    pub fn get_above_floor_level(&self) -> usize {
        self.dim - 2
    }

    pub fn gen_floor(&mut self, color: Block) {
        let material = self.intern_material(color);
        let floor_level = self.get_floor_level() as i32;
        for x in 0..self.dim as i32 {
            for z in 0..self.dim as i32 {
                self.set_voxel_material_i32(x, floor_level, z, material);
            }
        }
    }

    pub fn gen_sin_terrain(&mut self, chunk_pos: UVec3) {
        if self.is_chunk_genned(chunk_pos) {
            return;
        }

        let grass = self.intern_material(Color::new(56, 183, 100, 255));
        let dirt = self.intern_material(Color::new(122, 72, 65, 255));
        let base_x = chunk_pos.x as i32 * CHUNK_SIZE as i32;
        let base_z = chunk_pos.z as i32 * CHUNK_SIZE as i32;
        let floor = self.get_floor_level() as i32;
        let lower_void = self.get_lower_void() as i32;
        let frequency = std::f32::consts::PI * 2.0 / CHUNK_SIZE as f32;

        for x in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                let world_x = base_x + x;
                let world_z = base_z + z;
                let y_offset = (world_x as f32 * frequency).sin() * 2.0
                    + (world_z as f32 * frequency).cos() * 2.0;
                let surface_y = floor + y_offset as i32;

                self.set_voxel_material_i32(world_x, surface_y, world_z, grass);
                for y in (surface_y + 1)..lower_void {
                    self.set_voxel_material_i32(world_x, y, world_z, dirt);
                }
            }
        }

        self.mark_chunk_generated(chunk_pos);
    }

    pub fn gen_terrain_column(&mut self, chunk_x: u32, chunk_z: u32) {
        if chunk_x >= self.chunk_dim as u32 || chunk_z >= self.chunk_dim as u32 {
            return;
        }
        if self.is_terrain_column_generated(chunk_x, chunk_z) {
            return;
        }

        let grass = self.intern_material(Color::new(56, 183, 100, 255));
        let dirt = self.intern_material(Color::new(122, 72, 65, 255));
        let water = self.intern_material(Color::new(50, 120, 190, 130));
        let water_level = self.get_floor_level().saturating_sub(6) as i32;

        let base_x = chunk_x as i32 * CHUNK_SIZE as i32;
        let base_z = chunk_z as i32 * CHUNK_SIZE as i32;
        let floor = self.get_floor_level() as i32;
        let lower_void = self.get_lower_void() as i32;
        let perlin = Perlin::new(0);
        let scale = 0.1;

        for x in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                let world_x = base_x + x;
                let world_z = base_z + z;
                let y_offset = perlin.get([world_x as f64 * scale, world_z as f64 * scale]) * 10.0;
                let surface_y = floor + y_offset as i32;

                self.set_voxel_material_i32(world_x, surface_y, world_z, grass);
                if surface_y > water_level {
                    for y in water_level..surface_y {
                        self.set_voxel_material_i32(world_x, y, world_z, water);
                    }
                }
                for y in (surface_y + 1)..lower_void {
                    self.set_voxel_material_i32(world_x, y, world_z, dirt);
                }
            }
        }

        self.mark_terrain_column_generated(chunk_x, chunk_z);
    }

    pub fn gen_terrain(&mut self, chunk_pos: UVec3) {
        self.gen_terrain_column(chunk_pos.x, chunk_pos.z);
    }
}
