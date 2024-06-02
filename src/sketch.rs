use std::vec;

use glam::{UVec3, Vec2, Vec3};
use raylib::prelude::*;

use crate::camera::Camera;
use crate::viewplane::Viewplane;
use crate::world::{GetVoxelResult, World, CHUNK_SIZE};
use crate::UP;
use crate::{DIMS, MARCH_STEP_SIZE, NUM_RAY_STEPS};

pub const FRAMES_PER_SECOND: u32 = 60;

#[derive(Debug, Eq, PartialEq)]
pub enum Mode {
    Orbit,
    Fly,
}

pub struct State {
    pub running: bool,
    pub time_since_last_update: f32,

    pub world: Box<World>,
    pub camera: Box<Camera>,
    pub viewplane: Box<Viewplane>,

    pub mode: Mode,
    pub chunks_to_generate: Vec<UVec3>,
}

impl State {
    pub fn new() -> Self {
        let mut world = Box::new(World::new(512));
        world.gen_floor(Vec3::new(255.0, 255.0, 255.0));
        world.gen_cube(
            Vec3::new(1.0, world.get_above_floor_level() as f32 - 1.0, 1.0),
            Vec3::new(1.0, 2.0, 1.0),
            Vec3::new(255.0, 0.0, 0.0),
        );

        // put a cube in the 4 corners
        // white
        let cube_color = Vec3::new(255.0, 255.0, 255.0);
        world.gen_cube(
            Vec3::new(0.0, world.get_above_floor_level() as f32 - 1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            cube_color,
        );
        world.gen_cube(
            Vec3::new(
                0.0,
                world.get_above_floor_level() as f32 - 1.0,
                world.dim as f32 - 1.0,
            ),
            Vec3::new(1.0, 1.0, 1.0),
            cube_color,
        );
        world.gen_cube(
            Vec3::new(
                world.dim as f32 - 1.0,
                world.get_above_floor_level() as f32 - 1.0,
                0.0,
            ),
            Vec3::new(1.0, 1.0, 1.0),
            cube_color,
        );
        world.gen_cube(
            Vec3::new(
                world.dim as f32 - 1.0,
                world.get_above_floor_level() as f32 - 1.0,
                world.dim as f32 - 1.0,
            ),
            Vec3::new(1.0, 1.0, 1.0),
            cube_color,
        );

        // fill the whole world with cube
        // world.gen_cube(
        //     Vec3::new(0.0, 0.0, 0.0),
        //     Vec3::new(world.dim as f32, world.dim as f32, world.dim as f32),
        //     Vec3::new(255.0, 255.0, 255.0),
        // );

        // for every chunk in dim call world.gen_terrain with the chunk_pos
        // let chunk_dims = world.dim / CHUNK_SIZE;
        // for x in 0..chunk_dims {
        //     for z in 0..chunk_dims {
        //         world.gen_terrain(UVec3::new(x as u32, 0, z as u32));
        //     }
        // }

        let camera = Box::new(Camera::new(
            Vec3::new(0.0, world.get_above_floor_level() as f32, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            3.0,
        ));

        let viewplane = Box::new(Viewplane::new(Vec2::new(4.0, 3.0), 4.0 / 3.0));

        Self {
            running: true,
            time_since_last_update: 0.0,

            world,
            camera,
            viewplane,

            mode: Mode::Orbit,
            chunks_to_generate: Vec::new(),
        }
    }
}

pub fn process_events_and_input(rl: &mut RaylibHandle, state: &mut State) {
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE) {
        state.running = false;
    }

    // if m is pressed, toggle mode
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_M) {
        state.mode = match state.mode {
            Mode::Orbit => Mode::Fly,
            Mode::Fly => Mode::Orbit,
        };
    }

