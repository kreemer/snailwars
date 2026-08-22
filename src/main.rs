//! Snail Wars - a simple tower defense game.
//!
//! Snails and slugs crawl along a fixed garden path; the player places
//! towers alongside it to stop them before they reach the base.

mod enemy;
mod game;
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
    let mut game = Game::new();

    loop {
        let dt = get_frame_time();
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}
