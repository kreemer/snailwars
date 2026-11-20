//! Core game state: gold, lives, wave progression, and the update/draw
//! loop that ties enemies, towers, and projectiles together.

use crate::enemy::{Enemy, EnemyType};
use crate::fx::{Impact, LaserBeam};
use crate::level::Level;
use crate::map::Map;
use crate::projectile::{Projectile, ProjectileLook, ProjectileStatus, Shot};
use crate::sprites::Sprites;
use crate::tower::{EffectType, Tower, TowerType};
use crate::ui;
use crate::wave::Wave;
use macroquad::prelude::*;
use std::collections::{HashMap, VecDeque};

const STARTING_GOLD: u32 = 150;
const STARTING_LIVES: u32 = 20;
const BUILD_CLICK_RADIUS: f32 = 30.0;
const SPEED_LEVELS: [u32; 3] = [1, 2, 3];

#[derive(PartialEq, Eq, Debug)]
pub enum GameStatus {
    Playing,
    GameOver,
    Win,
}

struct PendingSpawn {
    kind: EnemyType,
    delay: f32,
    level: u16,
}

pub struct Game {
    map: Map,
    sprites: Sprites,
    /// One loaded texture per entry in [`Level::tilesets`], in the same
    /// order; see [`crate::map::Map::draw`].
    tileset_textures: Vec<Texture2D>,
    waves: Vec<Wave>,
    level_id: String,

    enemies: Vec<Enemy>,
    towers: HashMap<usize, Tower>,
    projectiles: Vec<Projectile>,
    beams: Vec<LaserBeam>,
    impacts: Vec<Impact>,
    occupied: Vec<bool>,

    gold: u32,
    lives: u32,
    wave_number: u32,
    wave_active: bool,
    spawn_queue: VecDeque<PendingSpawn>,
    spawn_timer: f32,

    next_enemy_id: u32,
    selected_tower: Option<TowerType>,
    /// Build-spot index of the placed tower whose Sell/Upgrade menu is
    /// open, if any.
    open_menu: Option<usize>,
    status: GameStatus,
    speed_index: usize,
}

impl Game {
    /// Build a fresh game from an already-loaded [`Level`], its tileset
    /// textures (one per [`Level::tilesets`] entry, in order), and its
    /// enemy/UI sprites (texture loading is async in macroquad and must
    /// happen before this constructor runs; see `main.rs`).
    pub fn load(
        level_id: &str,
        level: Level,
        tileset_textures: Vec<Texture2D>,
        sprites: Sprites,
    ) -> Self {
        let map = Map::from_level(&level);
        let occupied = vec![false; map.build_spots.len()];
        Game {
            map,
            sprites,
            tileset_textures,
            waves: level.waves,
            level_id: level_id.to_owned(),
            enemies: Vec::new(),
            towers: HashMap::new(),
            projectiles: Vec::new(),
            beams: Vec::new(),
            impacts: Vec::new(),
            occupied,
            gold: STARTING_GOLD,
            lives: STARTING_LIVES,
            wave_number: 0,
            wave_active: false,
            spawn_queue: VecDeque::new(),
            spawn_timer: 0.0,
            next_enemy_id: 0,
            selected_tower: None,
            open_menu: None,
            status: GameStatus::Playing,
            speed_index: 0,
        }
    }

    /// Full map size in world pixels; see [`crate::map::Map::world_size`].
    /// Used by [`crate::viewport::Viewport`] to clamp camera scrolling.
    pub fn world_size(&self) -> Vec2 {
        self.map.world_size()
    }

    /// Id of the level being played, matching its
    /// [`crate::catalog::LevelEntry::id`].
    pub fn level_id(&self) -> &str {
        &self.level_id
    }

    /// Whether the level is still running, was lost, or was beaten. The
    /// app layer (see [`crate::app::App`]) drives what happens next.
    pub fn status(&self) -> &GameStatus {
        &self.status
    }

    /// Throw away all progress within the level and start it over from
    /// the beginning, reusing the already-loaded textures.
    pub fn restart(&mut self) {
        let level = Level::load(&self.level_id);
        *self = Game::load(
            &self.level_id,
            level,
            self.tileset_textures.clone(),
            self.sprites.clone(),
        );
    }

