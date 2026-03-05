use glam::Vec3;
use raylib::prelude::*;
use rayon::prelude::*;

use crate::camera::Camera;
use crate::viewplane::Viewplane;
use crate::world::{MaterialId, World, CHUNK_SIZE};

pub const MIN_STEP_SIZE: f32 = 0.02;
pub const MAX_STEP_SIZE: f32 = 4.0;
pub const MAX_RAY_STEPS: i32 = 4096;

const DDA_EPSILON: f32 = 0.0001;
const AIR_MATERIAL_ID: MaterialId = 0;

#[derive(Debug, Copy, Clone, Default)]
pub struct RenderStats {
    pub rays_cast: u32,
    pub rays_hit: u32,
    pub voxel_steps: u64,
    pub empty_chunk_skips: u32,
}

pub struct RaymarchInput<'a> {
    pub world: &'a World,
    pub camera: &'a Camera,
    pub viewplane: &'a Viewplane,
    pub draw_distance: f32,
    pub march_step_size: f32,
}

#[derive(Copy, Clone)]
struct DdaState {
    voxel_x: i32,
    voxel_y: i32,
    voxel_z: i32,
    step_x: i32,
    step_y: i32,
    step_z: i32,
    t_max_x: f32,
    t_max_y: f32,
    t_max_z: f32,
    t_delta_x: f32,
    t_delta_y: f32,
    t_delta_z: f32,
}

#[inline]
fn ray_aabb_intersection(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<(f32, f32)> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    for axis in 0..3 {
        let o = origin[axis];
        let d = dir[axis];
        let aabb_min = min[axis];
        let aabb_max = max[axis];

        if d.abs() < 1e-8 {
            if o < aabb_min || o > aabb_max {
                return None;
            }
            continue;
        }

        let inv = 1.0 / d;
        let mut t1 = (aabb_min - o) * inv;
        let mut t2 = (aabb_max - o) * inv;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }

        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_max < t_min {
            return None;
        }
    }

    if t_max < 0.0 {
        return None;
    }

    Some((t_min.max(0.0), t_max))
}

#[inline]
fn init_dda(origin: Vec3, dir: Vec3, t: f32) -> DdaState {
    let p = origin + dir * (t + DDA_EPSILON);
    let voxel_x = p.x.floor() as i32;
    let voxel_y = p.y.floor() as i32;
    let voxel_z = p.z.floor() as i32;

    let step_x = if dir.x > 0.0 {
        1
    } else if dir.x < 0.0 {
        -1
    } else {
        0
    };
    let step_y = if dir.y > 0.0 {
        1
    } else if dir.y < 0.0 {
        -1
    } else {
        0
    };
    let step_z = if dir.z > 0.0 {
        1
    } else if dir.z < 0.0 {
        -1
    } else {
        0
    };

    let t_delta_x = if step_x == 0 {
        f32::INFINITY
    } else {
        1.0 / dir.x.abs()
    };
    let t_delta_y = if step_y == 0 {
        f32::INFINITY
    } else {
        1.0 / dir.y.abs()
    };
    let t_delta_z = if step_z == 0 {
        f32::INFINITY
    } else {
        1.0 / dir.z.abs()
    };

    let t_max_x = if step_x > 0 {
        t + ((voxel_x as f32 + 1.0 - p.x) / dir.x)
    } else if step_x < 0 {
        t + ((p.x - voxel_x as f32) / -dir.x)
    } else {
        f32::INFINITY
    };
    let t_max_y = if step_y > 0 {
        t + ((voxel_y as f32 + 1.0 - p.y) / dir.y)
    } else if step_y < 0 {
        t + ((p.y - voxel_y as f32) / -dir.y)
    } else {
        f32::INFINITY
    };
    let t_max_z = if step_z > 0 {
        t + ((voxel_z as f32 + 1.0 - p.z) / dir.z)
    } else if step_z < 0 {
        t + ((p.z - voxel_z as f32) / -dir.z)
    } else {
        f32::INFINITY
    };

    DdaState {
        voxel_x,
        voxel_y,
        voxel_z,
        step_x,
        step_y,
        step_z,
        t_max_x,
        t_max_y,
        t_max_z,
        t_delta_x,
        t_delta_y,
        t_delta_z,
    }
}

