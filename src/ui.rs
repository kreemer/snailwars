//! HUD, tower-selection panel, and the menu screens (title, map
//! selection) plus the end-of-level overlay.
//!
//! Everything here draws in logical UI coordinates on the fixed UI
//! layer (see [`crate::viewport`]), so hit-testing must always use the
//! mouse position converted with
//! [`crate::viewport::Viewport::to_ui_logical`].

use crate::map::{PANEL_HEIGHT, SCREEN_W, TOP_BAR, WINDOW_H};
use crate::tower::{MAX_LEVEL, Tower, TowerType};
use macroquad::prelude::*;

const TOWER_TYPES: [TowerType; 6] = [
    TowerType::Pebble,
    TowerType::Laser,
    TowerType::Pepper,
    TowerType::Salt,
    TowerType::CostDesTodes,
    TowerType::Lava,
];
const BUTTON_SIZE: f32 = 84.0;
const BUTTON_GAP: f32 = 16.0;

pub fn draw_hud(gold: u32, lives: u32, wave_number: u32, total_waves: u32) {
    draw_rectangle(
        0.0,
        0.0,
        SCREEN_W,
        TOP_BAR,
        Color::new(0.08, 0.08, 0.1, 1.0),
    );
    draw_text(format!("Gold: {gold}"), 12.0, 26.0, 26.0, GOLD);
    draw_text(
        format!("Lives: {lives}"),
        180.0,
        26.0,
        26.0,
        Color::new(1.0, 0.4, 0.4, 1.0),
    );
    draw_text(
        format!("Wave: {wave_number}/{total_waves}"),
        340.0,
        26.0,
        26.0,
        WHITE,
    );
}

/// Draw the virtual cursor used for gamepad navigation.
pub fn draw_gamepad_cursor(pos: Vec2) {
    draw_circle(pos.x, pos.y, 9.0, Color::new(0.05, 0.05, 0.05, 0.8));
    draw_circle_lines(pos.x, pos.y, 8.0, 2.0, WHITE);
    draw_circle(pos.x, pos.y, 2.5, Color::new(0.7, 1.0, 0.55, 1.0));
}

/// Rects for each tower button, in the same order as `TOWER_TYPES`.
pub fn tower_button_rects() -> Vec<(TowerType, Rect)> {
    let panel_y = WINDOW_H - PANEL_HEIGHT;
    let start_x = 16.0;
    TOWER_TYPES
        .iter()
        .enumerate()
        .map(|(i, &kind)| {
            let x = start_x + i as f32 * (BUTTON_SIZE + BUTTON_GAP);
            let y = panel_y + (PANEL_HEIGHT - BUTTON_SIZE) / 2.0;
            (kind, Rect::new(x, y, BUTTON_SIZE, BUTTON_SIZE))
        })
        .collect()
}

pub fn start_wave_button_rect() -> Rect {
    let panel_y = WINDOW_H - PANEL_HEIGHT;
    Rect::new(
        SCREEN_W - 190.0,
        panel_y + (PANEL_HEIGHT - 50.0) / 2.0,
        170.0,
        50.0,
    )
}

/// Button that cycles through simulation speed multipliers (1x/2x/3x).
pub fn speed_button_rect() -> Rect {
    let panel_y = WINDOW_H - PANEL_HEIGHT;
    Rect::new(
        SCREEN_W - 270.0,
        panel_y + (PANEL_HEIGHT - 50.0) / 2.0,
        70.0,
        50.0,
    )
}