    // if r is pressed, reset camera
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_R) {
        state.camera.reset();
    }

    let mut cam_speed = 0.1;
    let mut rotation_speed = 0.02;
    const HIGH_SPEED_MULTIPLIER: f32 = 4.0;

    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT_SHIFT) {
        cam_speed *= HIGH_SPEED_MULTIPLIER;
        rotation_speed *= HIGH_SPEED_MULTIPLIER;
    }

    // if w is pressed, move forward
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_W) {
        state.camera.pos += state.camera.dir * cam_speed;
    }
    // if s is pressed, move backward
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_S) {
        state.camera.pos -= state.camera.dir * cam_speed;
    }
    // if a is pressed, move left
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_A) {
        state.camera.pos -= state.camera.get_right() * cam_speed;
    }
    // if d is pressed, move right
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_D) {
        state.camera.pos += state.camera.get_right() * cam_speed;
    }
    // if space is pressed, move up
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_SPACE) {
        state.camera.pos += state.camera.get_up() * cam_speed;
    }
    // if left control is pressed, move down
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT_CONTROL) {
        state.camera.pos -= state.camera.get_up() * cam_speed;
    }

    // if q and e are pressed, rotate the camera
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_Q) {
        state.camera.rotate(UP, rotation_speed);
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_E) {
        state.camera.rotate(UP, -rotation_speed);
    }

    // t and g to move the viewplane forward and backward
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_T) {
        state.camera.viewplane_distance -= cam_speed;
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_G) {
        state.camera.viewplane_distance += cam_speed;
    }
}

pub fn step(rl: &mut RaylibHandle, rlt: &mut RaylibThread, state: &mut State) {
    // global mode
    // if mode == Mode.ORBIT:
    //     tm = 1.0
    //     t = pygame.time.get_ticks() / 1000 * tm
    //     orbit_radius = 10
    //     orbit_center = world.get_center()
    //     cam_height = camera.pos.y
    //     camera.pos = (
    //         glm.vec3(math.sin(t) * orbit_radius, 0, math.cos(t) * orbit_radius)
    //         + orbit_center
    //     )
    //     camera.dir = glm.normalize(world.get_center() - camera.pos)
    //     camera.pos.y = cam_height

    if state.mode == Mode::Orbit {
        let tm = 1.0;
        let t = rl.get_time() * tm;
        let orbit_radius: f32 = 10.0;
        let orbit_center = state.world.get_center();
        let cam_height = state.camera.pos.y;
        state.camera.pos = Vec3::new(
            t.sin() as f32 * orbit_radius,
            0.0,
            t.cos() as f32 * orbit_radius,
        ) + orbit_center;
        state.camera.dir = (state.world.get_center() - state.camera.pos).normalize();
        state.camera.pos.y = cam_height;
    }

    // println!(
    //     "pos: ({:.3}, {:.3}, {:.3}), dir: ({:.3}, {:.3}, {:.3}), vpd: {:.3}",
    //     state.camera.pos.x,
    //     state.camera.pos.y,
    //     state.camera.pos.z,
    //     state.camera.dir.x,
    //     state.camera.dir.y,
    //     state.camera.dir.z,
    //     state.camera.viewplane_distance
    // );
}

pub fn draw_voxels(state: &State, d: &mut RaylibTextureMode<RaylibDrawHandle>) -> Vec<UVec3> {
    let mut chunks_to_generate: Vec<UVec3> = vec![];

    let mut x: usize = 0;
    let mut y: usize = 0;
    for target in state.viewplane.get_targets(&state.camera, DIMS.as_vec2()) {
        let ray = target - state.camera.pos;
        let ray = ray.normalize();
        let mut hit = false;
        let mut dist_to_hit = NUM_RAY_STEPS as f32 * MARCH_STEP_SIZE; // starts at max
        let mut pos = state.camera.pos;
        let mut voxel: Option<Vec3> = None;
        for _ in 0..NUM_RAY_STEPS {
            pos += ray * MARCH_STEP_SIZE;
            let wp = pos.floor(); // remove the decimal part
            if state.world.is_in_bounds(wp) {
                let get_voxel_result = state.world.get_voxel(wp);
                match get_voxel_result {
                    GetVoxelResult::Voxel { color: v } => {
                        hit = true;
                        dist_to_hit = (pos - state.camera.pos).length();
                        voxel = Some(v);
                        break;
                    }
                    GetVoxelResult::ChunkNotGenerated => {
                        let chunk_pos = state.world.to_chunk_pos(wp);
                        if !chunks_to_generate.contains(&chunk_pos) {
                            chunks_to_generate.push(chunk_pos);
                        }
                    }
                    GetVoxelResult::NoVoxel => {}
                }
            }
        }
        let mut color = Color::BLACK;
        if hit {
            let mut brightness = 1.0 - dist_to_hit / (NUM_RAY_STEPS as f32 * MARCH_STEP_SIZE);
            brightness = brightness.max(0.0).min(1.0);
            if let Some(voxel) = voxel {
                color = Color::new(
                    (voxel.x * brightness) as u8,
                    (voxel.y * brightness) as u8,
                    (voxel.z * brightness) as u8,
                    255,
                );
            }
        } else {
            // if the position is above the floor level, color it blue
            if pos.y < state.world.get_above_floor_level() as f32 {
                const BLUE: Vec3 = Vec3::new(0.0, 0.0, 255.0);
                let blue = BLUE * 0.1;
                color = Color::new(blue.x as u8, blue.y as u8, blue.z as u8, 255);
            }
        }
        d.draw_rectangle(x as i32, y as i32, 1, 1, color);

        x += 1;
        if x >= DIMS.x as usize {
            x = 0;
            y += 1;
        }
    }

    chunks_to_generate
}

