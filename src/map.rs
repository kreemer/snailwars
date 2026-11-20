//! The garden map: the fixed path snails follow, and the set of spots
//! where the player is allowed to build towers.
//!
//! The playable area is vertically offset by `TOP_BAR` to leave room for
//! the HUD, and the window reserves `PANEL_HEIGHT` at the bottom for the
//! tower-selection panel. All world coordinates below already include the
//! `TOP_BAR` offset, so drawing and mouse-hit code can use them directly.
//!
//! Map layout itself (path, build spots, ground tiles) is authored by hand
//! in Tiled and loaded via [`crate::level::Level`]; this module only turns
//! that loaded data into something the game can draw and hit-test against.
//!
//! `TILE` is purely a *logical* layout unit (world/screen pixels per grid
//! cell) - it is independent of the tileset image's own tile pixel size
//! (see [`crate::level::Level::source_tile_size`]), so swapping in a
//! tileset with a different tile resolution (e.g. 32px instead of 64px)
//! doesn't require changing the map's grid layout.
//!
//! `SCREEN_W`/`WINDOW_H` define the game's fixed *logical* resolution,
//! rendered to an offscreen target and then scaled/letterboxed onto the
//! actual (resizable) OS window by [`crate::viewport`].

use std::collections::HashMap;

use crate::{
    level::{Level, PaintedTile, TilesetInfo},
    tower::EffectType,
};
use macroquad::prelude::*;

pub const SCREEN_W: f32 = 960.0;
pub const PLAY_H: f32 = 640.0;
pub const TOP_BAR: f32 = 40.0;
pub const PANEL_HEIGHT: f32 = 100.0;
pub const WINDOW_H: f32 = TOP_BAR + PLAY_H + PANEL_HEIGHT;
pub const TILE: f32 = 64.0;
/// How long a lava pool stays on the ground, in seconds.
pub const LAVA_EFFECT_DURATION: f32 = 15.0;
/// Radius of a lava pool, used both for drawing and for deciding which
/// enemies are standing in it.
pub const LAVA_POOL_RADIUS: f32 = 20.0;
/// Final stretch of a pool's lifetime during which it fades out.
const LAVA_POOL_FADE_TIME: f32 = 2.0;

/// A patch of lava left on the ground where a Lava tower projectile hit.
/// Enemies inside it keep getting [`crate::tower::EffectType::Lava`].
#[derive(Clone, Debug)]
pub struct LavaPool {
    pub pos: Vec2,
    /// Seconds left before the pool disappears.
    pub remaining: f32,
}

impl LavaPool {
    pub fn new(pos: Vec2) -> Self {
        LavaPool {
            pos,
            remaining: LAVA_EFFECT_DURATION,
        }
    }

    /// Whether `pos` lies inside the pool.
    pub fn contains(&self, pos: Vec2) -> bool {
        (pos - self.pos).length() <= LAVA_POOL_RADIUS
    }

    pub fn update(&mut self, dt: f32) {
        self.remaining -= dt;
    }

    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }

    /// Opacity of the pool, fading out over its last
    /// [`LAVA_POOL_FADE_TIME`] seconds.
    fn alpha(&self) -> f32 {
        (self.remaining / LAVA_POOL_FADE_TIME).clamp(0.0, 1.0)
    }
}

pub struct Map {
    pub waypoints: Vec<Vec2>,
    pub build_spots: Vec<Vec2>,
    pub lava_pools: Vec<LavaPool>,
    /// The `Ground` layer's tile grid, `[row][col]` (or `None` for an
    /// empty cell).
    ground_tiles: Vec<Vec<Option<PaintedTile>>>,
    /// The `Env` layer's tile grid, `[row][col]`, drawn on top of
    /// `Ground` (or `None` for an empty cell).
    env_tiles: Vec<Vec<Option<PaintedTile>>>,
    /// Every tileset referenced by the map; see
    /// [`crate::level::Level::tilesets`]. Used only to compute each
    /// tile's source `Rect` when cropping it for drawing - each cell is
    /// still drawn at the fixed logical [`TILE`] size on screen.
    tilesets: Vec<TilesetInfo>,
    /// Full map size in world pixels (`cols * TILE`, `rows * TILE`). May
    /// be larger than the fixed `SCREEN_W x PLAY_H` play area, in which
    /// case [`crate::viewport::Viewport`] scrolls the camera to show the
    /// rest of the map instead of shrinking it to fit.
    world_size: Vec2,
}

impl Map {
    /// Build the playable map from a loaded [`Level`].
    pub fn from_level(level: &Level) -> Self {
        assert!(
            level.cols as f32 * TILE >= SCREEN_W,
            "level map width ({} cols) is narrower than the {SCREEN_W}-wide play area",
            level.cols
        );
        assert!(
            level.rows as f32 * TILE >= PLAY_H,
            "level map height ({} rows) is shorter than the {PLAY_H}-tall play area",
            level.rows
        );

        Map {
            waypoints: level.waypoints.clone(),
            build_spots: level.build_spots.clone(),
            lava_pools: vec![],
            ground_tiles: level.ground_tiles.clone(),
            env_tiles: level.env_tiles.clone(),
            tilesets: level.tilesets.clone(),
            world_size: vec2(level.cols as f32 * TILE, level.rows as f32 * TILE),
        }
    }

