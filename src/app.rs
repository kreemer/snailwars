//! The screen state machine wrapped around the actual gameplay: title
//! screen, map selection, and a running level.
//!
//! ```text
//! Title ──Play──▶ MapSelect ──pick unlocked level──▶ Playing
//!   ▲                 ▲                                 │
//!   └─────────────────┴────────── win / lose ───────────┘
//! ```
//!
//! Everything that outlives a single level lives here: the loaded
//! [`Sprites`], the [`LevelCatalog`], the player's [`Progress`], and a
//! cache of already-loaded tileset textures (levels are switchable, so
//! their textures can no longer be loaded once at startup).

use crate::audio::{AudioChannel, AudioEvent, AudioManager};
use crate::catalog::LevelCatalog;
use crate::game::{Game, GameStatus};
use crate::input::{Controls, Input};
use crate::level::Level;
use crate::progress::Progress;
use crate::sprites::Sprites;
use crate::ui::{self, EndAction, MapSelectItem, TitleAction};
use crate::viewport::Viewport;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A level currently being played, together with the progression state
/// the end-of-level overlay needs.
struct Session {
    game: Game,
    level_index: usize,
    /// Whether the level's win/lose result has already been applied to
    /// [`App::progress`], so it only happens once even though the
    /// overlay stays up for many frames.
    outcome_handled: bool,
}

enum Screen {
    Title,
    MapSelect,
    /// Boxed because a [`Session`] is far larger than the menu variants.
    Playing(Box<Session>),
}

/// What the current frame decided the next screen should be.
enum Transition {
    Stay,
    Title,
    MapSelect,
    Play(usize),
    Quit,
}

pub struct App {
    catalog: LevelCatalog,
    progress: Progress,
    sprites: Sprites,
    audio: AudioManager,
    input: Input,
    tileset_textures: HashMap<PathBuf, Texture2D>,
    viewport: Viewport,
    screen: Screen,
}

impl App {
    pub async fn new() -> Self {
        let font = load_ttf_font("assets/fonts/DejaVuSans.ttf")
            .await
            .unwrap_or_else(|e| {
                panic!("failed to load UI font 'assets/fonts/DejaVuSans.ttf': {e}")
            });
        set_default_font(font);

        App {
            catalog: LevelCatalog::load(),
            progress: Progress::load(),
            sprites: Sprites::load().await,
            audio: AudioManager::new().await,
            input: Input::new(),
            tileset_textures: HashMap::new(),
            viewport: Viewport::new(),
            screen: Screen::Title,
        }
    }

    /// Run the game until the player quits.
    pub async fn run(&mut self) {
        loop {
            let dt = get_frame_time();
            let controls = self.input.update(dt);
            let ui_mouse = self.viewport.to_ui_logical(controls.screen_mouse);

            let transition = match self.screen {
                Screen::Title => self.frame_title(ui_mouse, controls.clicked),
                Screen::MapSelect => self.frame_map_select(ui_mouse, &controls),
                Screen::Playing(_) => self.frame_playing(dt, ui_mouse, &controls),
            };

            if let Some(cursor) = controls.gamepad_cursor {
                ui::draw_gamepad_cursor(self.viewport.to_ui_logical(cursor));
            }
            self.viewport.present();
            next_frame().await;

            match transition {
                Transition::Stay => {}
                Transition::Title => self.screen = Screen::Title,
                Transition::MapSelect => self.screen = Screen::MapSelect,
                Transition::Play(index) => self.start_level(index).await,
                Transition::Quit => return,
            }
        }
    }

    fn frame_title(&mut self, ui_mouse: Vec2, clicked: bool) -> Transition {
        self.begin_menu_frame();
        ui::draw_title_screen(self.sprites.title_background.as_ref(), ui_mouse);
        ui::draw_audio_buttons(ui::title_audio_buttons(), &self.audio, ui_mouse, 22.0);

        if !clicked {
            return Transition::Stay;
        }
        if handle_audio_click(&mut self.audio, ui_mouse, ui::title_audio_buttons()) {
            return Transition::Stay;
        }
        for (action, rect) in ui::title_buttons() {
            if rect.contains(ui_mouse) {
                return match action {
                    TitleAction::Play => {
                        self.audio.play(AudioEvent::UiClick);
                        Transition::MapSelect
                    }
                    TitleAction::Quit => {
                        self.audio.play(AudioEvent::UiClick);
                        Transition::Quit
                    }
                };
            }
        }
        Transition::Stay
    }

    fn frame_map_select(&mut self, ui_mouse: Vec2, controls: &Controls) -> Transition {
        self.begin_menu_frame();

        let items: Vec<MapSelectItem<'_>> = self
            .catalog
            .levels()
            .iter()
            .enumerate()
            .map(|(index, entry)| MapSelectItem {
                name: &entry.name,
                unlocked: self.progress.is_unlocked(&self.catalog, index),
                completed: self.progress.is_completed(&entry.id),
            })
            .collect();
        ui::draw_map_select(&items, ui_mouse);

        if controls.cancelled {
            self.audio.play(AudioEvent::UiClick);
            return Transition::Title;
        }
        if !controls.clicked {
            return Transition::Stay;
        }
        if ui::map_select_back_rect().contains(ui_mouse) {
            self.audio.play(AudioEvent::UiClick);
            return Transition::Title;
        }
        for (index, rect) in ui::map_select_card_rects(items.len())
            .into_iter()
            .enumerate()
        {
            if rect.contains(ui_mouse) && items[index].unlocked {
                self.audio.play(AudioEvent::UiClick);
                return Transition::Play(index);
            }
        }
        Transition::Stay
    }