pub fn draw_panel(
    gold: u32,
    selected: Option<TowerType>,
    sprites: &crate::sprites::Sprites,
    wave_active: bool,
    speed_multiplier: u32,
) {
    let panel_y = WINDOW_H - PANEL_HEIGHT;
    draw_rectangle(
        0.0,
        panel_y,
        SCREEN_W,
        PANEL_HEIGHT,
        Color::new(0.12, 0.1, 0.08, 1.0),
    );

    for (kind, rect) in tower_button_rects() {
        let affordable = gold >= kind.cost();
        let is_selected = selected == Some(kind);
        let bg = if is_selected {
            Color::new(0.3, 0.55, 0.3, 1.0)
        } else if affordable {
            Color::new(0.22, 0.22, 0.26, 1.0)
        } else {
            Color::new(0.15, 0.15, 0.15, 1.0)
        };
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, WHITE);

        let tex = match kind {
            TowerType::Pebble => &sprites.tower_pebble,
            TowerType::Laser => &sprites.tower_laser,
            TowerType::Pepper => &sprites.tower_pepper,
            TowerType::Salt => &sprites.tower_salt,
            TowerType::CostDesTodes => &sprites.tower_death,
            TowerType::Lava => &sprites.tower_lava,
        };
        let icon_size = 40.0;
        draw_texture_ex(
            tex,
            rect.x + (rect.w - icon_size) / 2.0,
            rect.y + 4.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(icon_size, icon_size)),
                ..Default::default()
            },
        );
        draw_text(
            format!("{}g", kind.cost()),
            rect.x + 6.0,
            rect.y + rect.h - 6.0,
            16.0,
            WHITE,
        );

        if is_selected {
            let name_dims = measure_text(kind.name(), None, 16, 1.0);
            draw_text(
                kind.name(),
                rect.x + rect.w / 2.0 - name_dims.width / 2.0,
                rect.y - 8.0,
                16.0,
                Color::new(1.0, 1.0, 0.8, 1.0),
            );
        }
    }

    // Speed toggle button.
    let speed_rect = speed_button_rect();
    draw_rectangle(
        speed_rect.x,
        speed_rect.y,
        speed_rect.w,
        speed_rect.h,
        Color::new(0.25, 0.25, 0.35, 1.0),
    );
    draw_rectangle_lines(
        speed_rect.x,
        speed_rect.y,
        speed_rect.w,
        speed_rect.h,
        2.0,
        WHITE,
    );
    let speed_label = format!("{speed_multiplier}x");
    let speed_dims = measure_text(&speed_label, None, 22, 1.0);
    draw_text(
        &speed_label,
        speed_rect.x + speed_rect.w / 2.0 - speed_dims.width / 2.0,
        speed_rect.y + speed_rect.h / 2.0 + 8.0,
        22.0,
        WHITE,
    );

    // Start wave button.
    let sw_rect = start_wave_button_rect();
    let (label, color) = if wave_active {
        ("Wave in progress...", Color::new(0.3, 0.3, 0.3, 1.0))
    } else {
        ("Start Wave", Color::new(0.2, 0.5, 0.2, 1.0))
    };
    draw_rectangle(sw_rect.x, sw_rect.y, sw_rect.w, sw_rect.h, color);
    draw_rectangle_lines(sw_rect.x, sw_rect.y, sw_rect.w, sw_rect.h, 2.0, WHITE);
    draw_text(
        label,
        sw_rect.x + 10.0,
        sw_rect.y + sw_rect.h / 2.0 + 6.0,
        18.0,
        WHITE,
    );
}

// --- Placed tower menu ----------------------------------------------
//
// Unlike the rest of this module, the tower menu lives in *world*
// coordinates: it is drawn by [`crate::game::Game::draw_world`] next to
// the tower it belongs to (so it follows scroll/zoom) and must be
// hit-tested with the world-space mouse position.

const TOWER_MENU_SIZE: Vec2 = vec2(160.0, 108.0);
const TOWER_MENU_BUTTON_H: f32 = 30.0;
const TOWER_MENU_PADDING: f32 = 8.0;
/// Gap between the tower's center and the nearest menu edge.
const TOWER_MENU_OFFSET: f32 = 28.0;

/// World-space rects of the menu opened on a placed tower.
pub struct TowerMenuLayout {
    pub panel: Rect,
    pub upgrade: Rect,
    pub sell: Rect,
}

