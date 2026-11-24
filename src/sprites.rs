//! Sprites used by the game.
//!
//! Enemy art (`snail`, `slug`, `big_snail`) is loaded from PNG files in
//! `assets/enemy/`, generated ahead of time by `assets/enemy/generate.py`
//! - original artwork inspired by hand-drawn crayon sketches (see
//! `assets/enemy/raw/` for the reference photos), rendered with a
//! transparent background. `flying_snail` and the armored variants
//! (`armored_snail`, `armored_slug`, `armored_big_snail`) use the same
//! convention but fall back to a generated placeholder while their art is
//! missing. The title screen background is looked for in
//! `assets/ui/title_background.png` and is simply absent (the title
//! screen paints a plain backdrop) while that art is missing.
//! Everything else (towers, projectiles, build
//! spot overlay, debuff icons) has no art yet, so it is still built at
//! startup by rasterizing simple shapes (circles / rounded rectangles /
//! rocks / crystals / glowing orbs) into an `Image` and uploading it as a
//! `Texture2D`. Debuff icons are looked for in
//! `assets/effects/{name}.png` first, the lava pool in
//! `assets/effects/lava_pool.png`, and projectiles in
//! `assets/projectiles/{name}.png`.

use macroquad::prelude::*;

use crate::tower::{EffectType, TowerType};

/// Optional art shown behind the title screen; see
/// [`Sprites::title_background`].
const TITLE_BACKGROUND_PATH: &str = "assets/ui/title_background.png";
/// Optional art for lava pools; see [`Sprites::lava_pool`].
const LAVA_POOL_PATH: &str = "assets/effects/lava_pool.png";

pub enum Direction {
    TOP,
    RIGHT,
    BOTTOM,
    LEFT,
}

/// All textures used by the game, loaded/generated once at startup.
#[derive(Clone)]
pub struct Sprites {
    pub snail: Texture2D,
    pub slug: Texture2D,
    pub big_snail: Texture2D,
    pub flying_snail: Texture2D,
    pub armored_snail: Texture2D,
    pub armored_slug: Texture2D,
    pub armored_big_snail: Texture2D,
    pub tower_pebble: Texture2D,
    pub tower_laser: Texture2D,
    pub tower_pepper: Texture2D,
    pub tower_salt: Texture2D,
    pub tower_death: Texture2D,
    pub tower_lava: Texture2D,
    /// One projectile per tower that fires them (the laser fires a beam
    /// instead; see [`crate::fx::LaserBeam`]). Looked for in
    /// `assets/projectiles/{name}.png` first; see
    /// [`crate::tower::TowerType::projectile_name`].
    pub projectile_pebble: Texture2D,
    pub projectile_pepper: Texture2D,
    pub projectile_salt: Texture2D,
    pub projectile_death: Texture2D,
    pub projectile_lava: Texture2D,
    /// Overlay marker drawn over unoccupied build spots; the ground art
    /// itself comes from the level's Tiled tileset (see `crate::map`).
    pub build_spot: Texture2D,
    /// Icons drawn below an enemy for each debuff it currently carries.
    pub effect_slow: Texture2D,
    pub effect_poison: Texture2D,
    pub effect_lava: Texture2D,
    /// Lava pool left on the ground by Lava tower hits; see
    /// [`crate::map::LavaPool`].
    pub lava_pool: Texture2D,
    /// Full-screen art behind the title screen, loaded from
    /// `assets/ui/title_background.png`. `None` while the art is
    /// missing, in which case the title screen paints a plain
    /// background instead.
    pub title_background: Option<Texture2D>,
}

impl Sprites {
    /// Loads enemy art from disk and generates the remaining placeholder
    /// textures. Texture loading is async in macroquad, so this must be
    /// awaited before starting the game (see `main.rs`).
    pub async fn load() -> Self {
        let placeholders = Self::generate_placeholders();
        Sprites {
            snail: load_enemy_texture("snail").await,
            slug: load_enemy_texture("slug").await,
            big_snail: load_enemy_texture("big_snail").await,
            flying_snail: try_load_enemy_texture("flying_snail")
                .await
                .unwrap_or_else(|| placeholders.flying_snail.clone()),
            armored_snail: try_load_enemy_texture("armored_snail")
                .await
                .unwrap_or_else(|| placeholders.armored_snail.clone()),
            armored_slug: try_load_enemy_texture("armored_slug")
                .await
                .unwrap_or_else(|| placeholders.armored_slug.clone()),
            armored_big_snail: try_load_enemy_texture("armored_big_snail")
                .await
                .unwrap_or_else(|| placeholders.armored_big_snail.clone()),
            effect_lava: load_effect_texture(EffectType::Lava, &placeholders.effect_lava).await,
            effect_slow: load_effect_texture(EffectType::Slow, &placeholders.effect_slow).await,
            effect_poison: load_effect_texture(EffectType::Poison, &placeholders.effect_poison)
                .await,
            lava_pool: try_load_texture(LAVA_POOL_PATH)
                .await
                .unwrap_or_else(|| placeholders.lava_pool.clone()),
            title_background: try_load_texture(TITLE_BACKGROUND_PATH).await,
            projectile_pebble: load_projectile_texture(
                TowerType::Pebble,
                &placeholders.projectile_pebble,
            )
            .await,
            projectile_pepper: load_projectile_texture(
                TowerType::Pepper,
                &placeholders.projectile_pepper,
            )
            .await,
            projectile_salt: load_projectile_texture(
                TowerType::Salt,
                &placeholders.projectile_salt,
            )
            .await,
            projectile_death: load_projectile_texture(
                TowerType::CostDesTodes,
                &placeholders.projectile_death,
            )
            .await,
            projectile_lava: load_projectile_texture(
                TowerType::Lava,
                &placeholders.projectile_lava,
            )
            .await,
            ..placeholders
        }
    }

