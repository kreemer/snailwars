//! Projectiles fired by towers, homing toward a specific enemy (tracked by
//! stable id, since the enemy vector is compacted each frame).
//!
//! Damage/effect application lives in [`Shot`] so instant-hit weapons
//! (the laser, see [`crate::fx::LaserBeam`]) share it with projectiles.

use std::collections::VecDeque;
use std::f32::consts::PI;

use crate::{
    enemy::Enemy,
    tower::{EffectType, TowerType},
};
use macroquad::prelude::*;

const PROJECTILE_SPEED: f32 = 420.0;
const HIT_DISTANCE: f32 = 10.0;
/// Number of past positions kept for the fading trail.
const TRAIL_LEN: usize = 8;
/// Seconds between trail samples.
const TRAIL_INTERVAL: f32 = 0.015;

/// Outcome of moving a projectile for one frame.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ProjectileStatus {
    /// Still on its way to the target.
    Flying,
    /// Reached the target, which stands at the given position.
    Hit(Vec2),
    /// The target died or escaped before impact; the projectile fizzles.
    Lost,
}

/// What a tower's attack does on impact: damage plus effects, to one
/// target or everything within a splash radius around it.
#[derive(Clone, Debug)]
pub struct Shot {
    pub target_id: u32,
    pub damage: f32,
    pub splash_radius: f32,
    pub effects: Vec<EffectType>,
    /// Type of the tower that fired this shot; decides which enemies it
    /// may damage (see [`Enemy::is_targetable_by`]).
    pub tower_kind: TowerType,
}

impl Shot {
    /// The attack a tower of `kind` makes against `target_id`.
    pub fn from_tower(kind: TowerType, target_id: u32) -> Self {
        Shot {
            target_id,
            damage: kind.damage(),
            splash_radius: kind.splash_radius(),
            effects: kind.projectiles_effect(),
            tower_kind: kind,
        }
    }

    /// Apply damage, then effects, to the target and (with a splash
    /// radius) to nearby enemies. Armor is applied per enemy unless the
    /// firing tower pierces it.
    pub fn apply(&self, enemies: &mut [Enemy]) {
        let damage = self.damage;
        let ignores_armor = self.tower_kind.pierces_armor();
        self.for_each_affected(enemies, |enemy| enemy.take_damage(damage, ignores_armor));
        if !self.effects.is_empty() {
            let effects = &self.effects;
            self.for_each_affected(enemies, |enemy| enemy.apply_effects(effects.clone()));
        }
    }

    /// Run `apply` on every enemy this shot hits: the target itself,
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
}

/// How a tower's projectile looks in flight.
#[derive(Clone, Copy, Debug)]
pub struct ProjectileLook {
    /// Drawn width/height in world pixels.
    pub size: f32,
    /// Rotation speed in radians per second.
    pub spin: f32,
    /// Peak height of a lobbed arc; 0.0 flies straight.
    pub arc_height: f32,
    pub trail_color: Color,
    /// Trail dot size relative to `size`.
    pub trail_scale: f32,
    pub impact_color: Color,
}

impl ProjectileLook {
    pub fn for_tower(kind: TowerType) -> Self {
        match kind {
            TowerType::Pebble => ProjectileLook {
                size: 14.0,
                spin: 10.0,
                arc_height: 0.0,
                trail_color: Color::new(0.75, 0.75, 0.75, 0.5),
                trail_scale: 0.4,
                impact_color: Color::new(0.8, 0.8, 0.8, 0.8),
            },
            TowerType::Pepper => ProjectileLook {
                size: 9.0,
                spin: 0.0,
                arc_height: 0.0,
                trail_color: Color::new(1.0, 0.25, 0.1, 0.6),
                trail_scale: 0.6,
                impact_color: Color::new(0.5, 0.9, 0.2, 0.8),
            },
            TowerType::Salt => ProjectileLook {
                size: 18.0,
                spin: 4.0,
                arc_height: 0.0,
                trail_color: Color::new(0.85, 0.95, 1.0, 0.6),
                trail_scale: 0.4,
                impact_color: Color::new(0.85, 0.95, 1.0, 0.7),
            },
            TowerType::CostDesTodes => ProjectileLook {
                size: 22.0,
                spin: 0.0,
                arc_height: 0.0,
                trail_color: Color::new(0.2, 0.9, 0.2, 0.7),
                trail_scale: 0.7,
                impact_color: Color::new(0.2, 1.0, 0.2, 0.8),
            },
            TowerType::Lava => ProjectileLook {
                size: 18.0,
                spin: 0.0,
                arc_height: 60.0,
                trail_color: Color::new(1.0, 0.45, 0.0, 0.7),
                trail_scale: 0.6,
                impact_color: Color::new(1.0, 0.5, 0.0, 0.8),
            },
            TowerType::Laser => ProjectileLook {
                size: 8.0,
                spin: 0.0,
                arc_height: 0.0,
                trail_color: Color::new(0.8, 0.3, 1.0, 0.6),
                trail_scale: 0.5,
                impact_color: Color::new(0.9, 0.5, 1.0, 0.9),
            },
        }
    }
}