/// Lay the menu out above the tower at `tower_pos`, or below it when
/// there is no room between the tower and the top of the map.
pub fn tower_menu_layout(tower_pos: Vec2) -> TowerMenuLayout {
    let x = tower_pos.x - TOWER_MENU_SIZE.x / 2.0;
    let above = tower_pos.y - TOWER_MENU_OFFSET - TOWER_MENU_SIZE.y;
    let y = if above >= TOP_BAR {
        above
    } else {
        tower_pos.y + TOWER_MENU_OFFSET
    };
    let panel = Rect::new(x, y, TOWER_MENU_SIZE.x, TOWER_MENU_SIZE.y);
    let button_w = panel.w - 2.0 * TOWER_MENU_PADDING;
    let button_x = panel.x + TOWER_MENU_PADDING;
    let sell_y = panel.y + panel.h - TOWER_MENU_PADDING - TOWER_MENU_BUTTON_H;
    let upgrade_y = sell_y - TOWER_MENU_PADDING / 2.0 - TOWER_MENU_BUTTON_H;
    TowerMenuLayout {
        panel,
        upgrade: Rect::new(button_x, upgrade_y, button_w, TOWER_MENU_BUTTON_H),
        sell: Rect::new(button_x, sell_y, button_w, TOWER_MENU_BUTTON_H),
    }
}

/// Draw the Sell/Upgrade menu for `tower`. `world_mouse` is the mouse in
/// world coordinates, used for hover highlighting.
pub fn draw_tower_menu(tower: &Tower, gold: u32, world_mouse: Vec2) {
    let layout = tower_menu_layout(tower.pos);
    let panel = layout.panel;
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::new(0.1, 0.1, 0.12, 0.92),
    );
    draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 2.0, WHITE);

    draw_text(
        tower.kind.name(),
        panel.x + TOWER_MENU_PADDING,
        panel.y + 18.0,
        18.0,
        Color::new(1.0, 1.0, 0.8, 1.0),
    );
    draw_text(
        format!("Level {}/{MAX_LEVEL}", tower.level),
        panel.x + TOWER_MENU_PADDING,
        panel.y + 34.0,
        16.0,
        GOLD,
    );

    let (upgrade_label, upgrade_enabled) = match tower.upgrade_cost() {
        Some(cost) => (format!("Upgrade ({cost}g)"), gold >= cost),
        None => ("Max level".to_owned(), false),
    };
    draw_button_sized(
        layout.upgrade,
        &upgrade_label,
        upgrade_enabled,
        upgrade_enabled && layout.upgrade.contains(world_mouse),
        18.0,
    );
    draw_button_sized(
        layout.sell,
        &format!("Sell (+{}g)", tower.sell_value()),
        true,
        layout.sell.contains(world_mouse),
        18.0,
    );
}

fn draw_dim_overlay() {
    draw_rectangle(0.0, 0.0, SCREEN_W, WINDOW_H, Color::new(0.0, 0.0, 0.0, 0.6));
}

fn draw_centered_title(title: &str, subtitle: &str) {
    let title_size = 48.0;
    let title_dims = measure_text(title, None, title_size as u16, 1.0);
    draw_text(
        title,
        SCREEN_W / 2.0 - title_dims.width / 2.0,
        WINDOW_H / 2.0 - 60.0,
        title_size,
        WHITE,
    );
    let sub_size = 22.0;
    let sub_dims = measure_text(subtitle, None, sub_size as u16, 1.0);
    draw_text(
        subtitle,
        SCREEN_W / 2.0 - sub_dims.width / 2.0,
        WINDOW_H / 2.0 - 20.0,
        sub_size,
        Color::new(0.85, 0.85, 0.85, 1.0),
    );
}

/// Draw a labelled menu button. Disabled buttons are dimmed and are
/// never reported as hovered by the screens below.
fn draw_button(rect: Rect, label: &str, enabled: bool, hovered: bool) {
    draw_button_sized(rect, label, enabled, hovered, 24.0);
}

