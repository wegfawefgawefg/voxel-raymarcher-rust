use std::collections::HashMap;

use glam::{UVec3, Vec3};
use raylib::color::Color;

pub type Block = Color;
pub type MaterialId = u16;
pub const CHUNK_SIZE: usize = 16;

const CHUNK_SHIFT: usize = 4;
const CHUNK_MASK: i32 = CHUNK_SIZE as i32 - 1;
const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
const AIR_COLOR: Color = Color::new(0, 0, 0, 0);
const AIR_MATERIAL: MaterialId = 0;

#[derive(Debug, Clone)]
pub struct Object {
    pub pos: Vec3,
    pub size: Vec3,
    pub color: Block,
}

#[derive(Copy, Clone, Debug)]
pub struct Material {
    pub color: Color,
    pub is_transparent: bool,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ChunkMeta {
    pub generated: bool,
    pub non_air_voxels: u16,
    pub has_transparency: bool,
}

impl ChunkMeta {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.non_air_voxels == 0
    }
}

#[derive(Debug)]
struct ChunkData {
    voxels: Option<Vec<MaterialId>>,
    meta: ChunkMeta,
}

impl ChunkData {
    fn new() -> Self {
        Self {
            voxels: None,
            meta: ChunkMeta::default(),
        }
    }
}

#[derive(Debug)]
pub struct World {
    pub dim: usize,
    pub chunk_dim: usize,
    chunks: Vec<ChunkData>,
    terrain_columns_generated: Vec<bool>,
    pub genned_objects: Vec<Object>,
    materials: Vec<Material>,
    material_lookup: HashMap<u32, MaterialId>,
}

impl World {
    pub fn new(dim: usize) -> Self {
        debug_assert_eq!(1usize << CHUNK_SHIFT, CHUNK_SIZE);
        let chunk_dim = dim / CHUNK_SIZE;
        let chunk_count = chunk_dim * chunk_dim * chunk_dim;

        let mut material_lookup = HashMap::new();
        material_lookup.insert(Self::color_key(AIR_COLOR), AIR_MATERIAL);

        Self {
            dim,
            chunk_dim,
            chunks: (0..chunk_count).map(|_| ChunkData::new()).collect(),
            terrain_columns_generated: vec![false; chunk_dim * chunk_dim],
            genned_objects: Vec::new(),
            materials: vec![Material {
                color: AIR_COLOR,
                is_transparent: false,
            }],
            material_lookup,
        }
    }

    #[inline]
    pub fn get_material(&self, material_id: MaterialId) -> Material {
        self.materials
            .get(material_id as usize)
            .copied()
            .unwrap_or(self.materials[AIR_MATERIAL as usize])
    }

    pub fn intern_material(&mut self, color: Color) -> MaterialId {
        let key = Self::color_key(color);
        if let Some(id) = self.material_lookup.get(&key) {
            return *id;
        }

        let new_id = self.materials.len() as MaterialId;
        self.materials.push(Material {
            color,
            is_transparent: color.a > 0 && color.a < 255,
        });
        self.material_lookup.insert(key, new_id);
        new_id
    }

    pub fn is_chunk_genned(&self, chunk_pos: UVec3) -> bool {
        self.chunk_meta(chunk_pos.x as i32, chunk_pos.y as i32, chunk_pos.z as i32)
            .map(|meta| meta.generated)
            .unwrap_or(false)
    }