pub struct Projectile {
    /// Ground position used for homing and hit detection.
    pub pos: Vec2,
    pub shot: Shot,
    look: ProjectileLook,
    /// Distance to the target at the first update; drives the lob arc.
    initial_dist: Option<f32>,
    /// Current on-screen height above `pos`; see [`Self::arc_lift`].
    lift: f32,
    rotation: f32,
    trail: VecDeque<Vec2>,
    trail_timer: f32,
}

impl Projectile {
    pub fn new(pos: Vec2, shot: Shot) -> Self {
        Projectile {
            pos,
            look: ProjectileLook::for_tower(shot.tower_kind),
            shot,
            initial_dist: None,
            lift: 0.0,
            rotation: 0.0,
            trail: VecDeque::with_capacity(TRAIL_LEN),
            trail_timer: 0.0,
        }
    }

    pub fn look(&self) -> ProjectileLook {
        self.look
    }

    /// Move toward the target enemy. Anything other than
    /// [`ProjectileStatus::Flying`] means the projectile should be removed.
    pub fn update(&mut self, dt: f32, enemies: &[Enemy]) -> ProjectileStatus {
        let Some(target) = enemies.iter().find(|e| e.id == self.shot.target_id) else {
            return ProjectileStatus::Lost;
        };
        if target.is_dead() || target.reached_base {
            return ProjectileStatus::Lost;
        }

        let to_target = target.pos - self.pos;
        let dist = to_target.length();
        let initial = *self.initial_dist.get_or_insert(dist);
        if dist <= HIT_DISTANCE {
            return ProjectileStatus::Hit(target.pos);
        }

        self.rotation += self.look.spin * dt;
        self.trail_timer -= dt;
        if self.trail_timer <= 0.0 {
            self.trail_timer = TRAIL_INTERVAL;
            if self.trail.len() == TRAIL_LEN {
                self.trail.pop_back();
            }
            self.trail.push_front(self.draw_pos());
        }

        let step = PROJECTILE_SPEED * dt;
        if step >= dist {
            self.pos = target.pos;
        } else {
            self.pos += to_target / dist * step;
        }
        self.lift = self.arc_lift((target.pos - self.pos).length(), initial);
        ProjectileStatus::Flying
    }

    /// Height above the ground for lobbed projectiles: a parabola over
    /// the flight, peaking halfway. Zero for straight shots.
    fn arc_lift(&self, dist: f32, initial: f32) -> f32 {
        if self.look.arc_height <= 0.0 || initial <= 0.0 {
            return 0.0;
        }
        let progress = (1.0 - dist / initial).clamp(0.0, 1.0);
        let height = self.look.arc_height.min(initial * 0.4);
        (progress * PI).sin() * height
    }

    /// Where the projectile appears on screen.
    fn draw_pos(&self) -> Vec2 {
        self.pos - vec2(0.0, self.lift)
    }

    pub fn draw(&self, texture: &Texture2D) {
        let look = self.look;
        let n = self.trail.len() as f32;
        for (i, point) in self.trail.iter().enumerate() {
            let fade = 1.0 - (i as f32 + 1.0) / (n + 1.0);
            let mut color = look.trail_color;
            color.a *= fade;
            let radius = look.size * look.trail_scale * 0.5 * fade.max(0.3);
            draw_circle(point.x, point.y, radius, color);
        }

        let pos = self.draw_pos();
        let size = look.size;
        draw_texture_ex(
            texture,
            pos.x - size / 2.0,
            pos.y - size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                rotation: self.rotation,
                ..Default::default()
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemy::EnemyType;

    fn lava_shot(target_id: u32) -> Projectile {
        Projectile::new(
            Vec2::new(0.0, 0.0),
            Shot {
                target_id,
                damage: 0.0,
                splash_radius: 0.0,
                effects: vec![EffectType::Lava],
                tower_kind: TowerType::Lava,
            },
        )
    }

    #[test]
    fn hitting_the_target_reports_its_position() {
        let target = Enemy::new(7, EnemyType::Snail, Vec2::new(5.0, 0.0), 1);
        let mut shot = lava_shot(7);
        assert_eq!(
            shot.update(0.016, &[target]),
            ProjectileStatus::Hit(Vec2::new(5.0, 0.0))
        );
    }

    #[test]
    fn losing_the_target_is_not_a_hit() {
        let mut shot = lava_shot(7);
        assert_eq!(shot.update(0.016, &[]), ProjectileStatus::Lost);
    }

    #[test]
    fn lobbed_projectile_rises_mid_flight_but_homes_on_the_ground() {
        let target = Enemy::new(7, EnemyType::Snail, Vec2::new(300.0, 0.0), 1);
        let mut shot = lava_shot(7);
        for _ in 0..20 {
            shot.update(0.016, std::slice::from_ref(&target));
        }
        assert_eq!(shot.pos.y, 0.0);
        assert!(shot.draw_pos().y < -10.0);
    }
}
