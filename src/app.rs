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

use crate::catalog::LevelCatalog;
use crate::game::{Game, GameStatus};
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
    tileset_textures: HashMap<PathBuf, Texture2D>,
    viewport: Viewport,
    screen: Screen,
}

impl App {
    pub async fn new() -> Self {
        App {
            catalog: LevelCatalog::load(),
            progress: Progress::load(),
            sprites: Sprites::load().await,
            tileset_textures: HashMap::new(),
            viewport: Viewport::new(),
            screen: Screen::Title,
        }
    }

    /// Run the game until the player quits.
    pub async fn run(&mut self) {
        loop {
            let dt = get_frame_time();
            let screen_mouse = Vec2::from(mouse_position());
            let ui_mouse = self.viewport.to_ui_logical(screen_mouse);
            let clicked = is_mouse_button_pressed(MouseButton::Left);

            let transition = match self.screen {
                Screen::Title => self.frame_title(ui_mouse, clicked),
                Screen::MapSelect => self.frame_map_select(ui_mouse, clicked),
                Screen::Playing(_) => self.frame_playing(dt, screen_mouse, ui_mouse, clicked),
            };

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

        if !clicked {
            return Transition::Stay;
        }
        for (action, rect) in ui::title_buttons() {
            if rect.contains(ui_mouse) {
                return match action {
                    TitleAction::Play => Transition::MapSelect,
                    TitleAction::Quit => Transition::Quit,
                };
            }
        }
        Transition::Stay
    }

    fn frame_map_select(&mut self, ui_mouse: Vec2, clicked: bool) -> Transition {
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

        if !clicked {
            return Transition::Stay;
        }
        if ui::map_select_back_rect().contains(ui_mouse) {
            return Transition::Title;
        }
        for (index, rect) in ui::map_select_card_rects(items.len())
            .into_iter()
            .enumerate()
        {
            if rect.contains(ui_mouse) && items[index].unlocked {
                return Transition::Play(index);
            }
        }
        Transition::Stay
    }

    fn frame_playing(
        &mut self,
        dt: f32,
        screen_mouse: Vec2,
        ui_mouse: Vec2,
        clicked: bool,
    ) -> Transition {
        let Screen::Playing(session) = &mut self.screen else {
            return Transition::Stay;
        };
        let game = &mut session.game;
        let level_index = session.level_index;

        let running = *game.status() == GameStatus::Playing;
        if running {
            self.viewport.handle_zoom_input();
            self.viewport.handle_scroll_input(dt);
        }
        let world_mouse = self.viewport.to_world_logical(screen_mouse);
        game.update(dt, ui_mouse, world_mouse);

        self.viewport.begin_world();
        game.draw_world(ui_mouse, world_mouse);
        self.viewport.begin_ui();
        game.draw_ui();

        if *game.status() == GameStatus::Playing {
            return if is_key_pressed(KeyCode::Escape) {
                Transition::Title
            } else {
                Transition::Stay
            };
        }

        let won = *game.status() == GameStatus::Win;
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

        if is_key_pressed(KeyCode::R) {
            return self.restart_level();
        }
        if !clicked {
            return Transition::Stay;
        }
        for (action, rect) in ui::end_overlay_buttons(&actions) {
            if rect.contains(ui_mouse) {
                return match action {
                    EndAction::NextLevel => Transition::Play(level_index + 1),
                    EndAction::Restart => self.restart_level(),
                    EndAction::BackToTitle => Transition::Title,
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