    #[inline]
    pub fn chunk_meta(&self, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Option<ChunkMeta> {
        if chunk_x < 0
            || chunk_y < 0
            || chunk_z < 0
            || chunk_x >= self.chunk_dim as i32
            || chunk_y >= self.chunk_dim as i32
            || chunk_z >= self.chunk_dim as i32
        {
            return None;
        }

        let chunk_index = self.chunk_index(chunk_x as usize, chunk_y as usize, chunk_z as usize);
        Some(self.chunks[chunk_index].meta)
    }

    #[inline]
    pub fn chunk_meta_from_voxel(&self, x: i32, y: i32, z: i32) -> Option<ChunkMeta> {
        if x < 0
            || y < 0
            || z < 0
            || x >= self.dim as i32
            || y >= self.dim as i32
            || z >= self.dim as i32
        {
            return None;
        }

        self.chunk_meta(
            (x as usize >> CHUNK_SHIFT) as i32,
            (y as usize >> CHUNK_SHIFT) as i32,
            (z as usize >> CHUNK_SHIFT) as i32,
        )
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
        self.get_material(self.get_voxel_material_i32(pos.x as i32, pos.y as i32, pos.z as i32))
            .color
    }

    #[inline]
    pub fn get_voxel_material_i32(&self, x: i32, y: i32, z: i32) -> MaterialId {
        if x < 0
            || y < 0
            || z < 0
            || x >= self.dim as i32
            || y >= self.dim as i32
            || z >= self.dim as i32
        {
            return AIR_MATERIAL;
        }

        let chunk_x = (x as usize) >> CHUNK_SHIFT;
        let chunk_y = (y as usize) >> CHUNK_SHIFT;
        let chunk_z = (z as usize) >> CHUNK_SHIFT;
        let chunk_index = self.chunk_index(chunk_x, chunk_y, chunk_z);
        let chunk = &self.chunks[chunk_index];
        let Some(voxels) = &chunk.voxels else {
            return AIR_MATERIAL;
        };

        let in_chunk_x = (x & CHUNK_MASK) as usize;
        let in_chunk_y = (y & CHUNK_MASK) as usize;
        let in_chunk_z = (z & CHUNK_MASK) as usize;
        voxels[self.voxel_index(in_chunk_x, in_chunk_y, in_chunk_z)]
    }

    #[inline]
    pub fn get_voxel_material_unchecked_i32(&self, x: i32, y: i32, z: i32) -> MaterialId {
        debug_assert!(x >= 0 && y >= 0 && z >= 0);
        debug_assert!(x < self.dim as i32 && y < self.dim as i32 && z < self.dim as i32);

        let chunk_x = (x as usize) >> CHUNK_SHIFT;
        let chunk_y = (y as usize) >> CHUNK_SHIFT;
        let chunk_z = (z as usize) >> CHUNK_SHIFT;
        let chunk_index = self.chunk_index(chunk_x, chunk_y, chunk_z);
        let Some(voxels) = self.chunks[chunk_index].voxels.as_ref() else {
            return AIR_MATERIAL;
        };

        let in_chunk_x = (x & CHUNK_MASK) as usize;
        let in_chunk_y = (y & CHUNK_MASK) as usize;
        let in_chunk_z = (z & CHUNK_MASK) as usize;
        voxels[self.voxel_index(in_chunk_x, in_chunk_y, in_chunk_z)]
    }

    pub fn set_voxel(&mut self, pos: Vec3, color: Color) {
        let material_id = self.intern_material(color);
        self.set_voxel_material_i32(pos.x as i32, pos.y as i32, pos.z as i32, material_id);
    }

    #[inline]
    pub fn set_voxel_material_i32(&mut self, x: i32, y: i32, z: i32, material_id: MaterialId) {
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
        let voxel_index = self.voxel_index(
            (x & CHUNK_MASK) as usize,
            (y & CHUNK_MASK) as usize,
            (z & CHUNK_MASK) as usize,
        );

        let material_transparent = self.get_material(material_id).is_transparent;
        let chunk = &mut self.chunks[chunk_index];

        if material_id == AIR_MATERIAL && chunk.voxels.is_none() {
            return;
        }
        if chunk.voxels.is_none() {
            chunk.voxels = Some(vec![AIR_MATERIAL; CHUNK_VOLUME]);
        }

        let voxels = chunk.voxels.as_mut().expect("chunk voxels allocated");
        let old_id = voxels[voxel_index];
        if old_id == material_id {
            return;
        }

        let old_non_air = old_id != AIR_MATERIAL;
        let new_non_air = material_id != AIR_MATERIAL;
        if old_non_air && !new_non_air {
            chunk.meta.non_air_voxels = chunk.meta.non_air_voxels.saturating_sub(1);
        } else if !old_non_air && new_non_air {
            chunk.meta.non_air_voxels = chunk.meta.non_air_voxels.saturating_add(1);
        }

        voxels[voxel_index] = material_id;
        if new_non_air && material_transparent {
            chunk.meta.has_transparency = true;
        }

        if chunk.meta.non_air_voxels == 0 {
            chunk.voxels = None;
            chunk.meta.has_transparency = false;
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
        self.terrain_columns_generated.fill(false);
        for chunk in self.chunks.iter_mut() {
            chunk.voxels = None;
            chunk.meta = ChunkMeta::default();
        }
    }

    #[inline]
    pub(crate) fn is_terrain_column_generated(&self, chunk_x: u32, chunk_z: u32) -> bool {
        let column_index = self.terrain_column_index(chunk_x as usize, chunk_z as usize);
        self.terrain_columns_generated[column_index]
    }

    pub(crate) fn mark_terrain_column_generated(&mut self, chunk_x: u32, chunk_z: u32) {
        let column_index = self.terrain_column_index(chunk_x as usize, chunk_z as usize);
        self.terrain_columns_generated[column_index] = true;
        for chunk_y in 0..self.chunk_dim {
            let chunk_index = self.chunk_index(chunk_x as usize, chunk_y, chunk_z as usize);
            self.chunks[chunk_index].meta.generated = true;
        }
    }

    pub(crate) fn mark_chunk_generated(&mut self, chunk_pos: UVec3) {
        if !self.is_in_chunk_bounds(chunk_pos) {
            return;
        }
        let chunk_index = self.chunk_index(
            chunk_pos.x as usize,
            chunk_pos.y as usize,
            chunk_pos.z as usize,
        );
        self.chunks[chunk_index].meta.generated = true;
    }

    #[inline]
    fn chunk_index(&self, chunk_x: usize, chunk_y: usize, chunk_z: usize) -> usize {
        chunk_x + chunk_y * self.chunk_dim + chunk_z * self.chunk_dim * self.chunk_dim
    }

    #[inline]
    fn voxel_index(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }

    #[inline]
    fn terrain_column_index(&self, chunk_x: usize, chunk_z: usize) -> usize {
        chunk_x + chunk_z * self.chunk_dim
    }

    #[inline]
    fn color_key(c: Color) -> u32 {
        ((c.r as u32) << 24) | ((c.g as u32) << 16) | ((c.b as u32) << 8) | c.a as u32
    }
}