    /// Projectile fired by towers of `kind`; `None` for the laser, which
    /// fires a beam instead.
    pub fn projectile_texture(&self, kind: TowerType) -> Option<&Texture2D> {
        match kind {
            TowerType::Pebble => Some(&self.projectile_pebble),
            TowerType::Pepper => Some(&self.projectile_pepper),
            TowerType::Salt => Some(&self.projectile_salt),
            TowerType::CostDesTodes => Some(&self.projectile_death),
            TowerType::Lava => Some(&self.projectile_lava),
            TowerType::Laser => None,
        }
    }

    /// Icon representing `effect`, drawn below enemies carrying it.
    pub fn effect_texture(&self, effect: EffectType) -> &Texture2D {
        match effect {
            EffectType::Slow => &self.effect_slow,
            EffectType::Poison => &self.effect_poison,
            EffectType::Lava => &self.effect_lava,
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
            flying_snail: circle_texture(
                24,
                Color::new(0.7, 0.85, 1.0, 1.0),
                Color::new(0.25, 0.4, 0.7, 1.0),
            ),
            armored_snail: circle_texture(
                30,
                Color::new(0.62, 0.65, 0.7, 1.0),
                Color::new(0.28, 0.3, 0.35, 1.0),
            ),
            armored_slug: circle_texture(
                26,
                Color::new(0.55, 0.5, 0.62, 1.0),
                Color::new(0.24, 0.22, 0.32, 1.0),
            ),
            armored_big_snail: circle_texture(
                44,
                Color::new(0.45, 0.48, 0.55, 1.0),
                Color::new(0.15, 0.17, 0.22, 1.0),
            ),
            tower_pebble: rounded_rect_texture(
                32,
                Color::new(0.55, 0.55, 0.6, 1.0),
                Color::new(0.2, 0.2, 0.25, 1.0),
            ),
            tower_laser: rounded_rect_texture(
                32,
                Color::new(0.65, 0.25, 0.9, 1.0),
                Color::new(0.3, 0.1, 0.45, 1.0),
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
            tower_lava: rounded_rect_texture(
                32,
                Color::new(1.0, 1.0, 0.0, 1.0),
                Color::new(0.5, 0.25, 0.5, 1.0),
            ),
            projectile_pebble: rock_texture(
                24,
                Color::new(0.62, 0.6, 0.56, 1.0),
                Color::new(0.2, 0.19, 0.17, 1.0),
            ),
            projectile_pepper: orb_texture(
                16,
                Color::new(0.95, 0.15, 0.05, 1.0),
                Color::new(1.0, 0.5, 0.1, 0.8),
            ),
            projectile_salt: diamond_texture(
                24,
                Color::new(0.97, 0.98, 1.0, 1.0),
                Color::new(0.35, 0.55, 0.8, 1.0),
            ),
            projectile_death: orb_texture(
                32,
                Color::new(0.05, 0.35, 0.05, 1.0),
                Color::new(0.3, 1.0, 0.2, 0.9),
            ),
            projectile_lava: orb_texture(
                32,
                Color::new(1.0, 0.55, 0.05, 1.0),
                Color::new(1.0, 0.2, 0.0, 0.9),
            ),
            build_spot: rounded_rect_texture(
                48,
                Color::new(0.3, 0.35, 0.3, 0.55),
                Color::new(0.9, 0.9, 0.9, 0.8),
            ),
            effect_slow: circle_texture(
                16,
                Color::new(0.35, 0.7, 1.0, 1.0),
                Color::new(0.1, 0.3, 0.6, 1.0),
            ),
            effect_poison: circle_texture(
                16,
                Color::new(0.4, 0.9, 0.2, 1.0),
                Color::new(0.1, 0.4, 0.05, 1.0),
            ),
            effect_lava: circle_texture(
                16,
                Color::new(0.9, 0.4, 0.2, 1.0),
                Color::new(0.4, 0.2, 0.05, 1.0),
            ),
            lava_pool: glow_texture(
                64,
                Color::new(1.0, 0.85, 0.2, 0.95),
                Color::new(0.85, 0.15, 0.0, 0.0),
            ),
            title_background: None,
        }
    }
}

/// Loads an enemy texture from `assets/enemy/{name}.png`, panicking with
/// a clear message if it's missing (same failure style as the tileset
/// load in `main.rs`).
async fn load_enemy_texture(name: &str) -> Texture2D {
    let path = enemy_texture_path(name);
    try_load_texture(&path)
        .await
        .unwrap_or_else(|| panic!("failed to load enemy texture '{path}'"))
}

/// Like [`load_enemy_texture`], but returns `None` instead of panicking when
/// the art does not exist yet, so the caller can fall back to a placeholder.
async fn try_load_enemy_texture(name: &str) -> Option<Texture2D> {
    try_load_texture(&enemy_texture_path(name)).await
}

/// Loads the icon for `effect` from `assets/effects/{name}.png`, falling
/// back to the generated placeholder while the art is missing.
async fn load_effect_texture(effect: EffectType, placeholder: &Texture2D) -> Texture2D {
    let path = format!("assets/effects/{}.png", effect.icon_name());
    try_load_texture(&path)
        .await
        .unwrap_or_else(|| placeholder.clone())
}

/// Loads the projectile for `kind` from `assets/projectiles/{name}.png`,
/// falling back to the generated placeholder while the art is missing.
async fn load_projectile_texture(kind: TowerType, placeholder: &Texture2D) -> Texture2D {
    let path = format!("assets/projectiles/{}.png", kind.projectile_name());
    try_load_texture(&path)
        .await
        .unwrap_or_else(|| placeholder.clone())
}

fn enemy_texture_path(name: &str) -> String {
    format!("assets/enemy/{name}.png")
}

async fn try_load_texture(path: &str) -> Option<Texture2D> {
    let texture = load_texture(path).await.ok()?;
    texture.set_filter(FilterMode::Linear);
    Some(texture)
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

/// Soft round blob blending from `center` in the middle to `edge` at the
/// rim, used for glowing ground effects like lava pools.
fn glow_texture(size: u16, center: Color, edge: Color) -> Texture2D {
    let mut image = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));
    let mid = size as f32 / 2.0;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - mid;
            let dy = y as f32 + 0.5 - mid;
            let t = (dx * dx + dy * dy).sqrt() / mid;
            if t <= 1.0 {
                let lerp = |a: f32, b: f32| a + (b - a) * t;
                image.set_pixel(
                    x as u32,
                    y as u32,
                    Color::new(
                        lerp(center.r, edge.r),
                        lerp(center.g, edge.g),
                        lerp(center.b, edge.b),
                        lerp(center.a, edge.a),
                    ),
                );
            }
        }
    }
    Texture2D::from_image(&image)
}

