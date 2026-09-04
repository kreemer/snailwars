//! Persisted player progression: which levels have been beaten, and
//! therefore which are unlocked.
//!
//! Unlocking is linear along the [`crate::catalog::LevelCatalog`] order:
//! the first level is always available, and every later level opens up
//! once the one before it has been completed.
//!
//! Progress is stored in `save.ron` in the working directory (the same
//! place `assets/` is resolved from). A missing or unreadable save file
//! is treated as "nothing completed yet" rather than an error - losing
//! progress is preferable to refusing to start the game.

use crate::catalog::LevelCatalog;
use serde::{Deserialize, Serialize};

const SAVE_PATH: &str = "save.ron";

#[derive(Default, Serialize, Deserialize)]
pub struct Progress {
    /// Ids of every level the player has beaten, in no particular order.
    completed: Vec<String>,
}

impl Progress {
    /// Read `save.ron`, falling back to empty progress if it is missing
    /// or cannot be parsed.
    pub fn load() -> Self {
        let Ok(text) = std::fs::read_to_string(SAVE_PATH) else {
            return Progress::default();
        };
        ron::from_str(&text).unwrap_or_default()
    }

    /// Write `save.ron`. Failures are reported on stderr but never
    /// interrupt play.
    pub fn save(&self) {
        let text = match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("failed to serialize progress: {e}");
                return;
            }
        };
        if let Err(e) = std::fs::write(SAVE_PATH, text) {
            eprintln!("failed to write progress to '{SAVE_PATH}': {e}");
        }
    }

    pub fn is_completed(&self, id: &str) -> bool {
        self.completed.iter().any(|c| c == id)
    }

    /// Whether the level at `index` in `catalog` can be played: the
    /// first level always is, any later one needs its predecessor
    /// beaten.
    pub fn is_unlocked(&self, catalog: &LevelCatalog, index: usize) -> bool {
        match index.checked_sub(1) {
            None => true,
            Some(previous) => catalog
                .get(previous)
                .is_some_and(|entry| self.is_completed(&entry.id)),
        }
    }

    /// Record `id` as beaten. Call [`Self::save`] to persist it.
    pub fn mark_completed(&mut self, id: &str) {
        if !self.is_completed(id) {
            self.completed.push(id.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::LevelEntry;

    fn catalog(ids: &[&str]) -> LevelCatalog {
        LevelCatalog::from_entries(
            ids.iter()
                .map(|id| LevelEntry {
                    id: (*id).to_owned(),
                    name: (*id).to_owned(),
                })
                .collect(),
        )
    }

    fn progress(completed: &[&str]) -> Progress {
        Progress {
            completed: completed.iter().map(|c| (*c).to_owned()).collect(),
        }
    }

    #[test]
    fn first_level_is_always_unlocked() {
        let catalog = catalog(&["a", "b"]);
        assert!(progress(&[]).is_unlocked(&catalog, 0));
    }

    #[test]
    fn later_level_needs_its_predecessor() {
        let catalog = catalog(&["a", "b", "c"]);
        let progress = progress(&["a"]);
        assert!(progress.is_unlocked(&catalog, 1));
        assert!(!progress.is_unlocked(&catalog, 2));
    }

    #[test]
    fn completing_out_of_order_does_not_skip_ahead() {
        let catalog = catalog(&["a", "b", "c"]);
        assert!(!progress(&["c"]).is_unlocked(&catalog, 2));
    }

    #[test]
    fn marking_completed_is_idempotent() {
        let mut progress = progress(&["a"]);
        progress.mark_completed("a");
        progress.mark_completed("b");
        assert!(progress.is_completed("b"));
        assert_eq!(progress.completed.len(), 2);
    }
}
