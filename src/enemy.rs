//! Enemies: snails and slugs that crawl along the fixed path toward the
//! player's base.

use std::collections::HashMap;

use macroquad::prelude::*;
use serde::Deserialize;

use crate::sprites::{Direction, Sprites};
use crate::tower::{EffectType, POISON_DPS, SLOW_FACTOR, TowerType};

/// Size of a single debuff icon in world units.
const EFFECT_ICON_SIZE: f32 = 10.0;
/// Horizontal gap between two debuff icons.
const EFFECT_ICON_SPACING: f32 = 2.0;
/// Gap between the enemy sprite and the debuff icon row.
const EFFECT_ICON_MARGIN: f32 = 2.0;

/// A property of an enemy that restricts which towers are able to engage
/// it. An enemy without any attributes can be hit by every tower.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyAttribute {
    Flying,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum EnemyType {
    Snail,
    Slug,
    BigSnail,
    FlyingSnail,
}

impl EnemyType {
    pub fn max_hp(self) -> f32 {
        match self {
            EnemyType::Snail => 50.0,
            EnemyType::Slug => 30.0,
            EnemyType::BigSnail => 500.0,
            EnemyType::FlyingSnail => 40.0,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            EnemyType::Snail => 55.0,
            EnemyType::Slug => 105.0,
            EnemyType::BigSnail => 45.0,
            EnemyType::FlyingSnail => 80.0,
        }
    }

    pub fn reward(self) -> u32 {
        match self {
            EnemyType::Snail => 5,
            EnemyType::Slug => 4,
            EnemyType::BigSnail => 30,
            EnemyType::FlyingSnail => 8,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            EnemyType::Snail => 14.0,
            EnemyType::Slug => 12.0,
            EnemyType::BigSnail => 20.0,
            EnemyType::FlyingSnail => 12.0,
        }
    }

    pub fn attributes(self) -> &'static [EnemyAttribute] {
        match self {
            EnemyType::FlyingSnail => &[EnemyAttribute::Flying],
            _ => &[],
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
    /// Active debuffs mapped to their remaining duration in seconds.
    pub effects: HashMap<EffectType, f32>,
    pub direction: Direction,
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
            effects: HashMap::new(),
            direction: Direction::RIGHT,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }

    /// Whether a tower of `tower_kind` is able to hit this enemy at all.
    /// The tower must be able to cope with every attribute the enemy has.
    pub fn is_targetable_by(&self, tower_kind: TowerType) -> bool {
        self.kind
            .attributes()
            .iter()
            .all(|&attribute| tower_kind.can_target(attribute))
    }

