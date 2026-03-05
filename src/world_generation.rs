use glam::{Vec2, Vec3};
use noise::Perlin;
use raylib::color::Color;

use crate::terrain_worker::sample_surface_height;
use crate::world::{Block, FeatureMaterialIds, MaterialId, TerrainMaterialIds, World, CHUNK_SIZE};

const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;
const VERTICAL_TORUS_DEFS: [(f32, f32, f32, f32, f32); 6] = [
    // offset_x, offset_y, offset_z, major_radius, minor_radius
    (-44.0, 7.0, 16.0, 22.0, 4.5),
    (-24.0, 8.0, 32.0, 16.0, 3.4),
    (-4.0, 7.0, 48.0, 20.0, 4.2),
    (16.0, 8.0, 64.0, 14.0, 3.0),
    (36.0, 7.0, 80.0, 24.0, 5.0),
    (56.0, 8.0, 96.0, 18.0, 3.6),
];

#[derive(Copy, Clone)]
struct FeatureBounds {
    min_x: i32,
    max_x: i32,
    min_z: i32,
    max_z: i32,
}

impl FeatureBounds {
    fn around(center: Vec3, radius: f32) -> Self {
        Self {
            min_x: (center.x - radius).floor() as i32,
            max_x: (center.x + radius).ceil() as i32,
            min_z: (center.z - radius).floor() as i32,
            max_z: (center.z + radius).ceil() as i32,
        }
    }
}

#[inline]
fn column_intersects_bounds(
    col_min_x: i32,
    col_max_x: i32,
    col_min_z: i32,
    col_max_z: i32,
    bounds: FeatureBounds,
) -> bool {
    col_min_x <= bounds.max_x
        && col_max_x >= bounds.min_x
        && col_min_z <= bounds.max_z
        && col_max_z >= bounds.min_z
}

