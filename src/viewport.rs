//! Decouples the game's fixed *logical* resolution (see
//! [`crate::map::SCREEN_W`]/[`crate::map::WINDOW_H`]) from the actual,
//! freely resizable OS window, and keeps the zoom control limited to the
//! playable map - the HUD and tower-selection panel stay readable and
//! always fill the real window edge-to-edge, regardless of zoom.
//!
//! This is done with two separate offscreen render targets, both sized
//! `SCREEN_W x WINDOW_H` in logical pixels:
//!
//! - The **world** layer: the map, towers, enemies, projectiles, etc.
//!   (see [`crate::game::Game::draw_world`]). The world *camera* itself
//!   captures a larger or smaller slice of the map depending on zoom
//!   (zooming out reveals more of the map instead of just shrinking the
//!   rendered image), then that fixed-size texture is scaled uniformly
//!   to fit the window (preserving aspect ratio, i.e. letterboxed).
//! - The **UI** layer: HUD, panel, and any full-screen overlays (see
//!   [`crate::game::Game::draw_ui`]), drawn with a transparent
//!   background everywhere except its own opaque bars. Stretched
//!   (non-uniformly, independently in x and y) to exactly cover the
//!   real window - so it always spans the full window width/height and
//!   resizes immediately as the window is resized, on top of the
//!   world layer. Zoom never applies to it.
//!
//! Based on macroquad's `letterbox` example pattern.
//!
//! When a level's map is larger than the fixed `SCREEN_W x PLAY_H` play
//! area (or once zoomed out enough to reveal more than that), the world
//! camera also scrolls (via arrow keys) instead of the map being shrunk
//! to fit - see [`Self::handle_scroll_input`] and [`Self::set_world_bounds`].

use crate::map::{PLAY_H, SCREEN_W, TOP_BAR, WINDOW_H};
use macroquad::prelude::*;

const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 2.5;
/// How much one mouse-wheel notch changes the zoom multiplier by.
const ZOOM_STEP: f32 = 0.1;
/// Camera scroll speed, in logical (unzoomed) pixels per second, when
/// the map is larger than the visible play area.
const SCROLL_SPEED: f32 = 500.0;

pub struct Viewport {
    world_target: RenderTarget,
    world_camera: Camera2D,
    ui_target: RenderTarget,
    ui_camera: Camera2D,
    zoom: f32,
    /// Full map size in world pixels; see [`crate::map::Map::world_size`].
    world_size: Vec2,
    /// World position of the top-left corner of the visible *play area*
    /// (i.e. excluding the HUD/panel bars). Zero unless the map is
    /// larger than what's currently visible at the current zoom.
    scroll: Vec2,
    /// The world-space rect currently captured by the world camera
    /// (spanning the whole `SCREEN_W x WINDOW_H` canvas, HUD/panel bars
    /// included) - recomputed by [`Self::rebuild_world_camera`] whenever
    /// zoom or scroll changes, and used to convert mouse positions back
    /// into world coordinates.
    world_rect: Rect,
}