fn draw_button_sized(rect: Rect, label: &str, enabled: bool, hovered: bool, size: f32) {
    let fill = match (enabled, hovered) {
        (false, _) => Color::new(0.15, 0.15, 0.17, 0.9),
        (true, true) => Color::new(0.30, 0.55, 0.30, 0.95),
        (true, false) => Color::new(0.20, 0.22, 0.26, 0.95),
    };
    let text_color = if enabled {
        WHITE
    } else {
        Color::new(0.5, 0.5, 0.5, 1.0)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, text_color);
    let dims = measure_text(label, None, size as u16, 1.0);
    draw_text(
        label,
        rect.x + rect.w / 2.0 - dims.width / 2.0,
        rect.y + rect.h / 2.0 + dims.height / 2.0,
        size,
        text_color,
    );
}

/// Draw `texture` scaled to cover the whole logical canvas, preserving
/// its aspect ratio (cropping the overflowing axis by centering it).
fn draw_background_cover(texture: &Texture2D) {
    let tex_size = vec2(texture.width(), texture.height());
    let scale = f32::max(SCREEN_W / tex_size.x, WINDOW_H / tex_size.y);
    let dest = tex_size * scale;
    draw_texture_ex(
        texture,
        (SCREEN_W - dest.x) / 2.0,
        (WINDOW_H - dest.y) / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(dest),
            ..Default::default()
        },
    );
}

// --- Title screen ---------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TitleAction {
    Play,
    Quit,
}

const MENU_BUTTON_SIZE: Vec2 = vec2(240.0, 60.0);

pub fn title_buttons() -> Vec<(TitleAction, Rect)> {
    let x = (SCREEN_W - MENU_BUTTON_SIZE.x) / 2.0;
    [TitleAction::Play, TitleAction::Quit]
        .into_iter()
        .enumerate()
        .map(|(i, action)| {
            let y = 420.0 + i as f32 * (MENU_BUTTON_SIZE.y + 20.0);
            (
                action,
                Rect::new(x, y, MENU_BUTTON_SIZE.x, MENU_BUTTON_SIZE.y),
            )
        })
        .collect()
}

/// `background` is the optional title art (see
/// [`crate::sprites::Sprites::title_background`]); a plain backdrop is
/// painted while it is missing.
pub fn draw_title_screen(background: Option<&Texture2D>, ui_mouse: Vec2) {
    match background {
        Some(texture) => draw_background_cover(texture),
        None => draw_rectangle(
            0.0,
            0.0,
            SCREEN_W,
            WINDOW_H,
            Color::new(0.09, 0.14, 0.09, 1.0),
        ),
    }

    let title = "Snail Wars";
    let title_size = 84.0;
    let title_dims = measure_text(title, None, title_size as u16, 1.0);
    let title_x = SCREEN_W / 2.0 - title_dims.width / 2.0;
    draw_text(
        title,
        title_x + 4.0,
        254.0,
        title_size,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    draw_text(
        title,
        title_x,
        250.0,
        title_size,
        Color::new(1.0, 0.95, 0.75, 1.0),
    );

    for (action, rect) in title_buttons() {
        let label = match action {
            TitleAction::Play => "Play",
            TitleAction::Quit => "Quit",
        };
        draw_button(rect, label, true, rect.contains(ui_mouse));
    }
}

// --- Map selection --------------------------------------------------

/// One level as shown on the map selection screen.
pub struct MapSelectItem<'a> {
    pub name: &'a str,
    pub unlocked: bool,
    pub completed: bool,
}

const CARD_SIZE: Vec2 = vec2(430.0, 80.0);
const CARD_GAP: f32 = 20.0;
const CARD_COLUMNS: usize = 2;
const CARD_TOP: f32 = 180.0;