    /// Move the enemy along the path toward the next waypoint. Returns
    /// true once it has walked past the final waypoint.
    pub fn update(&mut self, dt: f32, waypoints: &[Vec2]) {
        self.update_effects(dt);

        if self.reached_base || self.waypoint_index >= waypoints.len() {
            self.reached_base = true;
            return;
        }

        let target = waypoints[self.waypoint_index];
        let to_target = target - self.pos;

        // Update direction of enemy
        //
        self.direction = match (to_target[1] > 0.0, to_target[0] > 0.0) {
            (true, false) => Direction::BOTTOM,
            (false, true) => Direction::RIGHT,
            (true, true) => Direction::TOP,
            (false, false) => Direction::LEFT,
        };

        let dist = to_target.length();

        let mut speed = self.speed;
        if self.has_effect(EffectType::Slow) {
            speed = self.kind.speed() * SLOW_FACTOR;
        }
        let step = speed * dt;

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

    pub fn has_effect(&self, effect: EffectType) -> bool {
        self.effects.contains_key(&effect)
    }

    /// Tick every active debuff: apply its per-frame consequences and
    /// count down its remaining duration, dropping expired ones.
    fn update_effects(&mut self, dt: f32) {
        for effect in EffectType::ALL {
            if !self.has_effect(effect) {
                continue;
            }
            match effect {
                EffectType::Poison => self.hp -= POISON_DPS * dt,
                // Slow has no per-frame cost; it is read in `update`.
                EffectType::Slow => {}
            }
        }

        self.effects.retain(|_, remaining| {
            *remaining -= dt;
            *remaining > 0.0
        });
    }

    pub fn draw(&self, texture: &Texture2D, sprites: &Sprites) {
        let r = self.kind.radius();
        let half_tex = texture.width() / 2.0;
        let scale = (r * 2.0) / texture.width();

        let flip_y = match self.direction {
            Direction::LEFT => true,
            _ => false,
        };

        let rotation = match self.direction {
            Direction::TOP => -1.57,
            Direction::BOTTOM => 1.57,
            Direction::LEFT => 2.0 * 1.57,
            _ => 0.0,
        };

        draw_texture_ex(
            texture,
            self.pos.x - r,
            self.pos.y - r,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(texture.width() * scale, texture.height() * scale)),
                flip_y,
                rotation,
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

        self.draw_effect_icons(sprites);
    }

    /// Draw one icon per active debuff in a row centered below the enemy.
    /// Iterates [`EffectType::ALL`] so the icons keep a stable position
    /// instead of shuffling with the `HashMap` iteration order.
    fn draw_effect_icons(&self, sprites: &Sprites) {
        let active: Vec<EffectType> = EffectType::ALL
            .into_iter()
            .filter(|&effect| self.has_effect(effect))
            .collect();
        if active.is_empty() {
            return;
        }

        let total_width = active.len() as f32 * EFFECT_ICON_SIZE
            + (active.len() - 1) as f32 * EFFECT_ICON_SPACING;
        let mut x = self.pos.x - total_width / 2.0;
        let y = self.pos.y + self.kind.radius() + EFFECT_ICON_MARGIN;

        for effect in active {
            draw_texture_ex(
                sprites.effect_texture(effect),
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(EFFECT_ICON_SIZE, EFFECT_ICON_SIZE)),
                    ..Default::default()
                },
            );
            x += EFFECT_ICON_SIZE + EFFECT_ICON_SPACING;
        }
    }

    /// Apply the given debuffs, each for its own duration. Re-applying an
    /// effect never shortens the time left, so a weaker refresh cannot cut
    /// a longer running effect short.
    pub(crate) fn apply_effects(&mut self, effects: Vec<EffectType>) {
        for effect in effects {
            let remaining = self.effects.entry(effect).or_insert(0.0);
            *remaining = remaining.max(effect.duration());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATH: [Vec2; 2] = [Vec2::new(0.0, 0.0), Vec2::new(1000.0, 0.0)];

    fn snail() -> Enemy {
        Enemy::new(0, EnemyType::Snail, Vec2::new(0.0, 0.0))
    }

    #[test]
    fn poison_damages_over_time_and_expires() {
        let mut enemy = snail();
        enemy.apply_effects(vec![EffectType::Poison]);

        let hp_before = enemy.hp;
        enemy.update(1.0, &PATH);
        assert!((hp_before - enemy.hp - POISON_DPS).abs() < 0.001);

        for _ in 0..4 {
            enemy.update(1.0, &PATH);
        }
        assert!(!enemy.has_effect(EffectType::Poison));
    }

    #[test]
    fn slow_reduces_movement_speed() {
        let mut normal = snail();
        normal.update(1.0, &PATH);

        let mut slowed = snail();
        slowed.apply_effects(vec![EffectType::Slow]);
        slowed.update(1.0, &PATH);

        assert!((slowed.pos.x - normal.pos.x * SLOW_FACTOR).abs() < 0.001);
    }

    #[test]
    fn reapplying_an_effect_refreshes_its_duration() {
        let mut enemy = snail();
        enemy.apply_effects(vec![EffectType::Poison]);
        enemy.update(1.0, &PATH);
        assert!(enemy.effects[&EffectType::Poison] < EffectType::Poison.duration());

        enemy.apply_effects(vec![EffectType::Poison]);
        assert_eq!(
            enemy.effects[&EffectType::Poison],
            EffectType::Poison.duration()
        );
    }

    #[test]
    fn effects_are_independent_of_each_other() {
        let mut enemy = snail();
        enemy.apply_effects(vec![EffectType::Slow, EffectType::Poison]);

        for _ in 0..5 {
            enemy.update(1.0, &PATH);
        }
        assert!(!enemy.has_effect(EffectType::Poison));
        assert!(enemy.has_effect(EffectType::Slow));
    }
}
