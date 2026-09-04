# Snail Wars

A simple tower defense game written in Rust using [`macroquad`](https://github.com/not-fl3/macroquad).

Snails and slugs crawl along a hand-drawn path through the garden. Place towers
along the way to stop them before they reach your base.

## Running

```sh
cargo run
```

## Screens

The game opens on a **title screen** (background art is loaded from
`assets/ui/title_background.png`; a plain backdrop is drawn while that
file is missing). *Play* leads to the **map selection** screen, which
lists every level from `assets/maps/levels.ron`. Only unlocked maps can
be picked - a map unlocks once the map before it has been beaten, and
the first one is always available.

Progress is stored in `save.ron` next to where the game is run from.
Delete that file to reset all unlocks.

## Controls

- **Click a tower button** (bottom-left panel) to select a tower type, then
  **click a highlighted spot** on the map to place it (if you can afford it).
- Click the selected tower button again to deselect.
- **Start Wave** button (bottom-right) begins the next wave of enemies.
- **Speed button** (1x/2x/3x, next to Start Wave) or **Tab** cycles the
  simulation speed, so you can fast-forward through waves.
- **Esc** leaves a level and returns to the title screen.
- After winning, choose **Next Level** (if there is one) or **Back to
  Title**; after losing, choose **Restart** or **Back to Title**.
  **R** also restarts the current level.

## Towers

| Tower           | Cost | Damage | Range | Fire rate | Notes           |
|-----------------|------|--------|-------|-----------|-----------------|
| Pebble Turret   | 50g  | 12     | 120   | 1.0/s     | Balanced; can hit flying enemies |
| Pepper Sprayer  | 75g  | 3      | 100   | 5.0/s     | Fast, applies Poison |
| Salt Cannon     | 150g | 30     | 150   | 0.6/s     | Splash damage, applies Slow |
| Kost des Todes  | 300g | 50     | 200   | 0.5/s     | Heavy splash; can hit flying enemies |

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

| Enemy             | HP  | Speed | Armor | Reward | Notes                              |
|-------------------|-----|-------|-------|--------|------------------------------------|
| Snail             | 50  | 55    | -     | 5g     | Basic enemy                        |
| Slug              | 30  | 105   | -     | 4g     | Low HP, quick                      |
| Flying Snail      | 40  | 80    | -     | 8g     | Flying - only Pebble Turret and Kost des Todes can hit it |
| Big Snail         | 500 | 45    | -     | 30g    | Boss                               |
| Armored Slug      | 45  | 95    | 3     | 8g     | Fast, lightly plated               |
| Armored Snail     | 70  | 50    | 6     | 10g    | Shrugs off small hits              |
| Armored Big Snail | 600 | 40    | 12    | 60g    | Armored boss                       |

### Armor

Armored enemies absorb a flat amount of damage from every **direct hit**
(a projectile impact, splash included), down to a minimum of 1 damage per
hit - armor slows an enemy's death down but never makes it immortal.

Two things go straight through armor:

- **Poison** damage over time, which always ticks at its full 8 dps.
- **Kost des Todes**, whose projectiles ignore armor entirely.

So a Pepper Sprayer's 3 damage shots drop to 1 against an Armored Snail,
but the Poison they apply still hurts at full strength; a Pebble Turret
does 6 of its 12 damage, and a Salt Cannon 24 of its 30.

You start with 150 gold and 20 lives.

## Maps

Maps are hand-drawn in [Tiled](https://www.mapeditor.org/). Which maps
exist, what they are called, and the order they unlock in is defined by
`assets/maps/levels.ron`:

```ron
(
    levels: [
        (id: "level1", name: "The Garden Path"),
        (id: "level2", name: "Greenhouse Alley"),
        (id: "level3", name: "The Compost Maze"),
    ],
)
```

`id` is the base name of the level's file pair in `assets/maps/`; `name`
is the label shown on the map selection screen. The list order is the
progression order: each level unlocks once the one listed before it has
been beaten.

Each level is a pair of files living in `assets/maps/`:

- `<name>.tmx` — the Tiled map itself. It may reference several tilesets;
  gameplay-relevant cells come from `assets/tilesets/tiles.tsx` (backed by
  `tiles.png`, a placeholder spritesheet - swap it for real art any time,
  the `kind` tile properties described below are all the game depends on),
  while decorative cells may come from any other tileset the map declares
  (e.g. `Serene_Village_32x32.png`).

  The map's own `tilewidth`/`tileheight` (32px in the shipped levels) is
  only Tiled's painting grid — in game every cell is drawn at the fixed
  logical `TILE` size of 64px. The map must therefore be at least 15x10
  cells to cover the fixed 960x640 play area; the shipped levels are 20x20,
  and anything larger than the play area is scrolled by the camera rather
  than shrunk to fit.

  The map contains these tile layers:
  - `Ground` — purely visual; paint it with whatever tileset tiles look
    right (grass, path swatches, etc.). Not used for gameplay logic.
  - `Env` — optional, purely visual, drawn on top of `Ground`. Use it for
    props (bushes, trees, flowers) that should overlap the ground art.
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

    Keep `build` cells next to the lane: tower ranges are 100-200 world
    pixels against 64px cells, so a spot more than one cell away from the
    path is out of reach for the cheaper towers.
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
          // variant (Snail, Slug, FlyingSnail, BigSnail,
          // ArmoredSnail, ArmoredSlug, ArmoredBigSnail),
          // `delay_after_previous` is the pause (seconds) before that
          // spawn, relative to the previous one in the same wave.
      ],
  )
  ```

To add a new level: open `assets/maps/level1.tmx` in Tiled as a starting
point, save-as `<name>.tmx`, redraw `Ground`/`Env`/`Logic`, write a matching
`<name>.waves.ron`, then append `(id: "<name>", name: "...")` to
`assets/maps/levels.ron`. It will show up on the map selection screen,
locked until the level listed before it has been beaten.

### Custom tile properties

The `kind` property is set per-tile in `tiles.tsx`'s tileset editor (Tiled:
select a tile in the tileset view → Properties panel → add a `string`
property named `kind`). The game only looks at this property; it doesn't
care about tile IDs or which image is used, so you can freely reskin
`tiles.png` as long as the `kind` properties stay attached to the tiles
you use for path/start/end/build cells.

## Project layout

- `src/main.rs` — window setup and app startup
- `src/app.rs` — screen state machine (title, map selection, playing),
  level loading/switching, and end-of-level flow
- `src/catalog.rs` — the level list loaded from `assets/maps/levels.ron`
- `src/progress.rs` — which levels are beaten/unlocked, persisted to
  `save.ron`
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
- `src/ui.rs` — HUD, tower panel, title/map-selection screens, and the
  end-of-level overlay

Enemy/tower/projectile sprites are simple procedurally generated shapes (no
external art assets); swap in real textures later by replacing
`sprites.rs`. Ground tiles come from `assets/tilesets/tiles.png` instead.

