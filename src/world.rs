use std::collections::HashMap;

use glam::Vec3;
use raylib::color::Color;

pub type Block = Color;
pub type MaterialId = u16;
pub const CHUNK_SIZE: usize = 16;

const CHUNK_SHIFT: usize = 4;
const CHUNK_MASK: i32 = CHUNK_SIZE as i32 - 1;
const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
const AIR_COLOR: Color = Color::new(0, 0, 0, 0);
const AIR_MATERIAL: MaterialId = 0;

#[derive(Copy, Clone, Debug)]
pub struct Material {
    pub color: Color,
    pub is_transparent: bool,
    pub alpha: f32,
    pub premul_r: f32,
    pub premul_g: f32,
    pub premul_b: f32,
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

#[derive(Copy, Clone, Debug)]
pub struct TerrainMaterialIds {
    pub grass: MaterialId,
    pub dirt: MaterialId,
    pub water: MaterialId,
    pub stone: MaterialId,
    pub clay: MaterialId,
}

#[derive(Copy, Clone, Debug)]
pub struct FeatureMaterialIds {
    pub basalt: MaterialId,
    pub sandstone: MaterialId,
    pub glass: MaterialId,
    pub glow: MaterialId,
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
    revision: u64,
    materials: Vec<Material>,
    material_lookup: HashMap<u32, MaterialId>,
    pub(crate) terrain_materials: Option<TerrainMaterialIds>,
    pub(crate) feature_materials: Option<FeatureMaterialIds>,
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
            revision: 0,
            materials: vec![Material {
                color: AIR_COLOR,
                is_transparent: false,
                alpha: 0.0,
                premul_r: 0.0,
                premul_g: 0.0,
                premul_b: 0.0,
            }],
            material_lookup,
            terrain_materials: None,
            feature_materials: None,
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
        let alpha = color.a as f32 / 255.0;
        self.materials.push(Material {
            color,
            is_transparent: color.a > 0 && color.a < 255,
            alpha,
            premul_r: color.r as f32 * alpha,
            premul_g: color.g as f32 * alpha,
            premul_b: color.b as f32 * alpha,
        });
        self.material_lookup.insert(key, new_id);
        new_id
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
        self.revision = self.revision.saturating_add(1);
        if new_non_air && material_transparent {
            chunk.meta.has_transparency = true;
        }

        if chunk.meta.non_air_voxels == 0 {
            chunk.voxels = None;
            chunk.meta.has_transparency = false;
        }
    }

    pub fn get_center(&self) -> Vec3 {
        Vec3::new(
            self.dim as f32 / 2.0,
            self.dim as f32 / 2.0,
            self.dim as f32 / 2.0,
        )
    }

    #[inline]
    pub fn revision(&self) -> u64 {
        self.revision
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