#[inline]
fn step_dda(dda: &mut DdaState) -> f32 {
    if dda.t_max_x <= dda.t_max_y && dda.t_max_x <= dda.t_max_z {
        dda.voxel_x += dda.step_x;
        let next_t = dda.t_max_x;
        dda.t_max_x += dda.t_delta_x;
        next_t
    } else if dda.t_max_y <= dda.t_max_z {
        dda.voxel_y += dda.step_y;
        let next_t = dda.t_max_y;
        dda.t_max_y += dda.t_delta_y;
        next_t
    } else {
        dda.voxel_z += dda.step_z;
        let next_t = dda.t_max_z;
        dda.t_max_z += dda.t_delta_z;
        next_t
    }
}

#[inline]
fn chunk_exit_t(
    origin: Vec3,
    dir: Vec3,
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    current_t: f32,
) -> f32 {
    let chunk_min_x = (chunk_x * CHUNK_SIZE as i32) as f32;
    let chunk_min_y = (chunk_y * CHUNK_SIZE as i32) as f32;
    let chunk_min_z = (chunk_z * CHUNK_SIZE as i32) as f32;
    let chunk_max_x = chunk_min_x + CHUNK_SIZE as f32;
    let chunk_max_y = chunk_min_y + CHUNK_SIZE as f32;
    let chunk_max_z = chunk_min_z + CHUNK_SIZE as f32;

    let t_x = if dir.x > 0.0 {
        (chunk_max_x - origin.x) / dir.x
    } else if dir.x < 0.0 {
        (chunk_min_x - origin.x) / dir.x
    } else {
        f32::INFINITY
    };
    let t_y = if dir.y > 0.0 {
        (chunk_max_y - origin.y) / dir.y
    } else if dir.y < 0.0 {
        (chunk_min_y - origin.y) / dir.y
    } else {
        f32::INFINITY
    };
    let t_z = if dir.z > 0.0 {
        (chunk_max_z - origin.z) / dir.z
    } else if dir.z < 0.0 {
        (chunk_min_z - origin.z) / dir.z
    } else {
        f32::INFINITY
    };

    let mut t = t_x.min(t_y).min(t_z);
    if !t.is_finite() || t <= current_t {
        t = current_t + DDA_EPSILON;
    }
    t
}

