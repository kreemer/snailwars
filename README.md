# Snail Wars

A simple tower defense game written in Rust using [`macroquad`](https://github.com/not-fl3/macroquad).

Snails and slugs crawl along a hand-drawn path through the garden. Place towers
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
| Pepper Sprayer  | 75g  | 3      | 90    | 4.0/s     | Fast, applies Poison |
| Salt Cannon     | 150g | 35     | 140   | 0.6/s     | Splash damage, applies Slow |

## Effects

Towers can apply temporary debuffs. Every active debuff is shown as an icon
below the enemy; the art is loaded from `assets/effects/{name}.png` and falls
back to a generated placeholder while it is missing.

| Effect | Applied by     | Duration | Notes                       |
|--------|----------------|----------|-----------------------------|
| Slow   | Salt Cannon    | 10s      | 25% movement speed reduction |
| Poison | Pepper Sprayer | 4s       | 8 damage per second          |

Re-applying an effect refreshes its duration.

## Enemies

| Enemy     | HP  | Speed | Reward | Notes            |
|-----------|-----|-------|--------|------------------|
| Snail     | 40  | Slow  | 5g     | Basic enemy      |
| Slug      | 25  | Fast  | 4g     | Low HP, quick    |
| Big Snail | 220 | Slow  | 30g    | Boss, every 5 waves |

You start with 150 gold and 20 lives.

## Maps

Maps are hand-drawn in [Tiled](https://www.mapeditor.org/) and loaded at
startup (currently `assets/maps/level1.tmx`, wired up in `src/main.rs`; the
loader (`src/level.rs`) supports any `<name>.tmx`/`<name>.waves.ron` pair,
so more levels can be added and switched between later).

Each level is a pair of files living in `assets/maps/`:

- `<name>.tmx` — the Tiled map itself, using the tileset at
  `assets/tilesets/tiles.tsx` (backed by `tiles.png`, a placeholder
  spritesheet - swap it for real art any time, the `kind` tile properties
  described below are all the game depends on). The map must be 15x10
  tiles at 64px each (matching the fixed 960x640 play area) and contain
  exactly two tile layers:
  - `Ground` — purely visual; paint it with whatever tileset tiles look
    right (grass, path swatches, etc.). Not used for gameplay logic.
  - `Logic` — gameplay data, normally hidden in Tiled (it's not meant to
    be seen by the player, but its tile data is still read regardless of
    layer visibility). Paint it with these tileset tiles:
    - `path` — a cell the snails walk through
    - `start` — the single spawn cell (must appear exactly once)
    - `end` — the single base cell (must appear exactly once)
    - `build` — a cell where the player may place a tower
    The painted `path`/`start`/`end` cells must form one single,
    unbranching lane connecting `start` to `end` (4-directional
    adjacency, no forks, no disconnected pieces) - the game traces this
    chain into the ordered path snails follow, and panics with a
    descriptive error at startup if the painting is invalid.
- `<name>.waves.ron` — the hand-authored list of waves for that level, in
  [RON](https://github.com/ron-rs/ron) format:

  ```ron
  (
      waves: [
          (
              spawns: [
                  (kind: Snail, delay_after_previous: 0.0),
                  (kind: Snail, delay_after_previous: 0.7),
              ],
          ),
          // one entry per wave, in order; `kind` is any EnemyType
          // variant (Snail, Slug, BigSnail), `delay_after_previous` is
          // the pause (seconds) before that spawn, relative to the
          // previous one in the same wave.
      ],
  )
  ```

To add a new level: open `assets/maps/level1.tmx` in Tiled as a starting
point, save-as `<name>.tmx`, redraw `Ground`/`Logic`, write a matching
`<name>.waves.ron`, then point `src/main.rs`'s `level_name` at `<name>`.

### Custom tile properties

The `kind` property is set per-tile in `tiles.tsx`'s tileset editor (Tiled:
select a tile in the tileset view → Properties panel → add a `string`
property named `kind`). The game only looks at this property; it doesn't
care about tile IDs or which image is used, so you can freely reskin
`tiles.png` as long as the `kind` properties stay attached to the tiles
you use for path/start/end/build cells.

## Project layout

- `src/main.rs` — window setup, level loading, and main game loop
- `src/game.rs` — core game state, update/draw orchestration
- `src/level.rs` — loads a level's `.tmx` map and `.waves.ron` wave file
  (path tracing, tile parsing)
- `src/map.rs` — playable map built from a loaded level: path waypoints,
  buildable spots, ground tile rendering, layout constants
- `src/enemy.rs` — enemy types, movement, health
- `src/tower.rs` — tower types, targeting, firing
- `src/projectile.rs` — projectile movement and damage application
- `src/wave.rs` — wave/spawn-entry data shapes (deserialized from a
  level's `.waves.ron` file)
- `src/sprites.rs` — procedurally generated placeholder textures for
  towers/enemies/projectiles/UI
- `src/ui.rs` — HUD, tower panel, and overlay screens

Enemy/tower/projectile sprites are simple procedurally generated shapes (no
external art assets); swap in real textures later by replacing
`sprites.rs`. Ground tiles come from `assets/tilesets/tiles.png` instead.