    /// `ui_mouse` and `world_mouse` are the mouse position converted to
    /// logical UI coordinates (fixed, ignores zoom) and logical world
    /// coordinates (follows zoom) respectively - see
    /// [`crate::viewport::Viewport::to_ui_logical`] and
    /// [`crate::viewport::Viewport::to_world_logical`]. This module
    /// never needs to know about the real window size/zoom itself.
    pub fn update(&mut self, real_dt: f32, ui_mouse: Vec2, world_mouse: Vec2) {
        if self.status != GameStatus::Playing {
            return;
        }

        self.handle_input(ui_mouse, world_mouse);
        if is_key_pressed(KeyCode::Tab) {
            self.speed_index = (self.speed_index + 1) % SPEED_LEVELS.len();
        }
        let dt = real_dt * SPEED_LEVELS[self.speed_index] as f32;
        self.update_spawning(dt);
        self.map.update_lava_pools(dt);

        for enemy in &mut self.enemies {
            if enemy.is_targetable_by(TowerType::Lava)
                && self
                    .map
                    .lava_pools
                    .iter()
                    .any(|pool| pool.contains(enemy.pos))
            {
                enemy.apply_effects(vec![EffectType::Lava]);
            }
            enemy.update(dt, &self.map.waypoints);
        }

        for (_, tower) in &mut self.towers.iter_mut() {
            tower.update_cooldown(dt);
            if tower.can_fire() {
                if let Some(target_id) = tower.find_target(&self.enemies) {
                    let shot = Shot::from_placed(tower, target_id);
                    if tower.kind.fires_beam() {
                        fire_beam(
                            tower.pos,
                            &shot,
                            &mut self.enemies,
                            &mut self.beams,
                            &mut self.impacts,
                        );
                    } else {
                        self.projectiles.push(Projectile::new(tower.pos, shot));
                    }
                    tower.fire();
                }
            }
        }

        let mut i = 0;
        while i < self.projectiles.len() {
            match self.projectiles[i].update(dt, &self.enemies) {
                ProjectileStatus::Flying => {
                    i += 1;
                    continue;
                }
                ProjectileStatus::Hit(impact_pos) => {
                    let projectile = &self.projectiles[i];
                    projectile.shot.apply(&mut self.enemies);
                    if projectile.shot.effects.contains(&EffectType::Lava) {
                        self.map.add_lava_pool(impact_pos);
                    }
                    self.impacts
                        .push(impact_for(impact_pos, projectile.look(), &projectile.shot));
                }
                ProjectileStatus::Lost => {}
            }
            self.projectiles.remove(i);
        }

        for beam in &mut self.beams {
            beam.update(dt);
        }
        self.beams.retain(|beam| !beam.is_expired());
        for impact in &mut self.impacts {
            impact.update(dt);
        }
        self.impacts.retain(|impact| !impact.is_expired());

        let mut gold_earned = 0u32;
        let mut lives_lost = 0u32;
        self.enemies.retain(|enemy| {
            if enemy.is_dead() {
                gold_earned += enemy.reward;
                false
            } else if enemy.reached_base {
                lives_lost += 1;
                false
            } else {
                true
            }
        });
        self.gold += gold_earned;
        self.lives = self.lives.saturating_sub(lives_lost);

        if self.lives == 0 {
            self.status = GameStatus::GameOver;
            return;
        }

        if self.wave_active && self.spawn_queue.is_empty() && self.enemies.is_empty() {
            self.map.lava_pools.clear();
            self.wave_active = false;
            if self.wave_number >= self.waves.len() as u32 {
                self.status = GameStatus::Win;
            }
        }
    }

