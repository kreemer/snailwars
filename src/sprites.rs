//! Sprites used by the game.
//!
//! Enemy art (`snail`, `slug`, `big_snail`) is loaded from PNG files in
//! `assets/enemy/`, generated ahead of time by `assets/enemy/generate.py`
//! - original artwork inspired by hand-drawn crayon sketches (see
//! `assets/enemy/raw/` for the reference photos), rendered with a
//! transparent background. Everything else (towers, projectile, build
//! spot overlay) has no art yet, so it is still built at startup by
//! rasterizing simple shapes (circles / rounded rectangles) into an
//! `Image` and uploading it as a `Texture2D`.

use macroquad::prelude::*;

/// All textures used by the game, loaded/generated once at startup.
#[derive(Clone)]
pub struct Sprites {
    pub snail: Texture2D,
    pub slug: Texture2D,
    pub big_snail: Texture2D,
    pub tower_pebble: Texture2D,
    pub tower_pepper: Texture2D,
    pub tower_salt: Texture2D,
    pub tower_death: Texture2D,
    pub projectile: Texture2D,
    /// Overlay marker drawn over unoccupied build spots; the ground art
    /// itself comes from the level's Tiled tileset (see `crate::map`).
    pub build_spot: Texture2D,
}

impl Sprites {
    /// Loads enemy art from disk and generates the remaining placeholder
    /// textures. Texture loading is async in macroquad, so this must be
    /// awaited before starting the game (see `main.rs`).
    pub async fn load() -> Self {
        Sprites {
            snail: load_enemy_texture("snail").await,
            slug: load_enemy_texture("slug").await,
            big_snail: load_enemy_texture("big_snail").await,
            ..Self::generate_placeholders()
        }
    }

    fn generate_placeholders() -> Self {
        Sprites {
            snail: circle_texture(
                28,
                Color::new(0.85, 0.55, 0.2, 1.0),
                Color::new(0.5, 0.3, 0.1, 1.0),
            ),
            slug: circle_texture(
                24,
                Color::new(0.6, 0.2, 0.7, 1.0),
                Color::new(0.35, 0.1, 0.45, 1.0),
            ),
            big_snail: circle_texture(
                40,
                Color::new(0.8, 0.1, 0.1, 1.0),
                Color::new(0.45, 0.05, 0.05, 1.0),
            ),
            tower_pebble: rounded_rect_texture(
                32,
                Color::new(0.55, 0.55, 0.6, 1.0),
                Color::new(0.2, 0.2, 0.25, 1.0),
            ),
            tower_pepper: rounded_rect_texture(
                32,
                Color::new(0.9, 0.3, 0.1, 1.0),
                Color::new(0.5, 0.15, 0.05, 1.0),
            ),
            tower_salt: rounded_rect_texture(
                32,
                Color::new(0.2, 0.5, 0.9, 1.0),
                Color::new(0.1, 0.25, 0.5, 1.0),
            ),
            tower_death: rounded_rect_texture(
                32,
                Color::new(0.2, 1.0, 0.0, 1.0),
                Color::new(0.1, 0.25, 0.5, 1.0),
            ),
            projectile: circle_texture(
                8,
                Color::new(1.0, 0.95, 0.4, 1.0),
                Color::new(0.6, 0.55, 0.1, 1.0),
            ),
            build_spot: rounded_rect_texture(
                48,
                Color::new(0.3, 0.35, 0.3, 0.55),
                Color::new(0.9, 0.9, 0.9, 0.8),
            ),
        }
    }
}

/// Loads an enemy texture from `assets/enemy/{name}.png`, panicking with
/// a clear message if it's missing (same failure style as the tileset
/// load in `main.rs`).
async fn load_enemy_texture(name: &str) -> Texture2D {
    let path = format!("assets/enemy/{name}.png");
    let texture = load_texture(&path)
        .await
        .unwrap_or_else(|e| panic!("failed to load enemy texture '{path}': {e}"));
    texture.set_filter(FilterMode::Linear);
    texture
}

fn circle_texture(size: u16, fill: Color, outline: Color) -> Texture2D {
    let mut image = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));
    let center = size as f32 / 2.0;
    let radius = center - 1.0;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= radius {
                let color = if dist >= radius - 2.0 { outline } else { fill };
                image.set_pixel(x as u32, y as u32, color);
            }
        }
    }
    Texture2D::from_image(&image)
}

fn rounded_rect_texture(size: u16, fill: Color, outline: Color) -> Texture2D {
    let mut image = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));
    let corner = size as f32 * 0.25;
    let w = size as f32;
    for y in 0..size {
        for x in 0..size {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            if inside_rounded_rect(fx, fy, w, w, corner) {
                let border = 2.0;
                let is_border = !inside_rounded_rect_margin(fx, fy, w, w, corner, border);
                image.set_pixel(x as u32, y as u32, if is_border { outline } else { fill });
            }
        }
    }
    Texture2D::from_image(&image)
}

fn inside_rounded_rect(x: f32, y: f32, w: f32, h: f32, corner: f32) -> bool {
    inside_rounded_rect_margin(x, y, w, h, corner, 0.0)
}

fn inside_rounded_rect_margin(x: f32, y: f32, w: f32, h: f32, corner: f32, margin: f32) -> bool {
    let x0 = margin;
    let y0 = margin;
    let x1 = w - margin;
    let y1 = h - margin;
    if x < x0 || x > x1 || y < y0 || y > y1 {
        return false;
    }
    let c = corner;
    let corners = [
        (x0 + c, y0 + c),
        (x1 - c, y0 + c),
        (x0 + c, y1 - c),
        (x1 - c, y1 - c),
    ];
    for (cx, cy) in corners {
        let in_corner_zone = (x < cx && (cx - x0) > 0.0 && x < x0 + c) || (x > cx && x > x1 - c);
        let in_corner_zone_y = (y < cy && y < y0 + c) || (y > cy && y > y1 - c);
        if in_corner_zone && in_corner_zone_y {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy > c * c {
                return false;
            }
        }
    }
    true
}
