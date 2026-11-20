//! Persisted player progression: which levels have been beaten, and
//! therefore which are unlocked.
//!
//! Unlocking is linear along the [`crate::catalog::LevelCatalog`] order:
//! the first level is always available, and every later level opens up
//! once the one before it has been completed.
//!
//! Progress is stored under the user's XDG data directory. An existing
//! `save.ron` in the working directory is imported on first launch.

use crate::catalog::LevelCatalog;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const SAVE_FILE: &str = "save.ron";

fn data_home(xdg_data_home: Option<&OsStr>, home: Option<&OsStr>) -> Option<PathBuf> {
    xdg_data_home
        .map(Path::new)
        .filter(|path| path.is_absolute())
        .map(Path::to_path_buf)
        .or_else(|| {
            home.map(Path::new)
                .filter(|path| path.is_absolute())
                .map(|path| path.join(".local/share"))
        })
}

fn save_path() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_DATA_HOME");
    let home = std::env::var_os("HOME");
    let path = data_home(xdg.as_deref(), home.as_deref());
    if path.is_none() {
        eprintln!("cannot locate save file: set XDG_DATA_HOME or HOME to an absolute path");
    }
    path.map(|path| path.join("snailwars").join(SAVE_FILE))
}

#[derive(Default, Serialize, Deserialize)]
pub struct Progress {
    /// Ids of every level the player has beaten, in no particular order.
    completed: Vec<String>,
}

impl Progress {
    /// Read progress from XDG data home, importing a working-directory save
    /// if the new save does not exist yet.
    pub fn load() -> Self {
        let Some(path) = save_path() else {
            return Self::default();
        };
        Self::load_from_paths(&path, Path::new(SAVE_FILE))
    }

    fn load_from_paths(path: &Path, legacy_path: &Path) -> Self {
        let (text, migrate) = match std::fs::read_to_string(path) {
            Ok(text) => (text, false),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                match std::fs::read_to_string(legacy_path) {
                    Ok(text) => (text, true),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        return Self::default();
                    }
                    Err(error) => {
                        eprintln!("failed to read '{}': {error}", legacy_path.display());
                        return Self::default();
                    }
                }
            }
            Err(error) => {
                eprintln!("failed to read '{}': {error}", path.display());
                return Self::default();
            }
        };
        let progress: Self = match ron::from_str(&text) {
            Ok(progress) => progress,
            Err(error) => {
                let source = if migrate { legacy_path } else { path };
                eprintln!("failed to parse '{}': {error}", source.display());
                return Self::default();
            }
        };
        if migrate {
            progress.save_to_path(path);
        }
        progress
    }

    /// Write progress to XDG data home. Failures are reported on stderr.
    pub fn save(&self) {
        if let Some(path) = save_path() {
            self.save_to_path(&path);
        }
    }

    fn save_to_path(&self, path: &Path) {
        let text = match ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("failed to serialize progress: {e}");
                return;
            }
        };
        let directory = path.parent().expect("save path has a parent");
        if let Err(error) = std::fs::create_dir_all(directory) {
            eprintln!(
                "failed to create save directory '{}': {error}",
                directory.display()
            );
            return;
        }
        if let Err(error) = std::fs::write(path, text) {
            eprintln!("failed to write progress to '{}': {error}", path.display());
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

    #[test]
    fn xdg_data_home_takes_precedence_over_home() {
        assert_eq!(
            data_home(Some(OsStr::new("/data")), Some(OsStr::new("/home/user"))),
            Some(PathBuf::from("/data"))
        );
        assert_eq!(
            data_home(Some(OsStr::new("relative")), Some(OsStr::new("/home/user"))),
            Some(PathBuf::from("/home/user/.local/share"))
        );
        assert_eq!(data_home(None, None), None);
    }

    #[test]
    fn legacy_save_is_imported_without_overwriting_newer_progress() {
        let dir = std::env::temp_dir().join(format!("snailwars-progress-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let legacy = dir.join("save.ron");
        let new = dir.join("data/snailwars/save.ron");
        std::fs::write(&legacy, "(completed: [\"old\"])").unwrap();

        assert!(Progress::load_from_paths(&new, &legacy).is_completed("old"));
        assert!(new.exists());
        std::fs::write(&legacy, "(completed: [\"changed\"])").unwrap();
        assert!(Progress::load_from_paths(&new, &legacy).is_completed("old"));
        assert!(!Progress::load_from_paths(&new, &legacy).is_completed("changed"));

        std::fs::remove_dir_all(dir).unwrap();
    }

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
