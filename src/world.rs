use glam::Vec3;

#[derive(Debug, Clone)]
pub struct Object {
    pub pos: Vec3,
    pub size: Vec3,
    pub color: Vec3,
}

#[derive(Debug)]
pub struct World {
    pub dim: usize,
    pub pos: Vec3,
    pub voxels: Vec<Vec<Vec<Option<Vec3>>>>,
    pub genned_objects: Vec<Object>,
}

impl World {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            // pos: Vec3::new(dim as f32 / 2.0, dim as f32 / 2.0, dim as f32 / 2.0),
            pos: Vec3::new(0.0, 0.0, 0.0),
            voxels: Self::gen_empty_voxel_array(dim),
            genned_objects: Vec::new(),
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
        self.voxels = Self::gen_empty_voxel_array(self.dim);
    }

    pub fn gen_empty_voxel_array(dim: usize) -> Vec<Vec<Vec<Option<Vec3>>>> {
        vec![vec![vec![None; dim]; dim]; dim]
    }

    pub fn gen_cube(&mut self, pos: Vec3, size: Vec3, block: Vec3) {
        for x in 0..size.x as usize {
            for y in 0..size.y as usize {
                for z in 0..size.z as usize {
                    let voxel_pos = pos + Vec3::new(x as f32, y as f32, z as f32);
                    if self.is_in_bounds(voxel_pos) {
                        self.voxels[pos.x as usize + x][pos.y as usize + y][pos.z as usize + z] =
                            Some(block);
                    }
                }
            }
        }
        self.genned_objects.push(Object {
            pos,
            size,
            color: block,
        });
    }

    pub fn gen_sphere(&mut self, pos: Vec3, radius: f32, block: Vec3) {
        let radius = radius as i32;
        for x in -radius..=radius {
            for y in -radius..=radius {
                for z in -radius..=radius {
                    if Vec3::new(x as f32, y as f32, z as f32).length() <= radius as f32 {
                        let voxel_pos = pos + Vec3::new(x as f32, y as f32, z as f32);
                        if self.is_in_bounds(voxel_pos) {
                            self.voxels[pos.x as usize + x as usize][pos.y as usize + y as usize]
                                [pos.z as usize + z as usize] = Some(block);
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

    pub fn get_floor_level(&self) -> usize {
        self.dim - 1
    }

    pub fn get_above_floor_level(&self) -> usize {
        self.dim - 2
    }

    pub fn gen_floor(&mut self, block: Vec3) {
        let floor_level = self.get_floor_level();
        for x in 0..self.dim {
            for z in 0..self.dim {
                self.voxels[x][floor_level][z] = Some(block);
            }
        }
    }
}