/// Lumpy stone: a circle with a wobbly radius, outlined, with a light
/// highlight in the upper-left so spinning is visible.
fn rock_texture(size: u16, fill: Color, outline: Color) -> Texture2D {
    let mut image = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));
    let mid = size as f32 / 2.0;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - mid;
            let dy = y as f32 + 0.5 - mid;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);
            let radius =
                (mid - 1.0) * (0.82 + 0.1 * (3.0 * angle).sin() + 0.06 * (5.0 * angle).cos());
            if dist > radius {
                continue;
            }
            let color = if dist >= radius - 2.0 {
                outline
            } else if (dx + mid * 0.3).powi(2) + (dy + mid * 0.3).powi(2) < (mid * 0.25).powi(2) {
                Color::new(0.88, 0.87, 0.84, 1.0)
            } else {
                fill
            };
            image.set_pixel(x as u32, y as u32, color);
        }
    }
    Texture2D::from_image(&image)
}

/// Crystal chunk: an outlined diamond with a bright facet.
fn diamond_texture(size: u16, fill: Color, outline: Color) -> Texture2D {
    let mut image = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));
    let mid = size as f32 / 2.0;
    let radius = mid - 1.0;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - mid;
            let dy = y as f32 + 0.5 - mid;
            let d = dx.abs() + dy.abs();
            if d > radius {
                continue;
            }
            let color = if d >= radius - 2.5 {
                outline
            } else if dx < 0.0 && dy < 0.0 {
                WHITE
            } else {
                fill
            };
            image.set_pixel(x as u32, y as u32, color);
        }
    }
    Texture2D::from_image(&image)
}

/// Glowing ball: a solid `core` with a bright rim, surrounded by a
/// `halo` fading out to transparent.
fn orb_texture(size: u16, core: Color, halo: Color) -> Texture2D {
    let mut image = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));
    let mid = size as f32 / 2.0;
    let core_t = 0.55;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - mid;
            let dy = y as f32 + 0.5 - mid;
            let t = (dx * dx + dy * dy).sqrt() / mid;
            if t > 1.0 {
                continue;
            }
            let color = if t <= core_t {
                // Lighter toward the middle, blending into the halo at the rim.
                let s = t / core_t;
                let mix = |c: f32, h: f32| c + (h - c) * s * s;
                Color::new(
                    mix(core.r, halo.r),
                    mix(core.g, halo.g),
                    mix(core.b, halo.b),
                    1.0,
                )
            } else {
                let fade = 1.0 - (t - core_t) / (1.0 - core_t);
                Color::new(halo.r, halo.g, halo.b, halo.a * fade * fade)
            };
            image.set_pixel(x as u32, y as u32, color);
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
