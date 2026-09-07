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

use crate::level::{Level, PaintedTile, TilesetInfo};
use macroquad::prelude::*;

pub const SCREEN_W: f32 = 960.0;
pub const PLAY_H: f32 = 640.0;
pub const TOP_BAR: f32 = 40.0;
pub const PANEL_HEIGHT: f32 = 100.0;
pub const WINDOW_H: f32 = TOP_BAR + PLAY_H + PANEL_HEIGHT;
pub const TILE: f32 = 64.0;

pub struct Map {
    pub waypoints: Vec<Vec2>,
    pub build_spots: Vec<Vec2>,
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
