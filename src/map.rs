//! The garden map: the fixed path snails follow, and the set of spots
//! where the player is allowed to build towers.
//!
//! The playable area is vertically offset by `TOP_BAR` to leave room for
//! the HUD, and the window reserves `PANEL_HEIGHT` at the bottom for the
//! tower-selection panel. All world coordinates below already include the
//! `TOP_BAR` offset, so drawing and mouse-hit code can use them directly.

use macroquad::prelude::*;

pub const SCREEN_W: f32 = 960.0;
pub const PLAY_H: f32 = 640.0;
pub const TOP_BAR: f32 = 40.0;
pub const PANEL_HEIGHT: f32 = 100.0;
pub const WINDOW_H: f32 = TOP_BAR + PLAY_H + PANEL_HEIGHT;
pub const TILE: f32 = 64.0;
pub const PATH_WIDTH: f32 = 48.0;

pub struct Map {
    pub waypoints: Vec<Vec2>,
    pub build_spots: Vec<Vec2>,
}

impl Map {
    pub fn new() -> Self {
        // A simple zig-zag path across the garden, left to right, with
        // world-space y already shifted down by TOP_BAR.
        let raw_waypoints = [
            (-32.0, 100.0),
            (200.0, 100.0),
            (200.0, 300.0),
            (700.0, 300.0),
            (700.0, 120.0),
            (880.0, 120.0),
            (880.0, 480.0),
            (120.0, 480.0),
            (120.0, 560.0),
            (SCREEN_W + 32.0, 560.0),
        ];
        let waypoints: Vec<Vec2> = raw_waypoints.iter().map(|&(x, y)| vec2(x, y + TOP_BAR)).collect();

        let build_spots = generate_build_spots(&waypoints);

        Map { waypoints, build_spots }
    }

    pub fn draw(&self, grass_tile: &Texture2D) {
        // Tile the background with grass.
        let mut y = TOP_BAR;
        while y < TOP_BAR + PLAY_H {
            let mut x = 0.0;
            while x < SCREEN_W {
                draw_texture(grass_tile, x, y, WHITE);
                x += TILE;
            }
            y += TILE;
        }

        // Draw the path as thick line segments between waypoints.
        for pair in self.waypoints.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            draw_line(a.x, a.y, b.x, b.y, PATH_WIDTH, Color::new(0.72, 0.6, 0.4, 1.0));
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

/// Lay out a grid of candidate positions, then keep only ones that are far
/// enough from the path (so towers don't overlap it).
fn generate_build_spots(waypoints: &[Vec2]) -> Vec<Vec2> {
    let mut spots = Vec::new();
    let spacing = 80.0;
    let margin = 40.0;

    let mut y = TOP_BAR + margin;
    while y < TOP_BAR + PLAY_H - margin {
        let mut x = margin;
        while x < SCREEN_W - margin {
            let p = vec2(x, y);
            if min_dist_to_path(p, waypoints) > PATH_WIDTH / 2.0 + 34.0 {
                spots.push(p);
            }
            x += spacing;
        }
        y += spacing;
    }
    spots
}

fn min_dist_to_path(p: Vec2, waypoints: &[Vec2]) -> f32 {
    let mut min_dist = f32::MAX;
    for pair in waypoints.windows(2) {
        let d = dist_point_to_segment(p, pair[0], pair[1]);
        if d < min_dist {
            min_dist = d;
        }
    }
    min_dist
}

fn dist_point_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq == 0.0 {
        return (p - a).length();
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    let proj = a + ab * t;
    (p - proj).length()
}
