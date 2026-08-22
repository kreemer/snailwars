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
mod wave;

use game::Game;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Snail Wars".to_owned(),
        window_width: map::SCREEN_W as i32,
        window_height: map::WINDOW_H as i32,
        window_resizable: false,
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

    loop {
        let dt = get_frame_time();
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}