    fn frame_playing(&mut self, dt: f32, ui_mouse: Vec2, controls: &Controls) -> Transition {
        let Screen::Playing(session) = &mut self.screen else {
            return Transition::Stay;
        };
        let game = &mut session.game;
        let level_index = session.level_index;

        let running = *game.status() == GameStatus::Playing;
        let clicked = if running && controls.clicked {
            !handle_audio_click(&mut self.audio, ui_mouse, ui::hud_audio_buttons())
        } else {
            controls.clicked
        };
        if running {
            self.viewport.handle_zoom_input();
            self.viewport.handle_scroll_input(dt);
            self.viewport
                .handle_controller_input(dt, controls.camera_pan, controls.zoom_steps);
        }
        let world_mouse = self.viewport.to_world_logical(controls.screen_mouse);
        for event in game.update(dt, ui_mouse, world_mouse, clicked, controls.speed_pressed) {
            self.audio.play(event);
        }

        self.viewport.begin_world();
        game.draw_world(ui_mouse, world_mouse);
        self.viewport.begin_ui();
        game.draw_ui();
        if running {
            ui::draw_audio_buttons(ui::hud_audio_buttons(), &self.audio, ui_mouse, 15.0);
        }

        if *game.status() == GameStatus::Playing {
            return if is_key_pressed(KeyCode::Escape) || controls.cancelled {
                self.audio.stop_music();
                Transition::Title
            } else {
                Transition::Stay
            };
        }

        let won = *game.status() == GameStatus::Win;
        self.audio.stop_music();
        if !session.outcome_handled {
            session.outcome_handled = true;
            if won {
                let level_id = session.game.level_id().to_owned();
                self.progress.mark_completed(&level_id);
                self.progress.save();
            }
        }

        let has_next = won && level_index + 1 < self.catalog.len();
        let actions: Vec<EndAction> = if won {
            let mut actions = Vec::new();
            if has_next {
                actions.push(EndAction::NextLevel);
            }
            actions.push(EndAction::Restart);
            actions.push(EndAction::BackToTitle);
            actions
        } else {
            vec![EndAction::Restart, EndAction::BackToTitle]
        };

        let (title, subtitle) = if won {
            ("You Win!", "The garden is safe.")
        } else {
            ("Game Over", "The snails got through!")
        };
        ui::draw_end_overlay(title, subtitle, &actions, ui_mouse);

        if is_key_pressed(KeyCode::R) || controls.restart_pressed {
            return self.restart_level();
        }
        if controls.cancelled {
            return Transition::Title;
        }
        if !controls.clicked {
            return Transition::Stay;
        }
        for (action, rect) in ui::end_overlay_buttons(&actions) {
            if rect.contains(ui_mouse) {
                return match action {
                    EndAction::NextLevel => {
                        self.audio.play(AudioEvent::UiClick);
                        Transition::Play(level_index + 1)
                    }
                    EndAction::Restart => {
                        self.audio.play(AudioEvent::UiClick);
                        self.restart_level()
                    }
                    EndAction::BackToTitle => {
                        self.audio.play(AudioEvent::UiClick);
                        Transition::Title
                    }
                };
            }
        }
        Transition::Stay
    }

    /// Start the current level over, reusing its already-loaded
    /// textures (so no async reload is needed).
    fn restart_level(&mut self) -> Transition {
        if let Screen::Playing(session) = &mut self.screen {
            session.game.restart();
            session.outcome_handled = false;
            self.viewport.set_world_bounds(session.game.world_size());
            self.audio.start_music();
        }
        Transition::Stay
    }

    /// Prepare the frame for a menu screen: the world layer stays empty
    /// (menus draw entirely on the fixed UI layer, which is stretched
    /// to fill the window).
    fn begin_menu_frame(&self) {
        self.viewport.begin_world();
        clear_background(BLACK);
        self.viewport.begin_ui();
    }

    async fn start_level(&mut self, index: usize) {
        let entry = self
            .catalog
            .get(index)
            .unwrap_or_else(|| panic!("no level at catalog index {index}"))
            .clone();

        let level = Level::load(&entry.id);
        let image_paths: Vec<PathBuf> = level
            .tilesets
            .iter()
            .map(|tileset| tileset.image_path.clone())
            .collect();
        let mut textures = Vec::with_capacity(image_paths.len());
        for path in &image_paths {
            textures.push(self.tileset_texture(path).await);
        }

        let game = Game::load(&entry.id, level, textures, self.sprites.clone());
        self.viewport.set_world_bounds(game.world_size());
        self.audio.start_music();
        self.screen = Screen::Playing(Box::new(Session {
            game,
            level_index: index,
            outcome_handled: false,
        }));
    }

    /// Load a tileset image, reusing the already-loaded texture when the
    /// same image is used by another (or a previously played) level.
    async fn tileset_texture(&mut self, path: &Path) -> Texture2D {
        if let Some(texture) = self.tileset_textures.get(path) {
            return texture.clone();
        }
        let path_str = path.to_str().expect("tileset path must be valid UTF-8");
        let texture = load_texture(path_str)
            .await
            .unwrap_or_else(|e| panic!("failed to load tileset texture '{path_str}': {e}"));
        texture.set_filter(FilterMode::Nearest);
        self.tileset_textures
            .insert(path.to_owned(), texture.clone());
        texture
    }
}

fn handle_audio_click(
    audio: &mut AudioManager,
    ui_mouse: Vec2,
    buttons: [(AudioChannel, Rect); 2],
) -> bool {
    for (channel, rect) in buttons {
        if rect.contains(ui_mouse) {
            audio.toggle(channel);
            audio.play(AudioEvent::UiClick);
            return true;
        }
    }
    false
}
