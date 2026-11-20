//! Towers: player-placed structures that shoot at passing enemies.

use crate::enemy::{Enemy, EnemyAttribute};
use macroquad::prelude::*;
use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TowerType {
    Pebble,
    Laser,
    Pepper,
    Salt,
    CostDesTodes,
    Lava,
}

/// Speed multiplier applied to an enemy carrying [`EffectType::Slow`].
pub const SLOW_FACTOR: f32 = 0.75;
/// Damage per second dealt by [`EffectType::Poison`].
pub const POISON_DPS: f32 = 8.0;
/// Damage per second dealt by [`EffectType::Lava`].
pub const LAVA_DPS: f32 = 12.0;

/// A temporary debuff an enemy can carry. Effects are applied by
/// projectiles and expire on their own after [`EffectType::duration`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize, Hash)]
pub enum EffectType {
    Slow,
    Poison,
    Lava,
}

impl EffectType {
    /// Every effect, in a fixed order. Enemies store their effects in a
    /// `HashMap`, whose iteration order is not stable, so rendering and
    /// any other ordered processing iterates this instead.
    pub const ALL: [EffectType; 3] = [EffectType::Slow, EffectType::Poison, EffectType::Lava];

    /// How long the effect lasts, in seconds, when freshly applied.
    pub fn duration(self) -> f32 {
        match self {
            EffectType::Slow => 10.0,
            EffectType::Poison => 4.0,
            EffectType::Lava => 8.0,
        }
    }

    /// Base name of the effect's icon, used both for the asset path and
    /// for lookups in [`crate::sprites::Sprites`].
    pub fn icon_name(self) -> &'static str {
        match self {
            EffectType::Slow => "slow",
            EffectType::Poison => "poison",
            EffectType::Lava => "lava",
        }
    }
}

impl TowerType {
    pub fn cost(self) -> u32 {
        match self {
            TowerType::Pebble => 50,
            TowerType::Laser => 10,
            TowerType::Pepper => 75,
            TowerType::Salt => 150,
            TowerType::CostDesTodes => 300,
            TowerType::Lava => 100,
        }
    }

    pub fn range(self) -> f32 {
        match self {
            TowerType::Pebble => 120.0,
            TowerType::Laser => 120.0,
            TowerType::Pepper => 100.0,
            TowerType::Salt => 150.0,
            TowerType::CostDesTodes => 200.0,
            TowerType::Lava => 200.0,
        }
    }

    pub fn damage(self) -> f32 {
        match self {
            TowerType::Pebble => 12.0,
            TowerType::Laser => 100.0,
            TowerType::Pepper => 3.0,
            TowerType::Salt => 30.0,
            TowerType::CostDesTodes => 50.0,
            TowerType::Lava => 0.0,
        }
    }

    /// Shots per second.
    pub fn fire_rate(self) -> f32 {
        match self {
            TowerType::Pebble => 1.0,
            TowerType::Laser => 1.0,
            TowerType::Pepper => 5.0,
            TowerType::Salt => 0.6,
            TowerType::CostDesTodes => 0.5,
            TowerType::Lava => 0.2,
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
            TowerType::Laser => "Laser Tower",
            TowerType::Pepper => "Pepper Sprayer",
            TowerType::Salt => "Salt Cannon",
            TowerType::CostDesTodes => "Kost des Todes",
            TowerType::Lava => "Lava",
        }
    }

    /// Whether this tower hits instantly with a beam instead of firing a
    /// projectile (see [`crate::fx::LaserBeam`]).
    pub fn fires_beam(self) -> bool {
        matches!(self, TowerType::Laser)
    }

    /// Base name of this tower's projectile art, used for the asset path
    /// `assets/projectiles/{name}.png`.
    pub fn projectile_name(self) -> &'static str {
        match self {
            TowerType::Pebble => "pebble",
            TowerType::Laser => "laser",
            TowerType::Pepper => "pepper",
            TowerType::Salt => "salt",
            TowerType::CostDesTodes => "death",
            TowerType::Lava => "lava",
        }
    }

    pub fn projectiles_effect(self) -> Vec<EffectType> {
        match self {
            TowerType::Salt => vec![EffectType::Slow],
            TowerType::Pepper => vec![EffectType::Poison],
            TowerType::Lava => vec![EffectType::Lava],
            _ => vec![],
        }
    }

    /// Whether this tower is able to engage enemies carrying `attribute`.
    pub fn can_target(self, attribute: EnemyAttribute) -> bool {
        match attribute {
            EnemyAttribute::Flying => {
                matches!(
                    self,
                    TowerType::Pebble | TowerType::Laser | TowerType::CostDesTodes
                )
            }
        }
    }

    /// Whether this tower's damage goes straight through enemy armor (see
    /// [`crate::enemy::EnemyType::armor`]).
    pub fn pierces_armor(self) -> bool {
        matches!(self, TowerType::CostDesTodes)
    }
}

