use core::f32;

use chrono::{DateTime, Local};
use glam::{vec2, vec3, IVec2, Mat4, Vec2};

const MAX_VEL: f32 = 200.0;

struct BBox {
    pub min: Vec2,
    pub max: Vec2,
}

impl BBox {
    pub fn with(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn is_inside(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
}

impl Default for BBox {
    fn default() -> Self {
        Self {
            min: Vec2::NEG_INFINITY,
            max: Vec2::INFINITY,
        }
    }
}

#[derive(Default)]
struct Movement {
    pos: Vec2,
    vel: Vec2,
    acc: f32,
    // Direction and Circular Velocity in Degrees
    dir: f32,
    cvel: f32,
    pub bounds: BBox,
}

impl Movement {
    pub fn update(&mut self, elapsed_time: f32) {
        // Calculate new velocity based on acceleration
        let acceleration =
            self.acc * vec2(-self.dir.to_radians().sin(), self.dir.to_radians().cos());
        let velocity = self.vel + acceleration * elapsed_time;
        let velocity_mag = velocity.length();
        if velocity_mag > 0.0 {
            self.vel = velocity.normalize() * velocity_mag.min(MAX_VEL);
        }

        self.pos += self.vel * elapsed_time;
        self.dir += self.cvel * elapsed_time;
        if !self.bounds.is_inside(self.pos) {
            self.pos = -self.pos;
        }
    }
}

struct Timer {
    last: DateTime<Local>,
    now: DateTime<Local>,
}

impl Timer {
    pub fn tick(&mut self) -> &Timer {
        self.last = self.now;
        self.now = chrono::offset::Local::now();
        self
    }

    pub fn elapsed(&self) -> f32 {
        let delta = self.now - self.last;
        delta.num_seconds() as f32 + delta.num_milliseconds() as f32 / 1000.0
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            last: chrono::offset::Local::now(),
            now: chrono::offset::Local::now(),
        }
    }
}

pub enum Rotating {
    Left,
    Right,
    None,
}

pub struct Rusteroids {
    timer: Timer,
    player_movement: Movement,
    player_is_accelerating: bool,
    player_is_rotating: Rotating,
}

impl Rusteroids {
    pub fn new() -> Self {
        Self {
            timer: Timer::default(),
            player_movement: Movement::default(),
            player_is_accelerating: false,
            player_is_rotating: Rotating::None,
        }
    }

    pub fn set_bounds(&mut self, res: IVec2) {
        let half_res = vec2(res.x as f32 / 2.0, res.y as f32 / 2.0);
        self.player_movement.bounds = BBox::with(-half_res, half_res);
    }

    pub fn tick(&mut self) {
        let elapsed = self.timer.tick().elapsed();
        self.player_movement.cvel = match self.player_is_rotating {
            Rotating::Left => 180.0_f32,
            Rotating::Right => -180.0_f32,
            Rotating::None => 0.0_f32,
        };
        self.player_movement.acc = match self.player_is_accelerating {
            true => 150.0,
            false => 0.0,
        };
        self.player_movement.update(elapsed);
    }

    pub fn get_battleship_model_matrix(&self) -> [f32; 16] {
        Mat4::from_translation(vec3(
            self.player_movement.pos.x,
            self.player_movement.pos.y,
            0.0,
        ))
        .mul_mat4(&Mat4::from_rotation_z(
            self.player_movement.dir.to_radians(),
        ))
        .to_cols_array()
    }

    pub fn update_keys(&mut self, w: bool, a: bool, d: bool) {
        self.player_is_rotating = if a && !d {
            Rotating::Left
        } else if d && !a {
            Rotating::Right
        } else {
            Rotating::None
        };
        self.player_is_accelerating = w;
    }
}
