use glam::Vec3;
use noise::{NoiseFn, Perlin};
use raylib::color::Color;

use crate::world::{Block, TerrainMaterialIds, World, CHUNK_SIZE};

const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

impl World {
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

    pub fn apply_terrain_column_heights(
        &mut self,
        chunk_x: u32,
        chunk_z: u32,
        surface_y: &[i32; CHUNK_AREA],
    ) {
        if chunk_x >= self.chunk_dim as u32 || chunk_z >= self.chunk_dim as u32 {
            return;
        }
        if self.is_terrain_column_generated(chunk_x, chunk_z) {
            return;
        }
        self.paint_terrain_column(chunk_x, chunk_z, surface_y);
    }

    pub fn gen_terrain_column(&mut self, chunk_x: u32, chunk_z: u32) {
        if chunk_x >= self.chunk_dim as u32 || chunk_z >= self.chunk_dim as u32 {
            return;
        }
        if self.is_terrain_column_generated(chunk_x, chunk_z) {
            return;
        }

        let base_x = chunk_x as i32 * CHUNK_SIZE as i32;
        let base_z = chunk_z as i32 * CHUNK_SIZE as i32;
        let floor = self.get_floor_level() as i32;
        let perlin = Perlin::new(0);
        let scale = 0.1;
        let mut surface_y = [floor; CHUNK_AREA];

        for local_x in 0..CHUNK_SIZE as i32 {
            for local_z in 0..CHUNK_SIZE as i32 {
                let world_x = base_x + local_x;
                let world_z = base_z + local_z;
                let y_offset = perlin.get([world_x as f64 * scale, world_z as f64 * scale]) * 10.0;
                let idx = local_x as usize + local_z as usize * CHUNK_SIZE;
                surface_y[idx] = floor + y_offset as i32;
            }
        }
        self.paint_terrain_column(chunk_x, chunk_z, &surface_y);
    }

    fn terrain_material_ids(&mut self) -> TerrainMaterialIds {
        if let Some(ids) = self.terrain_materials {
            return ids;
        }
        let ids = TerrainMaterialIds {
            grass: self.intern_material(Color::new(56, 183, 100, 255)),
            dirt: self.intern_material(Color::new(122, 72, 65, 255)),
            water: self.intern_material(Color::new(50, 120, 190, 130)),
        };
        self.terrain_materials = Some(ids);
        ids
    }

    fn paint_terrain_column(&mut self, chunk_x: u32, chunk_z: u32, surface_y: &[i32; CHUNK_AREA]) {
        let materials = self.terrain_material_ids();
        let water_level = self.get_floor_level().saturating_sub(6) as i32;
        let lower_void = self.get_lower_void() as i32;
        let base_x = chunk_x as i32 * CHUNK_SIZE as i32;
        let base_z = chunk_z as i32 * CHUNK_SIZE as i32;

        for local_x in 0..CHUNK_SIZE as i32 {
            for local_z in 0..CHUNK_SIZE as i32 {
                let idx = local_x as usize + local_z as usize * CHUNK_SIZE;
                let world_x = base_x + local_x;
                let world_z = base_z + local_z;
                let surface = surface_y[idx].clamp(0, lower_void);

                self.set_voxel_material_i32(world_x, surface, world_z, materials.grass);
                if surface > water_level {
                    for y in water_level..surface {
                        self.set_voxel_material_i32(world_x, y, world_z, materials.water);
                    }
                }
                for y in (surface + 1)..lower_void {
                    self.set_voxel_material_i32(world_x, y, world_z, materials.dirt);
                }
            }
        }
        self.mark_terrain_column_generated(chunk_x, chunk_z);
    }
}
