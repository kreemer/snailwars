# Snail Wars

A simple tower defense game written in Rust using [`macroquad`](https://github.com/not-fl3/macroquad).

Snails and slugs crawl along a fixed path through the garden. Place towers
along the way to stop them before they reach your base.

## Running

```sh
cargo run
```

## Controls

- **Click a tower button** (bottom-left panel) to select a tower type, then
  **click a highlighted spot** on the map to place it (if you can afford it).
- Click the selected tower button again to deselect.
- **Start Wave** button (bottom-right) begins the next wave of enemies.
- **Speed button** (1x/2x/3x, next to Start Wave) or **Tab** cycles the
  simulation speed, so you can fast-forward through waves.
- Press **R** to restart after a game over or win.

## Towers

| Tower           | Cost | Damage | Range | Fire rate | Notes           |
|-----------------|------|--------|-------|-----------|-----------------|
| Pebble Turret   | 50g  | 12     | 120   | 1.0/s     | Balanced        |
| Pepper Sprayer  | 75g  | 5      | 90    | 4.0/s     | Fast, low damage|
| Salt Cannon     | 150g | 35     | 140   | 0.6/s     | Splash damage   |

## Enemies

| Enemy     | HP  | Speed | Reward | Notes            |
|-----------|-----|-------|--------|------------------|
| Snail     | 40  | Slow  | 5g     | Basic enemy      |
| Slug      | 25  | Fast  | 4g     | Low HP, quick    |
| Big Snail | 220 | Slow  | 30g    | Boss, every 5 waves |

You start with 150 gold and 20 lives. Survive all 10 waves to win.

## Project layout

- `src/main.rs` — window setup and main game loop
- `src/game.rs` — core game state, update/draw orchestration
- `src/map.rs` — path waypoints, buildable spots, layout constants
- `src/enemy.rs` — enemy types, movement, health
- `src/tower.rs` — tower types, targeting, firing
- `src/projectile.rs` — projectile movement and damage application
- `src/wave.rs` — per-wave enemy spawn definitions
- `src/sprites.rs` — procedurally generated placeholder textures
- `src/ui.rs` — HUD, tower panel, and overlay screens

All sprites are simple procedurally generated shapes (no external art
assets); swap in real textures later by replacing `sprites.rs`.
