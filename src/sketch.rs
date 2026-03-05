use glam::{UVec3, Vec2, Vec3};
use raylib::prelude::*;

use crate::camera::Camera;
use crate::viewplane::Viewplane;
use crate::world::{Block, World, AIR};
use crate::{CHUNK_GEN_RADIUS, DIMS, MARCH_STEP_SIZE, NUM_RAY_STEPS};
use crate::{UP, WORLD_SIZE};

pub const FRAMES_PER_SECOND: u32 = 60;
const MIN_DRAW_DISTANCE: f32 = 2.0;
const MAX_DRAW_DISTANCE: f32 = 2000.0;
const MIN_STEP_SIZE: f32 = 0.02;
const MAX_STEP_SIZE: f32 = 4.0;
const MAX_RAY_STEPS: i32 = 4096;
const DISTANCE_FACTOR: f32 = 1.1;
const STEP_FACTOR: f32 = 1.1;
const MOUSE_LOOK_SENSITIVITY: f32 = 0.003;
const MAX_VIEW_ALIGNMENT_WITH_UP: f32 = 0.995;

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
    pub draw_distance: f32,
    pub march_step_size: f32,
    pub fps: i32,
    pub mouse_scale: Vec2,
    pub mouse_look_locked: bool,
}

impl State {
    pub fn new() -> Self {
        let mut world = Box::new(World::new(WORLD_SIZE));
        world.gen_floor(Block::new(255, 255, 255, 255));
        world.gen_cube(
            Vec3::new(1.0, world.get_above_floor_level() as f32 - 1.0, 1.0),
            Vec3::new(1.0, 2.0, 1.0),
            Block::new(255, 0, 0, 255),
        );

        // put a cube in the 4 corners
        // white
        let cube_color = Block::new(255, 255, 255, 255);
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

        // Transparent reference geometry for compositing tests.
        let glass = Block::new(180, 220, 255, 85);
        world.gen_cube(
            Vec3::new(8.0, world.get_above_floor_level() as f32 - 10.0, 8.0),
            Vec3::new(1.0, 10.0, 8.0),
            glass,
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
            draw_distance: NUM_RAY_STEPS as f32 * MARCH_STEP_SIZE,
            march_step_size: MARCH_STEP_SIZE,
            fps: 0,
            mouse_scale: Vec2::ONE,
            mouse_look_locked: false,
        }
    }
}

struct UiLayout {
    panel: Rectangle,
    dist_dec: Rectangle,
    dist_inc: Rectangle,
    step_dec: Rectangle,
    step_inc: Rectangle,
}

fn clamp_render_budget(state: &mut State) {
    state.draw_distance = state
        .draw_distance
        .max(MIN_DRAW_DISTANCE)
        .min(MAX_DRAW_DISTANCE);
    state.march_step_size = state.march_step_size.max(MIN_STEP_SIZE).min(MAX_STEP_SIZE);
}

fn ui_layout(screen_width: i32, _screen_height: i32) -> UiLayout {
    let panel_width = 280.0;
    let panel_height = 148.0;
    let panel_x = screen_width as f32 - panel_width - 16.0;
    let panel_y = 16.0;
    let button_w = 28.0;
    let button_h = 22.0;

    let dist_row_y = panel_y + 66.0;
    let step_row_y = panel_y + 102.0;
    let dec_x = panel_x + panel_width - 72.0;
    let inc_x = panel_x + panel_width - 36.0;

    UiLayout {
        panel: Rectangle::new(panel_x, panel_y, panel_width, panel_height),
        dist_dec: Rectangle::new(dec_x, dist_row_y, button_w, button_h),
        dist_inc: Rectangle::new(inc_x, dist_row_y, button_w, button_h),
        step_dec: Rectangle::new(dec_x, step_row_y, button_w, button_h),
        step_inc: Rectangle::new(inc_x, step_row_y, button_w, button_h),
    }
}