    fn update_spawning(&mut self, dt: f32) {
        if self.spawn_queue.is_empty() {
            return;
        }
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            if let Some(entry) = self.spawn_queue.pop_front() {
                let start = self.map.waypoints[0];
                self.enemies.push(Enemy::new(
                    self.next_enemy_id,
                    entry.kind,
                    start,
                    entry.level,
                ));
                self.next_enemy_id += 1;
            }
            if let Some(next) = self.spawn_queue.front() {
                self.spawn_timer = next.delay;
            }
        }
    }

    fn handle_input(&mut self, ui_mouse: Vec2, world_mouse: Vec2) {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        // Tower selection buttons (fixed UI, ignores zoom).
        for (kind, rect) in ui::tower_button_rects() {
            if rect.contains(ui_mouse) {
                self.selected_tower = if self.selected_tower == Some(kind) {
                    None
                } else {
                    Some(kind)
                };
                self.open_menu = None;
                return;
            }
        }

        // Start wave button.
        if ui::start_wave_button_rect().contains(ui_mouse) && !self.wave_active {
            self.start_next_wave();
            return;
        }

        // Speed toggle button.
        if ui::speed_button_rect().contains(ui_mouse) {
            self.speed_index = (self.speed_index + 1) % SPEED_LEVELS.len();
            return;
        }

        // Everything below happens on the map (follows the zoomed/scrolled
        // world). Whether the cursor is over the play area (vs. the
        // HUD/panel bars) is a fixed on-screen question, so it must be
        // checked against `ui_mouse`, not `world_mouse` - the latter is in
        // absolute map coordinates, which can well exceed `PLAY_H` once
        // the camera has scrolled or zoomed out.
        if ui_mouse.y < crate::map::TOP_BAR
            || ui_mouse.y > crate::map::TOP_BAR + crate::map::PLAY_H
        {
            return;
        }

        if self.handle_tower_menu_click(world_mouse) {
            return;
        }

        // Clicking a placed tower opens its menu.
        if let Some(spot_index) =
            self.map
                .nearest_occupied_spot(world_mouse, &self.occupied, BUILD_CLICK_RADIUS)
        {
            self.open_menu = Some(spot_index);
            self.selected_tower = None;
            return;
        }

        // Placing a tower.
        if let Some(kind) = self.selected_tower {
            if self.gold < kind.cost() {
                return;
            }
            if let Some(spot_index) =
                self.map
                    .nearest_free_spot(world_mouse, &self.occupied, BUILD_CLICK_RADIUS)
            {
                self.gold -= kind.cost();
                self.occupied[spot_index] = true;
                self.towers.insert(
                    spot_index,
                    Tower::new(kind, self.map.build_spots[spot_index]),
                );
            }
        }
    }

    /// Handle a left click while a tower menu is open. Returns `true` if
    /// the click was consumed by the menu. Clicking anywhere outside the
    /// menu closes it (and lets the click fall through).
    fn handle_tower_menu_click(&mut self, world_mouse: Vec2) -> bool {
        let Some(spot_index) = self.open_menu else {
            return false;
        };
        let Some(tower) = self.towers.get_mut(&spot_index) else {
            self.open_menu = None;
            return false;
        };
        let layout = ui::tower_menu_layout(tower.pos);

        if layout.upgrade.contains(world_mouse) {
            if let Some(cost) = tower.upgrade_cost() {
                if self.gold >= cost {
                    tower.upgrade();
                    self.gold -= cost;
                }
            }
            return true;
        }
        if layout.sell.contains(world_mouse) {
            self.gold += tower.sell_value();
            self.occupied[spot_index] = false;
            self.towers.remove(&spot_index);
            self.open_menu = None;
            return true;
        }
        if layout.panel.contains(world_mouse) {
            return true;
        }
        self.open_menu = None;
        false
    }

    fn start_next_wave(&mut self) {
        self.wave_number += 1;
        let wave = &self.waves[self.wave_number as usize - 1];
        self.spawn_timer = 0.0;
        self.spawn_queue = wave
            .spawns
            .iter()
            .map(|s| PendingSpawn {
                kind: s.kind,
                delay: s.delay_after_previous,
                level: s.level,
            })
            .collect();
        self.wave_active = true;
    }

    /// Draw the zoomable world layer: map, towers, enemies, projectiles,
    /// and the tower-placement range preview. `ui_mouse`/`world_mouse`
    /// are the mouse position in logical UI/world coordinates; see
    /// [`Self::update`].
    pub fn draw_world(&self, ui_mouse: Vec2, world_mouse: Vec2) {
        clear_background(Color::new(0.05, 0.05, 0.05, 1.0));
        self.map.draw(&self.tileset_textures);
        self.map
            .draw_build_spots(&self.occupied, &self.sprites.build_spot);
        self.map.draw_lava_pools(&self.sprites.lava_pool);

        for tower in self.towers.values() {
            let tex = match tower.kind {
                TowerType::Pebble => &self.sprites.tower_pebble,
                TowerType::Laser => &self.sprites.tower_laser,
                TowerType::Pepper => &self.sprites.tower_pepper,
                TowerType::Salt => &self.sprites.tower_salt,
                TowerType::CostDesTodes => &self.sprites.tower_death,
                TowerType::Lava => &self.sprites.tower_lava,
            };
            tower.draw(tex);
        }

        for enemy in &self.enemies {
            let tex = match enemy.kind {
                EnemyType::Snail => &self.sprites.snail,
                EnemyType::Slug => &self.sprites.slug,
                EnemyType::BigSnail => &self.sprites.big_snail,
                EnemyType::FlyingSnail => &self.sprites.flying_snail,
                EnemyType::ArmoredSnail => &self.sprites.armored_snail,
                EnemyType::ArmoredSlug => &self.sprites.armored_slug,
                EnemyType::ArmoredBigSnail => &self.sprites.armored_big_snail,
            };
            enemy.draw(tex, &self.sprites);
        }

        for beam in &self.beams {
            beam.draw();
        }
        for projectile in &self.projectiles {
            if let Some(tex) = self.sprites.projectile_texture(projectile.shot.tower_kind) {
                projectile.draw(tex);
            }
        }
        for impact in &self.impacts {
            impact.draw();
        }

        // Range preview for the currently selected tower type, following
        // the mouse. Whether the cursor is over the play area is a
        // fixed on-screen question - check `ui_mouse`, not `world_mouse`
        // (see the matching check in `handle_input`).
        if let Some(kind) = self.selected_tower {
            if ui_mouse.y >= crate::map::TOP_BAR
                && ui_mouse.y <= crate::map::TOP_BAR + crate::map::PLAY_H
            {
                draw_circle_lines(
                    world_mouse.x,
                    world_mouse.y,
                    kind.range(),
                    1.5,
                    Color::new(1.0, 1.0, 1.0, 0.5),
                );
            }
        }

        if let Some(tower) = self.open_menu.and_then(|i| self.towers.get(&i)) {
            draw_circle_lines(
                tower.pos.x,
                tower.pos.y,
                tower.range(),
                1.5,
                Color::new(1.0, 1.0, 1.0, 0.5),
            );
            ui::draw_tower_menu(tower, self.gold, world_mouse);
        }
    }

    /// Draw the fixed (non-zoomable) UI layer: HUD and tower panel. Must
    /// be drawn with a transparent background so the world layer shows
    /// through beneath it; see
    /// [`crate::viewport::Viewport::begin_ui`]. End-of-level overlays
    /// are drawn on top by the app layer (see [`crate::app::App`]).
    pub fn draw_ui(&self) {
        ui::draw_hud(
            self.gold,
            self.lives,
            self.wave_number,
            self.waves.len() as u32,
        );
        ui::draw_panel(
            self.gold,
            self.selected_tower,
            &self.sprites,
            self.wave_active,
            SPEED_LEVELS[self.speed_index],
        );
    }
}

