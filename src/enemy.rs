//! Enemies: snails and slugs that crawl along the fixed path toward the
//! player's base.

use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyType {
    Snail,
    Slug,
    BigSnail,
}

impl EnemyType {
    pub fn max_hp(self) -> f32 {
        match self {
            EnemyType::Snail => 50.0,
            EnemyType::Slug => 30.0,
            EnemyType::BigSnail => 500.0,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            EnemyType::Snail => 55.0,
            EnemyType::Slug => 105.0,
            EnemyType::BigSnail => 45.0,
        }
    }

    pub fn reward(self) -> u32 {
        match self {
            EnemyType::Snail => 5,
            EnemyType::Slug => 4,
            EnemyType::BigSnail => 30,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            EnemyType::Snail => 14.0,
            EnemyType::Slug => 12.0,
            EnemyType::BigSnail => 20.0,
        }
    }
}

pub struct Enemy {
    pub id: u32,
    pub kind: EnemyType,
    pub pos: Vec2,
    pub hp: f32,
    pub speed: f32,
    pub waypoint_index: usize,
    /// Set to true once the enemy reaches the end of the path.
    pub reached_base: bool,
}

impl Enemy {
    pub fn new(id: u32, kind: EnemyType, start: Vec2) -> Self {
        Enemy {
            id,
            kind,
            pos: start,
            hp: kind.max_hp(),
            speed: kind.speed(),
            waypoint_index: 1,
            reached_base: false,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }

    /// Move the enemy along the path toward the next waypoint. Returns
    /// true once it has walked past the final waypoint.
    pub fn update(&mut self, dt: f32, waypoints: &[Vec2]) {
        if self.reached_base || self.waypoint_index >= waypoints.len() {
            self.reached_base = true;
            return;
        }
        let target = waypoints[self.waypoint_index];
        let to_target = target - self.pos;
        let dist = to_target.length();
        let step = self.speed * dt;

        if dist <= step {
            self.pos = target;
            self.waypoint_index += 1;
            if self.waypoint_index >= waypoints.len() {
                self.reached_base = true;
            }
        } else {
            self.pos += to_target / dist * step;
        }
    }

    pub fn draw(&self, texture: &Texture2D) {
        let r = self.kind.radius();
        let half_tex = texture.width() / 2.0;
        let scale = (r * 2.0) / texture.width();
        draw_texture_ex(
            texture,
            self.pos.x - r,
            self.pos.y - r,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(texture.width() * scale, texture.height() * scale)),
                ..Default::default()
            },
        );
        let _ = half_tex;

        // Health bar.
        let ratio = (self.hp / self.kind.max_hp()).clamp(0.0, 1.0);
        let bar_w = r * 2.0;
        let bar_y = self.pos.y - r - 8.0;
        draw_rectangle(
            self.pos.x - r,
            bar_y,
            bar_w,
            4.0,
            Color::new(0.2, 0.0, 0.0, 0.8),
        );
        draw_rectangle(
            self.pos.x - r,
            bar_y,
            bar_w * ratio,
            4.0,
            Color::new(0.1, 0.9, 0.1, 0.9),
        );
    }

    pub(crate) fn apply_debuf(&mut self, arg: i32) -> () {
        // if self.speed < self.kind.speed() {
        //     return;
        // }
        // self.speed = self.speed / 10.0;
    }
}