fn point_in_rect(p: Vec2, r: Rectangle) -> bool {
    p.x >= r.x && p.x <= r.x + r.width && p.y >= r.y && p.y <= r.y + r.height
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
        match state.mode {
            Mode::Fly => {
                state.mouse_look_locked = true;
                rl.disable_cursor();
            }
            Mode::Orbit => {
                state.mouse_look_locked = false;
                rl.enable_cursor();
            }
        }
    }

    // if r is pressed, reset camera
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_R) {
        state.camera.reset();
    }

    // Toggle freelook mouse capture.
    if state.mode == Mode::Fly && rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_TAB) {
        state.mouse_look_locked = !state.mouse_look_locked;
        if state.mouse_look_locked {
            rl.disable_cursor();
        } else {
            rl.enable_cursor();
        }
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

    // y and h to move the view angle up and down
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_Y) {
        state.camera.dir.y += rotation_speed;
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_H) {
        state.camera.dir.y -= rotation_speed;
    }

    // t and g to move the viewplane forward and backward
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_T) {
        state.camera.viewplane_distance -= cam_speed;
    }
    if rl.is_key_down(raylib::consts::KeyboardKey::KEY_G) {
        state.camera.viewplane_distance += cam_speed;
    }

    // Mouse freelook in fly mode while cursor is captured.
    if state.mode == Mode::Fly && state.mouse_look_locked {
        let scaled_delta = rl.get_mouse_delta();
        let inv_mouse_scale_x = 1.0 / state.mouse_scale.x.max(0.0001);
        let inv_mouse_scale_y = 1.0 / state.mouse_scale.y.max(0.0001);
        let delta_x = scaled_delta.x * inv_mouse_scale_x;
        let delta_y = scaled_delta.y * inv_mouse_scale_y;

        let yaw = -delta_x * MOUSE_LOOK_SENSITIVITY;
        let pitch = -delta_y * MOUSE_LOOK_SENSITIVITY;

        if yaw != 0.0 {
            state.camera.rotate(UP, yaw);
        }
        if pitch != 0.0 {
            let old_dir = state.camera.dir;
            state.camera.rotate(state.camera.get_right(), pitch);
            if state.camera.dir.dot(UP).abs() > MAX_VIEW_ALIGNMENT_WITH_UP {
                state.camera.dir = old_dir;
            }
        }
        state.camera.dir = state.camera.dir.normalize();
    }

    // Rendering budget keyboard controls.
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_MINUS) {
        state.draw_distance /= DISTANCE_FACTOR;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_EQUAL) {
        state.draw_distance *= DISTANCE_FACTOR;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_COMMA) {
        state.march_step_size /= STEP_FACTOR;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_PERIOD) {
        state.march_step_size *= STEP_FACTOR;
    }
    if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_BACKSPACE) {
        state.draw_distance = NUM_RAY_STEPS as f32 * MARCH_STEP_SIZE;
        state.march_step_size = MARCH_STEP_SIZE;
    }
    clamp_render_budget(state);

    // High-resolution overlay button input.
    let screen_width = rl.get_screen_width();
    let screen_height = rl.get_screen_height();
    let scaled_mouse = rl.get_mouse_position();
    let inv_mouse_scale_x = 1.0 / state.mouse_scale.x.max(0.0001);
    let inv_mouse_scale_y = 1.0 / state.mouse_scale.y.max(0.0001);
    let ui_mouse = Vec2::new(
        scaled_mouse.x * inv_mouse_scale_x,
        scaled_mouse.y * inv_mouse_scale_y,
    );
    if !state.mouse_look_locked && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        let layout = ui_layout(screen_width, screen_height);
        if point_in_rect(ui_mouse, layout.dist_dec) {
            state.draw_distance /= DISTANCE_FACTOR;
        }
        if point_in_rect(ui_mouse, layout.dist_inc) {
            state.draw_distance *= DISTANCE_FACTOR;
        }
        if point_in_rect(ui_mouse, layout.step_dec) {
            state.march_step_size /= STEP_FACTOR;
        }
        if point_in_rect(ui_mouse, layout.step_inc) {
            state.march_step_size *= STEP_FACTOR;
        }
        clamp_render_budget(state);
    }
}