/// Rects for the level cards, in catalog order, laid out as a
/// [`CARD_COLUMNS`]-wide grid.
pub fn map_select_card_rects(count: usize) -> Vec<Rect> {
    let grid_width = CARD_COLUMNS as f32 * CARD_SIZE.x + (CARD_COLUMNS as f32 - 1.0) * CARD_GAP;
    let left = (SCREEN_W - grid_width) / 2.0;
    (0..count)
        .map(|i| {
            let col = (i % CARD_COLUMNS) as f32;
            let row = (i / CARD_COLUMNS) as f32;
            Rect::new(
                left + col * (CARD_SIZE.x + CARD_GAP),
                CARD_TOP + row * (CARD_SIZE.y + CARD_GAP),
                CARD_SIZE.x,
                CARD_SIZE.y,
            )
        })
        .collect()
}

pub fn map_select_back_rect() -> Rect {
    Rect::new(40.0, WINDOW_H - 100.0, 180.0, 54.0)
}

pub fn draw_map_select(items: &[MapSelectItem<'_>], ui_mouse: Vec2) {
    draw_rectangle(
        0.0,
        0.0,
        SCREEN_W,
        WINDOW_H,
        Color::new(0.07, 0.09, 0.12, 1.0),
    );

    let heading = "Select a Map";
    let heading_size = 52.0;
    let heading_dims = measure_text(heading, None, heading_size as u16, 1.0);
    draw_text(
        heading,
        SCREEN_W / 2.0 - heading_dims.width / 2.0,
        110.0,
        heading_size,
        Color::new(1.0, 0.95, 0.75, 1.0),
    );

    for (item, rect) in items.iter().zip(map_select_card_rects(items.len())) {
        let hovered = item.unlocked && rect.contains(ui_mouse);
        draw_button(rect, "", item.unlocked, hovered);

        let name_color = if item.unlocked {
            WHITE
        } else {
            Color::new(0.5, 0.5, 0.5, 1.0)
        };
        draw_text(item.name, rect.x + 20.0, rect.y + 36.0, 26.0, name_color);

        let (status, status_color) = match (item.unlocked, item.completed) {
            (_, true) => ("Cleared", Color::new(0.5, 0.9, 0.5, 1.0)),
            (true, false) => ("Available", Color::new(0.8, 0.8, 0.8, 1.0)),
            (false, false) => (
                "Locked - beat the previous map",
                Color::new(0.6, 0.6, 0.6, 1.0),
            ),
        };
        draw_text(status, rect.x + 20.0, rect.y + 64.0, 20.0, status_color);
    }

    let back = map_select_back_rect();
    draw_button(back, "Back", true, back.contains(ui_mouse));
}

// --- End-of-level overlay -------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndAction {
    NextLevel,
    Restart,
    BackToTitle,
}

impl EndAction {
    fn label(self) -> &'static str {
        match self {
            EndAction::NextLevel => "Next Level",
            EndAction::Restart => "Restart",
            EndAction::BackToTitle => "Back to Title",
        }
    }
}

/// Lay `actions` out as a centered row of buttons below the overlay
/// message.
pub fn end_overlay_buttons(actions: &[EndAction]) -> Vec<(EndAction, Rect)> {
    let width = 220.0;
    let gap = 24.0;
    let total = actions.len() as f32 * width + (actions.len() as f32 - 1.0).max(0.0) * gap;
    let left = (SCREEN_W - total) / 2.0;
    actions
        .iter()
        .enumerate()
        .map(|(i, &action)| {
            (
                action,
                Rect::new(
                    left + i as f32 * (width + gap),
                    WINDOW_H / 2.0 + 30.0,
                    width,
                    56.0,
                ),
            )
        })
        .collect()
}

pub fn draw_end_overlay(title: &str, subtitle: &str, actions: &[EndAction], ui_mouse: Vec2) {
    draw_dim_overlay();
    draw_centered_title(title, subtitle);
    for (action, rect) in end_overlay_buttons(actions) {
        draw_button(rect, action.label(), true, rect.contains(ui_mouse));
    }
}