    /// Full map size in world pixels (`cols * TILE`, `rows * TILE`), used
    /// by [`crate::viewport::Viewport`] to clamp how far the camera may
    /// scroll when the map is larger than the fixed play area.
    pub fn world_size(&self) -> Vec2 {
        self.world_size
    }

    /// Draw the hand-painted `Ground` layer, then the `Env` layer on top
    /// of it, using `textures` (one loaded [`Texture2D`] per entry in
    /// [`Level::tilesets`], in the same order - see `main.rs`). Each
    /// source tile (whatever pixel size its own tileset image uses) is
    /// scaled to fill one `TILE`-sized logical grid cell.
    pub fn draw(&self, textures: &[Texture2D]) {
        Self::draw_layer(&self.ground_tiles, &self.tilesets, textures);
        Self::draw_layer(&self.env_tiles, &self.tilesets, textures);
    }

    fn draw_layer(
        layer: &[Vec<Option<PaintedTile>>],
        tilesets: &[TilesetInfo],
        textures: &[Texture2D],
    ) {
        for (row, tiles) in layer.iter().enumerate() {
            for (col, tile) in tiles.iter().enumerate() {
                let Some(tile) = tile else { continue };
                let tileset = &tilesets[tile.tileset_index];
                let dest_x = col as f32 * TILE;
                let dest_y = row as f32 * TILE + TOP_BAR;
                let src_col = (tile.local_id % tileset.columns) as f32;
                let src_row = (tile.local_id / tileset.columns) as f32;
                draw_texture_ex(
                    &textures[tile.tileset_index],
                    dest_x,
                    dest_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(TILE, TILE)),
                        source: Some(Rect::new(
                            src_col * tileset.tile_size,
                            src_row * tileset.tile_size,
                            tileset.tile_size,
                            tileset.tile_size,
                        )),
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn draw_build_spots(&self, occupied: &[bool], build_spot_tex: &Texture2D) {
        for (i, spot) in self.build_spots.iter().enumerate() {
            if occupied[i] {
                continue;
            }
            let half = build_spot_tex.width() / 2.0;
            draw_texture(build_spot_tex, spot.x - half, spot.y - half, WHITE);
        }
    }

    /// Leave a fresh lava pool on the ground at `pos`.
    pub fn add_lava_pool(&mut self, pos: Vec2) {
        self.lava_pools.push(LavaPool::new(pos));
    }

    /// Age all lava pools by `dt` seconds, removing expired ones.
    pub fn update_lava_pools(&mut self, dt: f32) {
        for pool in &mut self.lava_pools {
            pool.update(dt);
        }
        self.lava_pools.retain(|pool| !pool.is_expired());
    }

    pub fn draw_lava_pools(&self, texture: &Texture2D) {
        let size = LAVA_POOL_RADIUS * 2.0;
        for pool in &self.lava_pools {
            draw_texture_ex(
                texture,
                pool.pos.x - LAVA_POOL_RADIUS,
                pool.pos.y - LAVA_POOL_RADIUS,
                Color::new(1.0, 1.0, 1.0, pool.alpha()),
                DrawTextureParams {
                    dest_size: Some(vec2(size, size)),
                    ..Default::default()
                },
            );
        }
    }

    /// Index of the nearest unoccupied build spot to `pos`, within
    /// `max_dist`, if any.
    pub fn nearest_free_spot(&self, pos: Vec2, occupied: &[bool], max_dist: f32) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (i, spot) in self.build_spots.iter().enumerate() {
            if occupied[i] {
                continue;
            }
            let d = (pos - *spot).length();
            if d <= max_dist && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        best.map(|(i, _)| i)
    }

    /// Index of the nearest occupied build spot to `pos`, within
    /// `max_dist`, if any.
    pub fn nearest_occupied_spot(
        &self,
        pos: Vec2,
        occupied: &[bool],
        max_dist: f32,
    ) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (i, spot) in self.build_spots.iter().enumerate() {
            if !occupied[i] {
                continue;
            }
            let d = (pos - *spot).length();
            if d <= max_dist && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        best.map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lava_pool_expires_after_its_duration() {
        let mut pool = LavaPool::new(vec2(0.0, 0.0));
        pool.update(LAVA_EFFECT_DURATION - 0.1);
        assert!(!pool.is_expired());
        pool.update(0.2);
        assert!(pool.is_expired());
    }

    #[test]
    fn lava_pool_contains_only_positions_within_its_radius() {
        let pool = LavaPool::new(vec2(100.0, 100.0));
        assert!(pool.contains(vec2(100.0 + LAVA_POOL_RADIUS, 100.0)));
        assert!(!pool.contains(vec2(100.0 + LAVA_POOL_RADIUS + 1.0, 100.0)));
    }
}