#[inline]
fn layer_hash(x: i32, y: i32, z: i32) -> u32 {
    let mut h = x as u32;
    h = h.wrapping_mul(0x9E3779B1).wrapping_add(y as u32);
    h = h.rotate_left(13) ^ (z as u32).wrapping_mul(0x85EBCA77);
    h ^ (h >> 16)
}

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
        let mut surface_y = [floor; CHUNK_AREA];

        for local_x in 0..CHUNK_SIZE as i32 {
            for local_z in 0..CHUNK_SIZE as i32 {
                let world_x = base_x + local_x;
                let world_z = base_z + local_z;
                let idx = local_x as usize + local_z as usize * CHUNK_SIZE;
                surface_y[idx] = sample_surface_height(world_x, world_z, floor, &perlin);
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
            stone: self.intern_material(Color::new(95, 100, 108, 255)),
            clay: self.intern_material(Color::new(138, 116, 100, 255)),
        };
        self.terrain_materials = Some(ids);
        ids
    }

    fn feature_material_ids(&mut self) -> FeatureMaterialIds {
        if let Some(ids) = self.feature_materials {
            return ids;
        }
        let ids = FeatureMaterialIds {
            basalt: self.intern_material(Color::new(64, 66, 73, 255)),
            sandstone: self.intern_material(Color::new(196, 171, 120, 255)),
            glass: self.intern_material(Color::new(180, 220, 255, 95)),
            glow: self.intern_material(Color::new(48, 230, 255, 255)),
        };
        self.feature_materials = Some(ids);
        ids
    }

    fn paint_terrain_column(&mut self, chunk_x: u32, chunk_z: u32, surface_y: &[i32; CHUNK_AREA]) {
        let materials = self.terrain_material_ids();
        let feature_materials = self.feature_material_ids();
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

                let surface_material = if surface > water_level + 2 {
                    materials.clay
                } else {
                    materials.grass
                };
                self.set_voxel_material_i32(world_x, surface, world_z, surface_material);
                if surface > water_level {
                    for y in water_level..surface {
                        self.set_voxel_material_i32(world_x, y, world_z, materials.water);
                    }
                }
                for y in (surface + 1)..lower_void {
                    let depth = y - surface;
                    let mat = if depth <= 2 {
                        if surface > water_level + 2 {
                            materials.clay
                        } else {
                            materials.dirt
                        }
                    } else if depth <= 8 {
                        if (layer_hash(world_x, y, world_z) & 3) == 0 {
                            materials.stone
                        } else {
                            materials.dirt
                        }
                    } else {
                        let layer_band = (y + ((world_x * 3 + world_z * 5) >> 2)).abs() % 9;
                        if layer_band <= 1 {
                            materials.clay
                        } else {
                            materials.stone
                        }
                    };
                    self.set_voxel_material_i32(world_x, y, world_z, mat);
                }
            }
        }
        self.stamp_fun_features_for_column(chunk_x, chunk_z, feature_materials);
        self.mark_terrain_column_generated(chunk_x, chunk_z);
    }

    fn stamp_fun_features_for_column(
        &mut self,
        chunk_x: u32,
        chunk_z: u32,
        materials: FeatureMaterialIds,
    ) {
        let col_min_x = chunk_x as i32 * CHUNK_SIZE as i32;
        let col_max_x = col_min_x + CHUNK_SIZE as i32 - 1;
        let col_min_z = chunk_z as i32 * CHUNK_SIZE as i32;
        let col_max_z = col_min_z + CHUNK_SIZE as i32 - 1;

        let center = self.get_center();
        let terrain_y = self.get_floor_level() as f32 - 30.0;
        let mut bounds = Vec::with_capacity(VERTICAL_TORUS_DEFS.len() + 4);
        for (ox, oy, oz, major, minor) in VERTICAL_TORUS_DEFS {
            let torus_center = Vec3::new(center.x + ox, terrain_y + oy, center.z + oz);
            bounds.push(FeatureBounds::around(torus_center, major + minor + 1.0));
        }
        bounds.push(FeatureBounds::around(
            Vec3::new(center.x + 24.0, terrain_y - 10.0, center.z + 34.0),
            14.0,
        )); // cone
        bounds.push(FeatureBounds::around(
            Vec3::new(center.x + 0.0, terrain_y - 6.0, center.z + 44.0),
            18.0,
        )); // CSG sphere
        bounds.push(FeatureBounds::around(
            Vec3::new(center.x - 4.0, terrain_y - 10.0, center.z + 54.0),
            13.0,
        )); // pyramid
        bounds.push(FeatureBounds::around(
            Vec3::new(center.x - 34.0, terrain_y - 8.0, center.z + 56.0),
            13.0,
        )); // spiral sphere

        if !bounds
            .iter()
            .copied()
            .any(|b| column_intersects_bounds(col_min_x, col_max_x, col_min_z, col_max_z, b))
        {
            return;
        }

        let lower_void = self.get_lower_void() as i32;
        let y_min = (terrain_y as i32 - 40).max(0);
        let y_max = (terrain_y as i32 + 30).min(lower_void - 1);

        for world_x in col_min_x..=col_max_x {
            for world_z in col_min_z..=col_max_z {
                for y in y_min..=y_max {
                    let p = Vec3::new(world_x as f32 + 0.5, y as f32 + 0.5, world_z as f32 + 0.5);
                    let Some(material) = self.sample_feature_material(p, center, materials) else {
                        continue;
                    };
                    self.set_voxel_material_i32(world_x, y, world_z, material);
                }
            }
        }
    }

    fn sample_feature_material(
        &self,
        p: Vec3,
        center: Vec3,
        materials: FeatureMaterialIds,
    ) -> Option<MaterialId> {
        let terrain_y = self.get_floor_level() as f32 - 30.0;

        // Spiral line-sphere: a shell gated by two helical stripe patterns.
        let spiral_center = Vec3::new(center.x - 34.0, terrain_y - 8.0, center.z + 56.0);
        let spiral_local = p - spiral_center;
        let spiral_radius = 12.0;
        let shell_delta = (spiral_local.length() - spiral_radius).abs();
        if shell_delta <= 0.9 {
            let azimuth = spiral_local.z.atan2(spiral_local.x);
            let helix_a = (azimuth * 3.0 + spiral_local.y * 0.55).sin().abs();
            let helix_b = (azimuth * 2.0 - spiral_local.y * 0.8).cos().abs();
            if helix_a < 0.22 || helix_b < 0.22 {
                return Some(materials.glow);
            }
        }

        // Vertical half-toruses sticking out of terrain.
        for (idx, (ox, oy, oz, major, minor)) in VERTICAL_TORUS_DEFS.iter().copied().enumerate() {
            let torus_center = Vec3::new(center.x + ox, terrain_y + oy, center.z + oz);
            let local = p - torus_center;
            let torus_q = if (idx & 1) == 0 {
                Vec2::new(Vec2::new(local.x, local.y).length() - major, local.z)
            } else {
                Vec2::new(Vec2::new(local.z, local.y).length() - major, local.x)
            };
            let distance = torus_q.length() - minor;
            if distance <= 0.0 {
                return Some(match idx % 4 {
                    0 => materials.glass,
                    1 => materials.sandstone,
                    2 => materials.basalt,
                    _ => materials.glow,
                });
            }
        }

        // CSG-like carved sphere.
        let csg_outer_center = Vec3::new(center.x + 0.0, terrain_y - 6.0, center.z + 44.0);
        let csg_inner_center = csg_outer_center + Vec3::new(5.0, -2.0, 0.0);
        let inside_outer = (p - csg_outer_center).length() <= 16.0;
        let inside_inner = (p - csg_inner_center).length() <= 8.5;
        if inside_outer && !inside_inner {
            return Some(materials.basalt);
        }

        // Cone (tip high, base wide below).
        let cone_tip = Vec3::new(center.x + 24.0, terrain_y - 22.0, center.z + 34.0);
        let cone_local = p - cone_tip;
        let cone_height = 38.0;
        if cone_local.y >= 0.0 && cone_local.y <= cone_height {
            let t = cone_local.y / cone_height;
            let allowed_radius = 13.0 * t;
            let radial = Vec2::new(cone_local.x, cone_local.z).length();
            if radial <= allowed_radius {
                return Some(materials.sandstone);
            }
        }

        // Pyramid-ish structure (triangle vibe without mesh rasterization).
        let pyramid_tip = Vec3::new(center.x - 4.0, terrain_y - 24.0, center.z + 54.0);
        let pyramid_local = p - pyramid_tip;
        let pyramid_height = 32.0;
        if pyramid_local.y >= 0.0 && pyramid_local.y <= pyramid_height {
            let half_width = 11.0 * (pyramid_local.y / pyramid_height);
            if pyramid_local.x.abs() <= half_width && pyramid_local.z.abs() <= half_width {
                return Some(materials.basalt);
            }
        }

        None
    }
}
