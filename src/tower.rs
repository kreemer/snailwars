//! Towers: player-placed structures that shoot at passing enemies.

use crate::enemy::{Enemy, EnemyAttribute};
use macroquad::prelude::*;
use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TowerType {
    Pebble,
    Pepper,
    Salt,
    CostDesTodes,
}

/// Speed multiplier applied to an enemy carrying [`EffectType::Slow`].
pub const SLOW_FACTOR: f32 = 0.75;
/// Damage per second dealt by [`EffectType::Poison`].
pub const POISON_DPS: f32 = 8.0;

/// A temporary debuff an enemy can carry. Effects are applied by
/// projectiles and expire on their own after [`EffectType::duration`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize, Hash)]
pub enum EffectType {
    Slow,
    Poison,
}

impl EffectType {
    /// Every effect, in a fixed order. Enemies store their effects in a
    /// `HashMap`, whose iteration order is not stable, so rendering and
    /// any other ordered processing iterates this instead.
    pub const ALL: [EffectType; 2] = [EffectType::Slow, EffectType::Poison];

    /// How long the effect lasts, in seconds, when freshly applied.
    pub fn duration(self) -> f32 {
        match self {
            EffectType::Slow => 10.0,
            EffectType::Poison => 4.0,
        }
    }

    /// Base name of the effect's icon, used both for the asset path and
    /// for lookups in [`crate::sprites::Sprites`].
    pub fn icon_name(self) -> &'static str {
        match self {
            EffectType::Slow => "slow",
            EffectType::Poison => "poison",
        }
    }
}

impl TowerType {
    pub fn cost(self) -> u32 {
        match self {
            TowerType::Pebble => 50,
            TowerType::Pepper => 75,
            TowerType::Salt => 150,
            TowerType::CostDesTodes => 300,
        }
    }

    pub fn range(self) -> f32 {
        match self {
            TowerType::Pebble => 120.0,
            TowerType::Pepper => 100.0,
            TowerType::Salt => 150.0,
            TowerType::CostDesTodes => 200.0,
        }
    }

    pub fn damage(self) -> f32 {
        match self {
            TowerType::Pebble => 12.0,
            TowerType::Pepper => 3.0,
            TowerType::Salt => 30.0,
            TowerType::CostDesTodes => 50.0,
        }
    }

    /// Shots per second.
    pub fn fire_rate(self) -> f32 {
        match self {
            TowerType::Pebble => 1.0,
            TowerType::Pepper => 5.0,
            TowerType::Salt => 0.6,
            TowerType::CostDesTodes => 0.5,
        }
    }

    /// Splash radius; 0.0 means single-target only.
    pub fn splash_radius(self) -> f32 {
        match self {
            TowerType::Salt => 40.0,
            TowerType::CostDesTodes => 50.0,
            _ => 0.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TowerType::Pebble => "Pebble Turret",
            TowerType::Pepper => "Pepper Sprayer",
            TowerType::Salt => "Salt Cannon",
            TowerType::CostDesTodes => "Kost des Todes",
        }
    }

    pub fn projectiles_effect(self) -> Vec<EffectType> {
        match self {
            TowerType::Salt => vec![EffectType::Slow],
            TowerType::Pepper => vec![EffectType::Poison],
            _ => vec![],
        }
    }

    /// Whether this tower is able to engage enemies carrying `attribute`.
    pub fn can_target(self, attribute: EnemyAttribute) -> bool {
        match attribute {
            EnemyAttribute::Flying => {
                matches!(self, TowerType::Pebble | TowerType::CostDesTodes)
            }
        }
    }

    /// Whether this tower's damage goes straight through enemy armor (see
    /// [`crate::enemy::EnemyType::armor`]).
    pub fn pierces_armor(self) -> bool {
        matches!(self, TowerType::CostDesTodes)
    }
}

pub struct Tower {
    pub kind: TowerType,
    pub pos: Vec2,
    pub cooldown: f32,
}

impl Tower {
    pub fn new(kind: TowerType, pos: Vec2) -> Self {
        Tower {
            kind,
            pos,
            cooldown: 0.0,
        }
    }

    pub fn update_cooldown(&mut self, dt: f32) {
        if self.cooldown > 0.0 {
            self.cooldown -= dt;
        }
    }

    pub fn can_fire(&self) -> bool {
        self.cooldown <= 0.0
    }

    pub fn fire(&mut self) {
        self.cooldown = 1.0 / self.kind.fire_rate();
    }

    /// Find the id of the enemy furthest along the path within range, so
    /// towers prioritize the enemy closest to the base.
    pub fn find_target(&self, enemies: &[Enemy]) -> Option<u32> {
        let range = self.kind.range();
        let mut best: Option<(u32, usize)> = None; // (id, waypoint_index)
        for enemy in enemies.iter() {
            if enemy.is_dead() || enemy.reached_base {
                continue;
            }
            if !enemy.is_targetable_by(self.kind) {
                continue;
            }
            let dist = (enemy.pos - self.pos).length();
            if dist <= range {
                let better = match &best {
                    None => true,
                    Some((_, wp)) => enemy.waypoint_index > *wp,
                };
                if better {
                    best = Some((enemy.id, enemy.waypoint_index));
                }
            }
        }
        best.map(|(id, _)| id)
    }

    pub fn draw(&self, texture: &Texture2D) {
        let size = 40.0;
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
