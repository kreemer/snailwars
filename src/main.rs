//! Snail Wars - a simple tower defense game.
//!
//! Snails and slugs crawl along a fixed garden path; the player places
//! towers alongside it to stop them before they reach the base.

mod enemy;
mod game;
mod level;
mod map;
mod projectile;
mod sprites;
mod tower;
mod ui;
mod viewport;
mod wave;

use game::Game;
use macroquad::prelude::*;
use viewport::Viewport;

fn window_conf() -> Conf {
    Conf {
        window_title: "Snail Wars".to_owned(),
        window_width: map::SCREEN_W as i32,
        window_height: map::WINDOW_H as i32,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let level_name = "level1";
    let level = level::Level::load(level_name);
    let tileset_path = level.tileset_image_path.clone();
    let tileset = load_texture(tileset_path.to_str().expect("tileset path must be valid UTF-8"))
        .await
        .unwrap_or_else(|e| panic!("failed to load tileset texture '{}': {e}", tileset_path.display()));
    tileset.set_filter(FilterMode::Nearest);

    let mut game = Game::load(level_name, level, tileset);
    let mut viewport = Viewport::new();

    loop {
        let dt = get_frame_time();
        viewport.handle_zoom_input();
        let screen_mouse = Vec2::from(mouse_position());
        let ui_mouse = viewport.to_ui_logical(screen_mouse);
        let world_mouse = viewport.to_world_logical(screen_mouse);

        game.update(dt, ui_mouse, world_mouse);

        viewport.begin_world();
        game.draw_world(world_mouse);

        viewport.begin_ui();
        game.draw_ui();

        viewport.present();

        next_frame().await;
    }
}