pub fn draw_map(state: &State, d: &mut RaylibTextureMode<RaylibDrawHandle>) {
    let map_offset = Vec2::new(state.world.get_center().x, state.world.get_center().z);
    let map_offset = map_offset + Vec2::new(5.0, 5.0);
    let map_scale = 1.0;

    // draw world bounds
    let world_size = Vec2::new(state.world.dim as f32, state.world.dim as f32) * map_scale;
    let world_pos = Vec2::ZERO + map_offset * map_scale;
    d.draw_rectangle(
        world_pos.x as i32,
        world_pos.y as i32,
        world_size.x as i32,
        world_size.y as i32,
        Color::BLUE,
    );

    // draw center of generated objects
    for obj in state.world.genned_objects.iter() {
        let obj_pos = Vec2::new(obj.pos.x, obj.pos.z) * map_scale + map_offset * map_scale;
        d.draw_circle(
            obj_pos.x as i32,
            obj_pos.y as i32,
            2.0,
            Color::new(255, 255, 255, 255),
        );
    }

    // draw camera
    let cam_pos =
        Vec2::new(state.camera.pos.x, state.camera.pos.z) * map_scale + map_offset * map_scale;
    d.draw_circle(cam_pos.x as i32, cam_pos.y as i32, 2.0, Color::GREEN);

    // draw camera dir
    let cam_dir = state.camera.dir.normalize();
    let cam_dir = Vec2::new(cam_dir.x, cam_dir.z);
    let end = cam_pos + cam_dir * 10.0;
    d.draw_line(
        cam_pos.x as i32,
        cam_pos.y as i32,
        end.x as i32,
        end.y as i32,
        Color::GREEN,
    );

    // draw viewplane
    let top_left = state
        .viewplane
        .top_left_corner_from_perspective_of(&state.camera);
    let right = state.viewplane.get_right_from_perspective_of(&state.camera);
    let bottom_right = top_left + right * state.viewplane.size.x;
    let tl_flat = Vec2::new(top_left.x, top_left.z) * map_scale + map_offset * map_scale;
    let br_flat = Vec2::new(bottom_right.x, bottom_right.z) * map_scale + map_offset * map_scale;
    d.draw_line(
        tl_flat.x as i32,
        tl_flat.y as i32,
        br_flat.x as i32,
        br_flat.y as i32,
        Color::RED,
    );
}

pub fn draw(state: &State, d: &mut RaylibTextureMode<RaylibDrawHandle>) -> Vec<UVec3> {
    d.draw_text("Voxels", 12, 12, 12, Color::WHITE);

    let chunks_to_generate = draw_voxels(state, d);
    // draw_map(state, d);

    let mouse_pos = d.get_mouse_position();
    d.draw_circle(mouse_pos.x as i32, mouse_pos.y as i32, 6.0, Color::GREEN);
    chunks_to_generate
}