/// Instantly apply a beam tower's `shot` and leave a fading laser beam
/// from `from` to where the target stood.
fn fire_beam(
    from: Vec2,
    shot: &Shot,
    enemies: &mut [Enemy],
    beams: &mut Vec<LaserBeam>,
    impacts: &mut Vec<Impact>,
) {
    let Some(target_pos) = enemies
        .iter()
        .find(|e| e.id == shot.target_id)
        .map(|e| e.pos)
    else {
        return;
    };
    shot.apply(enemies);
    beams.push(LaserBeam::new(from, target_pos));
    impacts.push(impact_for(
        target_pos,
        ProjectileLook::for_tower(shot.tower_kind),
        shot,
    ));
}

/// Impact puff for a hit: as large as the splash radius, or a small
/// burst for single-target shots.
fn impact_for(pos: Vec2, look: ProjectileLook, shot: &Shot) -> Impact {
    let radius = if shot.splash_radius > 0.0 {
        shot.splash_radius
    } else {
        look.size.max(12.0)
    };
    Impact::new(pos, look.impact_color, radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn laser_hits_instantly_and_leaves_a_beam() {
        let mut enemies = vec![Enemy::new(3, EnemyType::Snail, Vec2::new(50.0, 0.0), 1)];
        let hp_before = enemies[0].hp;
        let (mut beams, mut impacts) = (Vec::new(), Vec::new());
        let shot = Shot::from_tower(TowerType::Laser, 3);

        fire_beam(Vec2::ZERO, &shot, &mut enemies, &mut beams, &mut impacts);

        assert!(enemies[0].hp < hp_before);
        assert_eq!(beams.len(), 1);
        assert_eq!(beams[0].to, Vec2::new(50.0, 0.0));
    }
}