impl Viewport {
    pub fn new() -> Self {
        let world_target = render_target(SCREEN_W as u32, WINDOW_H as u32);
        world_target.texture.set_filter(FilterMode::Nearest);
        let mut world_camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_W, WINDOW_H));
        world_camera.render_target = Some(world_target.clone());

        let ui_target = render_target(SCREEN_W as u32, WINDOW_H as u32);
        ui_target.texture.set_filter(FilterMode::Nearest);
        let mut ui_camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_W, WINDOW_H));
        ui_camera.render_target = Some(ui_target.clone());

        Viewport {
            world_target,
            world_camera,
            ui_target,
            ui_camera,
            zoom: 1.0,
            world_size: vec2(SCREEN_W, PLAY_H),
            scroll: Vec2::ZERO,
            world_rect: Rect::new(0.0, 0.0, SCREEN_W, WINDOW_H),
        }
    }

    /// Tell the viewport the current level's full map size (in world
    /// pixels; see [`crate::map::Map::world_size`]), so it knows how far
    /// the camera is allowed to scroll. Call once after loading a level.
    /// Resets scroll back to the top-left corner of the map.
    pub fn set_world_bounds(&mut self, world_size: Vec2) {
        self.world_size = world_size;
        self.scroll = Vec2::ZERO;
        self.rebuild_world_camera();
    }

    /// Read mouse-wheel input and update the zoom multiplier - zooming
    /// out grows the world-space area the camera captures (revealing
    /// more of the map), zooming in shrinks it. The view stays centered
    /// on the same world point as zoom changes. Call once per frame,
    /// before using [`Self::to_world_logical`] or [`Self::present`].
    pub fn handle_zoom_input(&mut self) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            let center = self.scroll + self.visible_play_size() / 2.0;
            self.zoom = (self.zoom + wheel_y.signum() * ZOOM_STEP).clamp(MIN_ZOOM, MAX_ZOOM);
            self.scroll = center - self.visible_play_size() / 2.0;
            self.rebuild_world_camera();
        }
    }

    /// Read arrow-key input and pan the camera across a map larger than
    /// what's currently visible, clamped so the play area never shows
    /// past the map's edge. A no-op once the whole map already fits.
    /// Call once per frame, before drawing.
    pub fn handle_scroll_input(&mut self, dt: f32) {
        if self.max_scroll() == Vec2::ZERO {
            return;
        }

        let mut delta = Vec2::ZERO;
        if is_key_down(KeyCode::Left) {
            delta.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) {
            delta.x += 1.0;
        }
        if is_key_down(KeyCode::Up) {
            delta.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) {
            delta.y += 1.0;
        }
        if delta == Vec2::ZERO {
            return;
        }

        self.scroll += delta * SCROLL_SPEED * dt;
        self.rebuild_world_camera();
    }

    /// Size (in world pixels) of the play area currently visible at the
    /// current zoom level - larger than `(SCREEN_W, PLAY_H)` when zoomed
    /// out, smaller when zoomed in.
    fn visible_play_size(&self) -> Vec2 {
        vec2(SCREEN_W, PLAY_H) / self.zoom
    }

    /// Furthest `scroll` may go on each axis before the visible play
    /// area would show past the map's edge, given the current zoom.
    fn max_scroll(&self) -> Vec2 {
        (self.world_size - self.visible_play_size()).max(Vec2::ZERO)
    }

    /// Recompute the world-space rect the camera captures from the
    /// current `zoom`/`scroll`, clamp `scroll` to it, and rebuild the
    /// `Camera2D` display rect accordingly. Must be called whenever zoom
    /// or scroll changes, since `Camera2D` bakes its display rect in at
    /// construction time.
    fn rebuild_world_camera(&mut self) {
        self.scroll = self.scroll.clamp(Vec2::ZERO, self.max_scroll());

        let visible = self.visible_play_size();
        // The play area (screen rows TOP_BAR..TOP_BAR+PLAY_H) must map
        // to world y range [scroll.y, scroll.y + visible.y], so the
        // whole captured rect (which also includes the HUD/panel bar
        // rows, hidden behind the opaque UI layer) starts `TOP_BAR`
        // world-units (at this zoom) above that.
        self.world_rect = Rect::new(
            self.scroll.x,
            self.scroll.y - TOP_BAR / self.zoom,
            visible.x,
            WINDOW_H / self.zoom,
        );

        let mut world_camera = Camera2D::from_display_rect(self.world_rect);
        world_camera.render_target = Some(self.world_target.clone());
        self.world_camera = world_camera;
    }

    /// Switch drawing to the offscreen world (zoomable) layer. Call this
    /// before drawing the map/entities each frame.
    pub fn begin_world(&self) {
        set_camera(&self.world_camera);
    }

    /// Switch drawing to the offscreen UI (fixed, non-zoomable) layer.
    /// Call this before drawing the HUD/panel/overlays each frame.
    /// Clears to fully transparent, so the world layer shows through
    /// everywhere the UI doesn't paint its own opaque pixels.
    pub fn begin_ui(&self) {
        set_camera(&self.ui_camera);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
    }

    /// Scale factor from logical pixels to real window pixels needed to
    /// fit the logical resolution inside the window, preserving aspect
    /// ratio (i.e. letterboxed). Zoom is already baked into the world
    /// texture itself (see [`Self::rebuild_world_camera`]), so this is
    /// used unmodified for both the world and UI layers.
    fn world_fit_scale(&self) -> f32 {
        f32::min(screen_width() / SCREEN_W, screen_height() / WINDOW_H)
    }

    /// Top-left offset (in real window pixels) to center a logical
    /// canvas drawn at `scale`.
    fn world_offset(&self, scale: f32) -> Vec2 {
        vec2(
            (screen_width() - SCREEN_W * scale) * 0.5,
            (screen_height() - WINDOW_H * scale) * 0.5,
        )
    }

    /// Independent x/y scale factors that stretch the logical UI canvas
    /// to exactly cover the real window, however it's resized - no
    /// letterboxing, no offset.
    fn ui_scale(&self) -> Vec2 {
        vec2(screen_width() / SCREEN_W, screen_height() / WINDOW_H)
    }

    /// Convert a real window/mouse position into logical world
    /// coordinates, accounting for the current fit scale and the
    /// camera's zoomed/scrolled `world_rect` - use this for anything
    /// that should follow the zoomed/scrolled map (e.g. placing towers).
    pub fn to_world_logical(&self, screen_pos: Vec2) -> Vec2 {
        let scale = self.world_fit_scale();
        let local = (screen_pos - self.world_offset(scale)) / scale;
        vec2(self.world_rect.x, self.world_rect.y)
            + local * vec2(self.world_rect.w / SCREEN_W, self.world_rect.h / WINDOW_H)
    }

    /// Convert a real window/mouse position into logical UI coordinates,
    /// accounting for the UI layer's stretch scale - use this for
    /// hit-testing HUD/panel buttons, which always span the full window.
    pub fn to_ui_logical(&self, screen_pos: Vec2) -> Vec2 {
        screen_pos / self.ui_scale()
    }

    /// Switch back to the real window and composite both layers onto it:
    /// the world layer (uniformly scaled to fit, letterboxed - zoom is
    /// already baked into its captured content), then the UI layer on
    /// top (stretched to fill the entire window), so the UI always
    /// spans the full window and resizes with it, regardless of zoom.
    pub fn present(&self) {
        set_default_camera();
        clear_background(BLACK);

        let world_scale = self.world_fit_scale();
        draw_layer(
            &self.world_target,
            self.world_offset(world_scale),
            vec2(SCREEN_W, WINDOW_H) * world_scale,
        );

        draw_layer(
            &self.ui_target,
            Vec2::ZERO,
            vec2(screen_width(), screen_height()),
        );
    }
}

fn draw_layer(target: &RenderTarget, dest_pos: Vec2, dest_size: Vec2) {
    draw_texture_ex(
        &target.texture,
        dest_pos.x,
        dest_pos.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(dest_size),
            flip_y: true, // render targets are upside down otherwise
            ..Default::default()
        },
    );
}

