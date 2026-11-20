# Placeholder art and new level designs

Image dimensions below are **width x height in pixels**. Generated placeholders
are the reference sizes; the game scales sprites when drawing them. Save PNGs
relative to the project root. Transparent backgrounds suit sprites and effects.

## Art with an existing file loader

Saving these files at the listed paths replaces the generated placeholder (or,
for the title, its plain-color background) without code changes.

| Subject | Filename | Reference size |
|---|---|---:|
| Flying snail | `assets/enemy/flying_snail.png` | 24 x 24 |
| Armored snail | `assets/enemy/armored_snail.png` | 30 x 30 |
| Armored slug | `assets/enemy/armored_slug.png` | 26 x 26 |
| Armored big snail | `assets/enemy/armored_big_snail.png` | 44 x 44 |
| Slow icon | `assets/effects/slow.png` | 16 x 16 |
| Poison icon | `assets/effects/poison.png` | 16 x 16 |
| Lava icon | `assets/effects/lava.png` | 16 x 16 |
| Lava pool | `assets/effects/lava_pool.png` | 64 x 64 |
| Pebble projectile | `assets/projectiles/pebble.png` | 24 x 24 |
| Pepper projectile | `assets/projectiles/pepper.png` | 16 x 16 |
| Salt projectile | `assets/projectiles/salt.png` | 24 x 24 |
| Kost des Todes projectile | `assets/projectiles/death.png` | 32 x 32 |
| Lava projectile | `assets/projectiles/lava.png` | 32 x 32 |
| Title background | `assets/ui/title_background.png` | 960 x 780 |

`assets/tilesets/tiles.png` is an existing **160 x 32** placeholder tilesheet:
five 32 x 32 tiles in one row. All five levels reference it. Preserve its tile
order when replacing it, because the Tiled maps use those tile IDs for path,
build, start, and end cells.

The existing `assets/enemy/snail.png`, `slug.png`, and `big_snail.png` already
contain artwork and are not generated fallbacks.

## Generated-only art

These are placeholder sizes and **suggested** filenames, not working asset
paths: the game does not load PNGs for them yet. Saving files here alone will
not replace the generated textures; image loading must be added to `src/sprites.rs`.

| Subject | Suggested filename | Placeholder size |
|---|---|---:|
| Pebble Turret | `assets/towers/pebble.png` | 32 x 32 |
| Laser Tower | `assets/towers/laser.png` | 32 x 32 |
| Pepper Sprayer | `assets/towers/pepper.png` | 32 x 32 |
| Salt Cannon | `assets/towers/salt.png` | 32 x 32 |
| Kost des Todes tower | `assets/towers/death.png` | 32 x 32 |
| Lava tower | `assets/towers/lava.png` | 32 x 32 |
| Build-spot overlay | `assets/ui/build_spot.png` | 48 x 48 |
| Enemy-level icon | `assets/ui/enemy_level.png` | 256 x 256 |

The enemy-level icon placeholder is currently unused; enemy levels are drawn
as text. The laser fires a beam rather than using a projectile PNG.

## Level 4: The Thorny Trellis

| Asset | Purpose | Dimensions |
|---|---|---|
| `assets/maps/level4.tmx` | Tiled map; ground, environment, and path/build logic | 20 x 20 cells at 32 x 32 pixels per Tiled cell; 64 x 64 logical pixels in-game |
| `assets/maps/level4.waves.ron` | 12 hand-authored waves | Not an image |

**Existing route:** Starts at the left edge `(0, 2)`, runs east to `(15, 2)`,
turns south to `(15, 7)`, winds west to `(4, 7)`, then goes south to `(4, 16)`
and east to the base at `(18, 16)`. Coordinates are zero-based Tiled cells.
Build spots flank the path, especially its long straight sections.

**Suggested visual direction:** A thorn-lined trellis or overgrown garden
walkway. Use the `Ground` layer for the route and foliage-colored ground, and
the `Env` layer for trellises, vines, and thorns outside the path and build
spots. The current map uses simple ground/path tiles and sparse decoration;
this is an art brief, not artwork already present in the map.

**Difficulty:** Waves grow from 18 to 40 enemies. Enemy levels start at 2
and rise to 3 halfway through; flying and armored enemies enter later.
Spawn intervals are tighter than earlier levels.

## Level 5: The Root Cellar

| Asset | Purpose | Dimensions |
|---|---|---|
| `assets/maps/level5.tmx` | Tiled map; ground, environment, and path/build logic | 20 x 20 cells at 32 x 32 pixels per Tiled cell; 64 x 64 logical pixels in-game |
| `assets/maps/level5.waves.ron` | 12 hand-authored waves | Not an image |

**Existing route:** Starts at `(18, 3)`, moves west to `(3, 3)`, south to
`(3, 8)`, east to `(16, 8)`, south to `(16, 13)`, west to `(6, 13)`,
south to `(6, 17)`, and east to the base at `(18, 17)`. Build spots
sit near the lane throughout.

**Suggested visual direction:** A dark underground cellar with roots, soil,
and storage details. Keep path and build cells readable beneath any decorative
props. The current map uses the shared garden tileset with sparse decoration;
new cellar art would require editing the existing `Ground`/`Env` layers or
adding tilesets in Tiled. A standalone `level5.png` would not be loaded.

**Difficulty:** Waves grow from 30 to 52 enemies, with shorter intervals
than level 4. Enemy levels start at 4 and rise to 5 halfway through; later
waves mix armored bosses with flying enemies.

Both maps use the shared `assets/tilesets/tiles.png` for gameplay cells and
`assets/tilesets/Serene_Village_32x32.png` for visual tiles. There are no
per-level background PNG paths; edit the `.tmx` maps in Tiled to change their
appearance while keeping the `Logic` path connected.