pub fn step(rl: &mut RaylibHandle, state: &mut State) {
    // Generate chunks around the camera.
    let cam_chunk_pos = state.world.to_chunk_pos(state.camera.pos);
    let chunk_dim = state.world.chunk_dim as i32;
    for x in -CHUNK_GEN_RADIUS..=CHUNK_GEN_RADIUS {
        for y in -CHUNK_GEN_RADIUS..=CHUNK_GEN_RADIUS {
            for z in -CHUNK_GEN_RADIUS..=CHUNK_GEN_RADIUS {
                let chunk_x = cam_chunk_pos.x as i32 + x;
                let chunk_y = cam_chunk_pos.y as i32 + y;
                let chunk_z = cam_chunk_pos.z as i32 + z;
                if chunk_x < 0
                    || chunk_x >= chunk_dim
                    || chunk_y < 0
                    || chunk_y >= chunk_dim
                    || chunk_z < 0
                    || chunk_z >= chunk_dim
                {
                    continue;
                }

                let chunk_pos = UVec3::new(chunk_x as u32, chunk_y as u32, chunk_z as u32);
                if !state.world.is_chunk_genned(chunk_pos) {
                    state.world.gen_terrain(chunk_pos);
                }
            }
        }
    }

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

pub fn draw_voxels(state: &State, d: &mut RaylibTextureMode<RaylibDrawHandle>) {
    let width = DIMS.x as i32;
    let height = DIMS.y as i32;
    let step_size = state.march_step_size.max(MIN_STEP_SIZE).min(MAX_STEP_SIZE);
    let mut num_ray_steps = (state.draw_distance / step_size).ceil() as i32;
    num_ray_steps = num_ray_steps.max(1).min(MAX_RAY_STEPS);
    let max_march_distance = num_ray_steps as f32 * step_size;
    let inv_max_march_distance = 1.0 / max_march_distance;
    let sky_limit = state.world.get_above_floor_level() as f32;

    let tl = state
        .viewplane
        .top_left_corner_from_perspective_of(&state.camera);
    let right = state.viewplane.get_right_from_perspective_of(&state.camera);
    let down = state.viewplane.get_down_from_perspective_of(&state.camera);
    let pixel_size = state.viewplane.size / DIMS.as_vec2();
    let right_step = right * pixel_size.x;
    let down_step = down * pixel_size.y;
    let mut row_target = tl + right_step * 0.5 + down_step * 0.5;

    let cam = state.camera.pos;
    let cam_x = cam.x;
    let cam_y = cam.y;
    let cam_z = cam.z;

    for y in 0..height {
        let mut target = row_target;
        for x in 0..width {
            let ray = (target - cam).normalize();

            let mut pos_x = cam_x;
            let mut pos_y = cam_y;
            let mut pos_z = cam_z;
            let mut hit_distance = max_march_distance;
            let mut hit_anything = false;

            let mut accumulated_r = 0.0;
            let mut accumulated_g = 0.0;
            let mut accumulated_b = 0.0;
            let mut transmittance = 1.0;
            let mut last_voxel_x = i32::MIN;
            let mut last_voxel_y = i32::MIN;
            let mut last_voxel_z = i32::MIN;

            for step_idx in 0..num_ray_steps {
                pos_x += ray.x * step_size;
                pos_y += ray.y * step_size;
                pos_z += ray.z * step_size;

                let voxel_x = pos_x.floor() as i32;
                let voxel_y = pos_y.floor() as i32;
                let voxel_z = pos_z.floor() as i32;
                if voxel_x == last_voxel_x && voxel_y == last_voxel_y && voxel_z == last_voxel_z {
                    continue;
                }
                last_voxel_x = voxel_x;
                last_voxel_y = voxel_y;
                last_voxel_z = voxel_z;

                let block = state.world.get_voxel_i32(voxel_x, voxel_y, voxel_z);
                if block == AIR {
                    continue;
                }

                if !hit_anything {
                    hit_anything = true;
                    hit_distance = (step_idx + 1) as f32 * step_size;
                }

                // Front-to-back alpha compositing.
                let alpha = block.a as f32 / 255.0;
                accumulated_r += block.r as f32 * alpha * transmittance;
                accumulated_g += block.g as f32 * alpha * transmittance;
                accumulated_b += block.b as f32 * alpha * transmittance;
                transmittance *= 1.0 - alpha;
                if transmittance <= 0.01 {
                    break;
                }
            }

            let mut color = Color::BLACK;
            if hit_anything {
                let mut brightness = 1.0 - hit_distance * inv_max_march_distance;
                brightness = brightness.max(0.0).min(1.0);
                let lit_scale = 0.25 + brightness * 0.75;
                color = Color::new(
                    (accumulated_r * lit_scale).max(0.0).min(255.0) as u8,
                    (accumulated_g * lit_scale).max(0.0).min(255.0) as u8,
                    (accumulated_b * lit_scale).max(0.0).min(255.0) as u8,
                    255,
                );
            } else {
                if pos_y < sky_limit {
                    const BLUE: Vec3 = Vec3::new(0.0, 0.0, 255.0);
                    let blue = BLUE * 0.1;
                    color = Color::new(blue.x as u8, blue.y as u8, blue.z as u8, 255);
                }
            }
            d.draw_rectangle(x, y, 1, 1, color);
            target += right_step;
        }
        row_target += down_step;
    }
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

pub fn draw(state: &State, d: &mut RaylibTextureMode<RaylibDrawHandle>) {
    draw_voxels(state, d);
}

fn draw_button(d: &mut RaylibDrawHandle, rect: Rectangle, label: &str) {
    d.draw_rectangle(
        rect.x as i32,
        rect.y as i32,
        rect.width as i32,
        rect.height as i32,
        Color::new(40, 40, 40, 220),
    );
    d.draw_rectangle_lines(
        rect.x as i32,
        rect.y as i32,
        rect.width as i32,
        rect.height as i32,
        Color::new(200, 200, 200, 255),
    );
    d.draw_text(label, rect.x as i32 + 9, rect.y as i32 + 4, 16, Color::WHITE);
}

pub fn draw_ui_overlay(state: &State, d: &mut RaylibDrawHandle) {
    let screen_width = d.get_screen_width();
    let screen_height = d.get_screen_height();
    let layout = ui_layout(screen_width, screen_height);

    let step_size = state.march_step_size.max(MIN_STEP_SIZE).min(MAX_STEP_SIZE);
    let mut num_ray_steps = (state.draw_distance / step_size).ceil() as i32;
    num_ray_steps = num_ray_steps.max(1).min(MAX_RAY_STEPS);
    let draw_distance = num_ray_steps as f32 * step_size;
    let pixel_budget = DIMS.x as i64 * DIMS.y as i64 * num_ray_steps as i64;

    d.draw_rectangle(
        layout.panel.x as i32,
        layout.panel.y as i32,
        layout.panel.width as i32,
        layout.panel.height as i32,
        Color::new(0, 0, 0, 170),
    );
    d.draw_rectangle_lines(
        layout.panel.x as i32,
        layout.panel.y as i32,
        layout.panel.width as i32,
        layout.panel.height as i32,
        Color::new(180, 180, 180, 255),
    );

    let mode_label = match state.mode {
        Mode::Orbit => "Orbit",
        Mode::Fly => "Fly",
    };

    d.draw_text("Perf Overlay", layout.panel.x as i32 + 10, layout.panel.y as i32 + 8, 18, Color::WHITE);
    d.draw_text(
        &format!("FPS: {}", state.fps),
        layout.panel.x as i32 + 10,
        layout.panel.y as i32 + 32,
        18,
        Color::GREEN,
    );
    d.draw_text(
        &format!("Mode: {}", mode_label),
        layout.panel.x as i32 + 130,
        layout.panel.y as i32 + 32,
        18,
        Color::WHITE,
    );

    d.draw_text(
        &format!("Draw Dist: {:>6.2}", draw_distance),
        layout.panel.x as i32 + 10,
        layout.panel.y as i32 + 68,
        18,
        Color::WHITE,
    );
    d.draw_text(
        &format!("Step Size: {:>6.3}", step_size),
        layout.panel.x as i32 + 10,
        layout.panel.y as i32 + 104,
        18,
        Color::WHITE,
    );
    d.draw_text(
        &format!("Steps: {}  Budget: {}", num_ray_steps, pixel_budget),
        layout.panel.x as i32 + 10,
        layout.panel.y as i32 + 128,
        16,
        Color::new(200, 200, 200, 255),
    );

    draw_button(d, layout.dist_dec, "-");
    draw_button(d, layout.dist_inc, "+");
    draw_button(d, layout.step_dec, "-");
    draw_button(d, layout.step_inc, "+");

    d.draw_text(
        "Keys: Tab mouse-lock, [-]/[+] dist, [,]/[.] step, Backspace reset",
        16,
        screen_height - 28,
        18,
        Color::new(220, 220, 220, 255),
    );

    d.draw_text(
        if state.mouse_look_locked {
            "Mouse Look: ON"
        } else {
            "Mouse Look: OFF"
        },
        layout.panel.x as i32 + 130,
        layout.panel.y as i32 + 8,
        18,
        if state.mouse_look_locked {
            Color::GREEN
        } else {
            Color::WHITE
        },
    );

    let cx = screen_width / 2;
    let cy = screen_height / 2;
    d.draw_line(cx - 8, cy, cx + 8, cy, Color::GREEN);
    d.draw_line(cx, cy - 8, cx, cy + 8, Color::GREEN);
}
