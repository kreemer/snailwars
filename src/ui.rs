//! HUD, tower-selection panel, and overlay screens (start / game over / win).

use crate::map::{PANEL_HEIGHT, SCREEN_W, TOP_BAR, WINDOW_H};
use crate::tower::TowerType;
use macroquad::prelude::*;

const TOWER_TYPES: [TowerType; 4] = [
    TowerType::Pebble,
    TowerType::Pepper,
    TowerType::Salt,
    TowerType::CostDesTodes,
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
            TowerType::Pepper => &sprites.tower_pepper,
            TowerType::Salt => &sprites.tower_salt,
            TowerType::CostDesTodes => &sprites.tower_death,
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

pub fn draw_center_message(title: &str, subtitle: &str) {
    draw_rectangle(0.0, 0.0, SCREEN_W, WINDOW_H, Color::new(0.0, 0.0, 0.0, 0.6));
    let title_size = 48.0;
    let title_dims = measure_text(title, None, title_size as u16, 1.0);
    draw_text(
        title,
        SCREEN_W / 2.0 - title_dims.width / 2.0,
        WINDOW_H / 2.0 - 10.0,
        title_size,
        WHITE,
    );
    let sub_size = 22.0;
    let sub_dims = measure_text(subtitle, None, sub_size as u16, 1.0);
    draw_text(
        subtitle,
        SCREEN_W / 2.0 - sub_dims.width / 2.0,
        WINDOW_H / 2.0 + 30.0,
        sub_size,
        Color::new(0.85, 0.85, 0.85, 1.0),
    );
}