/// Highest level a tower can be upgraded to.
pub const MAX_LEVEL: u32 = 5;
/// Stat bonus gained per level above 1 (damage, fire rate, splash).
const LEVEL_BONUS: f32 = 0.25;

pub struct Tower {
    pub kind: TowerType,
    pub pos: Vec2,
    pub cooldown: f32,
    /// Upgrade level, from 1 up to [`MAX_LEVEL`].
    pub level: u32,
    /// Gold paid for this tower so far (purchase plus all upgrades).
    pub total_spent: u32,
}

impl Tower {
    pub fn new(kind: TowerType, pos: Vec2) -> Self {
        Tower {
            kind,
            pos,
            cooldown: 0.0,
            level: 1,
            total_spent: kind.cost(),
        }
    }

    fn level_multiplier(&self) -> f32 {
        1.0 + LEVEL_BONUS * (self.level - 1) as f32
    }

    pub fn damage(&self) -> f32 {
        self.kind.damage() * self.level_multiplier()
    }

    pub fn fire_rate(&self) -> f32 {
        self.kind.fire_rate() * self.level_multiplier()
    }

    pub fn splash_radius(&self) -> f32 {
        self.kind.splash_radius() * self.level_multiplier()
    }

    pub fn range(&self) -> f32 {
        self.kind.range()
    }

    /// Gold needed to reach the next level, or `None` at [`MAX_LEVEL`].
    pub fn upgrade_cost(&self) -> Option<u32> {
        (self.level < MAX_LEVEL).then(|| self.kind.cost() * self.level)
    }

    /// Raise the level by one, recording the cost. Returns the gold
    /// spent, or `None` if already at [`MAX_LEVEL`].
    pub fn upgrade(&mut self) -> Option<u32> {
        let cost = self.upgrade_cost()?;
        self.level += 1;
        self.total_spent += cost;
        Some(cost)
    }

    /// Gold refunded when selling: half of everything spent on it.
    pub fn sell_value(&self) -> u32 {
        self.total_spent / 2
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
        self.cooldown = 1.0 / self.fire_rate();
    }

    /// Find the id of the enemy furthest along the path within range, so
    /// towers prioritize the enemy closest to the base.
    pub fn find_target(&self, enemies: &[Enemy]) -> Option<u32> {
        let range = self.range();
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

        let label = self.level.to_string();
        let font_size = 16.0;
        let dims = measure_text(&label, None, font_size as u16, 1.0);
        let badge = vec2(self.pos.x + size / 2.0 - 6.0, self.pos.y + size / 2.0 - 6.0);
        draw_circle(badge.x, badge.y, 8.0, Color::new(0.1, 0.1, 0.12, 0.9));
        draw_circle_lines(badge.x, badge.y, 8.0, 1.0, GOLD);
        draw_text(
            &label,
            badge.x - dims.width / 2.0,
            badge.y + dims.height / 2.0,
            font_size,
            WHITE,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrading_scales_stats_and_costs_more_each_level() {
        let mut tower = Tower::new(TowerType::Salt, Vec2::ZERO);
        assert_eq!(tower.upgrade(), Some(150));
        assert_eq!(tower.upgrade(), Some(300));
        assert_eq!(tower.level, 3);
        assert_eq!(tower.damage(), 30.0 * 1.5);
        assert_eq!(tower.fire_rate(), 0.6 * 1.5);
        assert_eq!(tower.splash_radius(), 40.0 * 1.5);
        assert_eq!(tower.range(), TowerType::Salt.range());
    }

    #[test]
    fn upgrades_stop_at_max_level() {
        let mut tower = Tower::new(TowerType::Pebble, Vec2::ZERO);
        while tower.upgrade().is_some() {}
        assert_eq!(tower.level, MAX_LEVEL);
        assert_eq!(tower.upgrade_cost(), None);
    }

    #[test]
    fn sell_value_refunds_half_of_everything_spent() {
        let mut tower = Tower::new(TowerType::Pebble, Vec2::ZERO);
        tower.upgrade();
        tower.upgrade();
        // 50 + 50 + 100 spent.
        assert_eq!(tower.sell_value(), 100);
    }
}