pub fn draw_voxels(
    input: RaymarchInput<'_>,
    pixels: &mut [u8],
    width: i32,
    height: i32,
) -> RenderStats {
    debug_assert_eq!(pixels.len(), (width as usize) * (height as usize) * 4);

    let step_size = input.march_step_size.max(MIN_STEP_SIZE).min(MAX_STEP_SIZE);
    let mut num_ray_steps = (input.draw_distance / step_size).ceil() as i32;
    num_ray_steps = num_ray_steps.max(1).min(MAX_RAY_STEPS);

    let draw_distance = num_ray_steps as f32 * step_size;
    let inv_draw_distance = 1.0 / draw_distance.max(0.0001);
    let sky_limit = input.world.get_above_floor_level() as f32;
    let world_min = Vec3::ZERO;
    let world_max = Vec3::splat(input.world.dim as f32 - DDA_EPSILON);

    let tl = input
        .viewplane
        .top_left_corner_from_perspective_of(input.camera);
    let right = input.viewplane.get_right_from_perspective_of(input.camera);
    let down = input.viewplane.get_down_from_perspective_of(input.camera);
    let pixel_size = input.viewplane.size / glam::Vec2::new(width as f32, height as f32);
    let right_step = right * pixel_size.x;
    let down_step = down * pixel_size.y;
    let row_start = tl + right_step * 0.5 + down_step * 0.5;

    let cam = input.camera.pos;
    let cam_y = cam.y;
    let world_dim = input.world.dim as i32;
    let row_stride = (width as usize) * 4;

    pixels
        .par_chunks_exact_mut(row_stride)
        .enumerate()
        .map(|(y, row)| {
            let mut stats = RenderStats::default();
            let mut target = row_start + down_step * y as f32;
            for x in 0..width as usize {
                stats.rays_cast += 1;
                let ray = (target - cam).normalize();
                let mut hit_anything = false;
                let mut hit_distance = draw_distance;

                let mut accumulated_r = 0.0;
                let mut accumulated_g = 0.0;
                let mut accumulated_b = 0.0;
                let mut transmittance = 1.0;

                if let Some((mut t_enter, mut t_exit)) =
                    ray_aabb_intersection(cam, ray, world_min, world_max)
                {
                    t_enter = t_enter.max(0.0);
                    t_exit = t_exit.min(draw_distance);

                    if t_enter <= t_exit {
                        let mut t = t_enter;
                        let mut dda = init_dda(cam, ray, t);
                        let mut remaining_steps = num_ray_steps;
                        let mut last_chunk_x = i32::MIN;
                        let mut last_chunk_y = i32::MIN;
                        let mut last_chunk_z = i32::MIN;
                        let mut current_chunk_empty = false;
                        let mut current_chunk_has_transparency = false;

                        while t <= t_exit && remaining_steps > 0 {
                            if dda.voxel_x < 0
                                || dda.voxel_y < 0
                                || dda.voxel_z < 0
                                || dda.voxel_x >= world_dim
                                || dda.voxel_y >= world_dim
                                || dda.voxel_z >= world_dim
                            {
                                break;
                            }

                            let chunk_x = dda.voxel_x.div_euclid(CHUNK_SIZE as i32);
                            let chunk_y = dda.voxel_y.div_euclid(CHUNK_SIZE as i32);
                            let chunk_z = dda.voxel_z.div_euclid(CHUNK_SIZE as i32);
                            if chunk_x != last_chunk_x
                                || chunk_y != last_chunk_y
                                || chunk_z != last_chunk_z
                            {
                                let Some(chunk_meta) =
                                    input.world.chunk_meta(chunk_x, chunk_y, chunk_z)
                                else {
                                    break;
                                };
                                current_chunk_empty = chunk_meta.is_empty();
                                current_chunk_has_transparency = chunk_meta.has_transparency;
                                last_chunk_x = chunk_x;
                                last_chunk_y = chunk_y;
                                last_chunk_z = chunk_z;
                            }

                            if current_chunk_empty {
                                stats.empty_chunk_skips += 1;
                                t = chunk_exit_t(cam, ray, chunk_x, chunk_y, chunk_z, t)
                                    + DDA_EPSILON;
                                if t > t_exit {
                                    break;
                                }
                                dda = init_dda(cam, ray, t);
                                continue;
                            }

                            stats.voxel_steps += 1;
                            remaining_steps -= 1;

                            let material_id = input.world.get_voxel_material_unchecked_i32(
                                dda.voxel_x,
                                dda.voxel_y,
                                dda.voxel_z,
                            );
                            if material_id != AIR_MATERIAL_ID {
                                if !hit_anything {
                                    hit_anything = true;
                                    hit_distance = t.max(0.0);
                                }

                                let material = input.world.get_material(material_id);
                                if !current_chunk_has_transparency {
                                    let color = material.color;
                                    accumulated_r = color.r as f32;
                                    accumulated_g = color.g as f32;
                                    accumulated_b = color.b as f32;
                                    break;
                                }

                                accumulated_r += material.premul_r * transmittance;
                                accumulated_g += material.premul_g * transmittance;
                                accumulated_b += material.premul_b * transmittance;
                                transmittance *= 1.0 - material.alpha;
                                if transmittance <= 0.01 {
                                    break;
                                }
                            }

                            t = step_dda(&mut dda);
                        }
                    }
                }

                let mut color = Color::BLACK;
                if hit_anything {
                    stats.rays_hit += 1;
                    let mut brightness = 1.0 - hit_distance * inv_draw_distance;
                    brightness = brightness.max(0.0).min(1.0);
                    let lit_scale = 0.25 + brightness * 0.75;
                    color = Color::new(
                        (accumulated_r * lit_scale).max(0.0).min(255.0) as u8,
                        (accumulated_g * lit_scale).max(0.0).min(255.0) as u8,
                        (accumulated_b * lit_scale).max(0.0).min(255.0) as u8,
                        255,
                    );
                } else {
                    let sky_probe_y = cam_y + ray.y * draw_distance;
                    if sky_probe_y < sky_limit {
                        const BLUE: Vec3 = Vec3::new(0.0, 0.0, 255.0);
                        let blue = BLUE * 0.1;
                        color = Color::new(blue.x as u8, blue.y as u8, blue.z as u8, 255);
                    }
                }

                let pixel_index = x * 4;
                row[pixel_index] = color.r;
                row[pixel_index + 1] = color.g;
                row[pixel_index + 2] = color.b;
                row[pixel_index + 3] = 255;
                target += right_step;
            }
            stats
        })
        .reduce(RenderStats::default, |mut acc, row| {
            acc.rays_cast += row.rays_cast;
            acc.rays_hit += row.rays_hit;
            acc.voxel_steps += row.voxel_steps;
            acc.empty_chunk_skips += row.empty_chunk_skips;
            acc
        })
}
