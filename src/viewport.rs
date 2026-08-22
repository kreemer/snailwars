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
//!   (see [`crate::game::Game::draw_world`]). Scaled uniformly to fit
//!   the window (preserving aspect ratio, i.e. letterboxed) times the
//!   current mouse-wheel zoom.
//! - The **UI** layer: HUD, panel, and any full-screen overlays (see
//!   [`crate::game::Game::draw_ui`]), drawn with a transparent
//!   background everywhere except its own opaque bars. Stretched
//!   (non-uniformly, independently in x and y) to exactly cover the
//!   real window - so it always spans the full window width/height and
//!   resizes immediately as the window is resized, on top of the
//!   world layer. Zoom never applies to it.
//!
//! Based on macroquad's `letterbox` example pattern.

use crate::map::{SCREEN_W, WINDOW_H};
use macroquad::prelude::*;

const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 2.5;
/// How much one mouse-wheel notch changes the zoom multiplier by.
const ZOOM_STEP: f32 = 0.1;

pub struct Viewport {
    world_target: RenderTarget,
    world_camera: Camera2D,
    ui_target: RenderTarget,
    ui_camera: Camera2D,
    zoom: f32,
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
        }
    }

    /// Read mouse-wheel input and update the zoom multiplier. Call once
    /// per frame, before using [`Self::to_world_logical`] or
    /// [`Self::present`].
    pub fn handle_zoom_input(&mut self) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            self.zoom = (self.zoom + wheel_y.signum() * ZOOM_STEP).clamp(MIN_ZOOM, MAX_ZOOM);
        }
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
    /// ratio (i.e. letterboxed), *before* applying zoom. Used for the
    /// world layer only.
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
    /// coordinates, accounting for the current fit scale *and* zoom -
    /// use this for anything that should follow the zoomed map (e.g.
    /// placing towers).
    pub fn to_world_logical(&self, screen_pos: Vec2) -> Vec2 {
        let scale = self.world_fit_scale() * self.zoom;
        (screen_pos - self.world_offset(scale)) / scale
    }

    /// Convert a real window/mouse position into logical UI coordinates,
    /// accounting for the UI layer's stretch scale - use this for
    /// hit-testing HUD/panel buttons, which always span the full window.
    pub fn to_ui_logical(&self, screen_pos: Vec2) -> Vec2 {
        screen_pos / self.ui_scale()
    }

    /// Switch back to the real window and composite both layers onto it:
    /// the world layer (uniformly scaled by fit x zoom, letterboxed),
    /// then the UI layer on top (stretched to fill the entire window),
    /// so the UI always spans the full window and resizes with it,
    /// regardless of the map's zoom level.
    pub fn present(&self) {
        set_default_camera();
        clear_background(BLACK);

        let world_scale = self.world_fit_scale() * self.zoom;
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
