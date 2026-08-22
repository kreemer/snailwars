//! Core game state: gold, lives, wave progression, and the update/draw
//! loop that ties enemies, towers, and projectiles together.

use crate::enemy::{Enemy, EnemyType};
use crate::map::Map;
use crate::projectile::Projectile;
use crate::sprites::Sprites;
use crate::tower::{Tower, TowerType};
use crate::ui;
use crate::wave::{self, TOTAL_WAVES};
use macroquad::prelude::*;
use std::collections::VecDeque;

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
}

pub struct Game {
    map: Map,
    sprites: Sprites,

    enemies: Vec<Enemy>,
    towers: Vec<Tower>,
    projectiles: Vec<Projectile>,
    occupied: Vec<bool>,

    gold: u32,
    lives: u32,
    wave_number: u32,
    wave_active: bool,
    spawn_queue: VecDeque<PendingSpawn>,
    spawn_timer: f32,

    next_enemy_id: u32,
    selected_tower: Option<TowerType>,
    status: GameStatus,
    speed_index: usize,
}

impl Game {
    pub fn new() -> Self {
        let map = Map::new();
        let occupied = vec![false; map.build_spots.len()];
        Game {
            map,
            sprites: Sprites::generate(),
            enemies: Vec::new(),
            towers: Vec::new(),
            projectiles: Vec::new(),
            occupied,
            gold: STARTING_GOLD,
            lives: STARTING_LIVES,
            wave_number: 0,
            wave_active: false,
            spawn_queue: VecDeque::new(),
            spawn_timer: 0.0,
            next_enemy_id: 0,
            selected_tower: None,
            status: GameStatus::Playing,
            speed_index: 0,
        }
    }

    pub fn update(&mut self, real_dt: f32) {
        if self.status != GameStatus::Playing {
            self.handle_restart_input();
            return;
        }

        self.handle_input();
        if is_key_pressed(KeyCode::Tab) {
            self.speed_index = (self.speed_index + 1) % SPEED_LEVELS.len();
        }
        let dt = real_dt * SPEED_LEVELS[self.speed_index] as f32;
        self.update_spawning(dt);

        for enemy in &mut self.enemies {
            enemy.update(dt, &self.map.waypoints);
        }

        for tower in &mut self.towers {
            tower.update_cooldown(dt);
            if tower.can_fire()
                && let Some(target_id) = tower.find_target(&self.enemies)
            {
                self.projectiles.push(Projectile::new(
                    tower.pos,
                    target_id,
                    tower.kind.damage(),
                    tower.kind.splash_radius(),
                    tower.kind.applies_debuf(),
                ));
                tower.fire();
            }
        }

        let mut i = 0;
        while i < self.projectiles.len() {
            let hit = self.projectiles[i].update(dt, &self.enemies);
            if hit {
                self.projectiles[i].apply_damage(&mut self.enemies);
                self.projectiles.remove(i);
            } else {
                i += 1;
            }
        }

        let mut gold_earned = 0u32;
        let mut lives_lost = 0u32;
        self.enemies.retain(|enemy| {
            if enemy.is_dead() {
                gold_earned += enemy.kind.reward();
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
            self.wave_active = false;
            if self.wave_number >= TOTAL_WAVES {
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
                self.enemies
                    .push(Enemy::new(self.next_enemy_id, entry.kind, start));
                self.next_enemy_id += 1;
            }
            if let Some(next) = self.spawn_queue.front() {
                self.spawn_timer = next.delay;
            }
        }
    }

    fn handle_input(&mut self) {
        if !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }
        let mouse = Vec2::from(mouse_position());

        // Tower selection buttons.
        for (kind, rect) in ui::tower_button_rects() {
            if rect.contains(mouse) {
                self.selected_tower = if self.selected_tower == Some(kind) {
                    None
                } else {
                    Some(kind)
                };
                return;
            }
        }

        // Start wave button.
        if ui::start_wave_button_rect().contains(mouse) && !self.wave_active {
            self.start_next_wave();
            return;
        }

        // Speed toggle button.
        if ui::speed_button_rect().contains(mouse) {
            self.speed_index = (self.speed_index + 1) % SPEED_LEVELS.len();
            return;
        }

        // Placing a tower on the map.
        if let Some(kind) = self.selected_tower {
            if mouse.y < crate::map::TOP_BAR || mouse.y > crate::map::TOP_BAR + crate::map::PLAY_H {
                return;
            }
            if self.gold < kind.cost() {
                return;
            }
            if let Some(spot_index) =
                self.map
                    .nearest_free_spot(mouse, &self.occupied, BUILD_CLICK_RADIUS)
            {
                self.gold -= kind.cost();
                self.occupied[spot_index] = true;
                self.towers
                    .push(Tower::new(kind, self.map.build_spots[spot_index]));
            }
        }
    }

    fn handle_restart_input(&mut self) {
        if is_key_pressed(KeyCode::R) {
            *self = Game::new();
        }
    }

    fn start_next_wave(&mut self) {
        self.wave_number += 1;
        let wave = wave::build_wave(self.wave_number);
        self.spawn_timer = 0.0;
        self.spawn_queue = wave
            .spawns
            .into_iter()
            .map(|s| PendingSpawn {
                kind: s.kind,
                delay: s.delay_after_previous,
            })
            .collect();
        self.wave_active = true;
    }

    pub fn draw(&self) {
        clear_background(Color::new(0.05, 0.05, 0.05, 1.0));
        self.map.draw(&self.sprites.grass_tile);
        self.map
            .draw_build_spots(&self.occupied, &self.sprites.build_spot);

        for tower in &self.towers {
            let tex = match tower.kind {
                TowerType::Pebble => &self.sprites.tower_pebble,
                TowerType::Pepper => &self.sprites.tower_pepper,
                TowerType::Salt => &self.sprites.tower_salt,
                TowerType::CostDesTodes => &self.sprites.tower_death,
            };
            tower.draw(tex);
        }

        for enemy in &self.enemies {
            let tex = match enemy.kind {
                EnemyType::Snail => &self.sprites.snail,
                EnemyType::Slug => &self.sprites.slug,
                EnemyType::BigSnail => &self.sprites.big_snail,
            };
            enemy.draw(tex);
        }

        for projectile in &self.projectiles {
            projectile.draw(&self.sprites.projectile);
        }

        // Range preview for the currently selected tower type, following the mouse.
        if let Some(kind) = self.selected_tower {
            let mouse = Vec2::from(mouse_position());
            if mouse.y >= crate::map::TOP_BAR && mouse.y <= crate::map::TOP_BAR + crate::map::PLAY_H
            {
                draw_circle_lines(
                    mouse.x,
                    mouse.y,
                    kind.range(),
                    1.5,
                    Color::new(1.0, 1.0, 1.0, 0.5),
                );
            }
        }

        ui::draw_hud(self.gold, self.lives, self.wave_number, TOTAL_WAVES);
        ui::draw_panel(
            self.gold,
            self.selected_tower,
            &self.sprites,
            self.wave_active,
            SPEED_LEVELS[self.speed_index],
        );

        match self.status {
            GameStatus::GameOver => {
                ui::draw_center_message("Game Over", "The snails got through! Press R to restart.")
            }
            GameStatus::Win => {
                ui::draw_center_message("You Win!", "The garden is safe. Press R to play again.")
            }
            GameStatus::Playing => {}
        }
    }
}
