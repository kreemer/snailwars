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

use crate::level::Level;
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
    /// The `Ground` layer's tile grid, `[row][col]`, holding each cell's
    /// local tile id within the tileset (or `None` for an empty cell).
    ground_tiles: Vec<Vec<Option<u32>>>,
}

impl Map {
    /// Build the playable map from a loaded [`Level`].
    pub fn from_level(level: &Level) -> Self {
        assert_eq!(
            level.tile_size, TILE,
            "level map tile size ({}) must match the game's TILE constant ({TILE})",
            level.tile_size
        );
        assert_eq!(
            level.cols as f32 * TILE,
            SCREEN_W,
            "level map width ({} cols) must fill the {SCREEN_W}-wide play area",
            level.cols
        );
        assert_eq!(
            level.rows as f32 * TILE,
            PLAY_H,
            "level map height ({} rows) must fill the {PLAY_H}-tall play area",
            level.rows
        );

        Map {
            waypoints: level.waypoints.clone(),
            build_spots: level.build_spots.clone(),
            ground_tiles: level.ground_tiles.clone(),
        }
    }

    /// Draw the hand-painted `Ground` layer using the level's tileset
    /// texture: a single row of square tiles, indexed by local tile id.
    pub fn draw(&self, tileset: &Texture2D) {
        for (row, tiles) in self.ground_tiles.iter().enumerate() {
            for (col, tile_id) in tiles.iter().enumerate() {
                let Some(id) = tile_id else { continue };
                let dest_x = col as f32 * TILE;
                let dest_y = row as f32 * TILE + TOP_BAR;
                draw_texture_ex(
                    tileset,
                    dest_x,
                    dest_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(TILE, TILE)),
                        source: Some(Rect::new(*id as f32 * TILE, 0.0, TILE, TILE)),
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
            if d <= max_dist
                && best.is_none_or(|(_, bd)| d < bd) {
                    best = Some((i, d));
                }
        }
        best.map(|(i, _)| i)
    }
}
