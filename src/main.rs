//! Snail Wars - a simple tower defense game.
//!
//! Snails and slugs crawl along a fixed garden path; the player places
//! towers alongside it to stop them before they reach the base.

mod app;
mod catalog;
mod enemy;
mod game;
mod level;
mod map;
mod progress;
mod projectile;
mod sprites;
mod tower;
mod ui;
mod viewport;
mod wave;

use app::App;
use macroquad::prelude::*;

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
    App::new().await.run().await;
}
