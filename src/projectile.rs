//! Projectiles fired by towers, homing toward a specific enemy (tracked by
//! stable id, since the enemy vector is compacted each frame).

use crate::{
    enemy::Enemy,
    tower::{EffectType, TowerType},
};
use macroquad::prelude::*;

const PROJECTILE_SPEED: f32 = 420.0;
const HIT_DISTANCE: f32 = 10.0;

pub struct Projectile {
    pub pos: Vec2,
    pub target_id: u32,
    pub damage: f32,
    pub splash_radius: f32,
    pub effects: Vec<EffectType>,
    /// Type of the tower that fired this projectile; decides which
    /// enemies it may damage (see [`Enemy::is_targetable_by`]).
    pub tower_kind: TowerType,
}

impl Projectile {
    pub fn new(
        pos: Vec2,
        target_id: u32,
        damage: f32,
        splash_radius: f32,
        effects: Vec<EffectType>,
        tower_kind: TowerType,
    ) -> Self {
        Projectile {
            pos,
            target_id,
            damage,
            splash_radius,
            effects,
            tower_kind,
        }
    }

    /// Move toward the target enemy. Returns true if the projectile hit
    /// (and should be removed) this frame.
    pub fn update(&mut self, dt: f32, enemies: &[Enemy]) -> bool {
        let Some(target) = enemies.iter().find(|e| e.id == self.target_id) else {
            return true; // target no longer exists
        };
        if target.is_dead() || target.reached_base {
            return true;
        }

        let to_target = target.pos - self.pos;
        let dist = to_target.length();
        if dist <= HIT_DISTANCE {
            return true;
        }
        let step = PROJECTILE_SPEED * dt;
        if step >= dist {
            self.pos = target.pos;
        } else {
            self.pos += to_target / dist * step;
        }
        false
    }

    /// Apply damage to the target, and to any nearby enemies if this
    /// projectile has a splash radius.
    pub fn apply_damage(&self, enemies: &mut [Enemy]) {
        let damage = self.damage;
        self.for_each_affected(enemies, |enemy| enemy.hp -= damage);
    }

    /// Apply effect to the target, and to any nearby enemies if this
    /// projectile has a splash radius.
    pub fn apply_effect(&self, enemies: &mut [Enemy]) {
        if self.effects.is_empty() {
            return;
        }
        let effects = &self.effects;
        self.for_each_affected(enemies, |enemy| enemy.apply_effects(effects.clone()));
    }

    /// Run `apply` on every enemy this projectile hits: the target itself,
    /// plus everything inside the splash radius around it. Enemies the
    /// firing tower cannot engage are never affected, not even by splash.
    fn for_each_affected(&self, enemies: &mut [Enemy], mut apply: impl FnMut(&mut Enemy)) {
        let impact_pos = enemies
            .iter()
            .find(|e| e.id == self.target_id)
            .map(|e| e.pos);
        let Some(center) = impact_pos else {
            return;
        };

        for enemy in enemies.iter_mut() {
            if !enemy.is_targetable_by(self.tower_kind) {
                continue;
            }
            let hit = if self.splash_radius <= 0.0 {
                enemy.id == self.target_id
            } else {
                (enemy.pos - center).length() <= self.splash_radius
            };
            if hit {
                apply(enemy);
            }
        }
    }

    pub fn draw(&self, texture: &Texture2D) {
        let size = 12.0;
        draw_texture_ex(
            texture,
            self.pos.x - size / 2.0,
            self.pos.y - size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    }
}
