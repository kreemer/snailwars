//! The ordered list of playable levels, loaded from
//! `assets/maps/levels.ron`.
//!
//! Each entry only names a level; the level's actual content still lives
//! in the `<id>.tmx` / `<id>.waves.ron` pair loaded by
//! [`crate::level::Level::load`]. The manifest order is also the
//! progression order used for unlocking (see [`crate::progress`]).

use serde::Deserialize;

const MANIFEST_PATH: &str = "assets/maps/levels.ron";

#[derive(Clone, Debug, Deserialize)]
pub struct LevelEntry {
    /// Base name of the level's files in `assets/maps/`.
    pub id: String,
    /// Human-readable name shown on the map selection screen.
    pub name: String,
}

#[derive(Deserialize)]
struct ManifestFile {
    levels: Vec<LevelEntry>,
}

/// All levels the game knows about, in progression order.
pub struct LevelCatalog {
    levels: Vec<LevelEntry>,
}

impl LevelCatalog {
    /// Load `assets/maps/levels.ron`.
    ///
    /// Panics with a descriptive message if the manifest is missing,
    /// malformed, empty, or contains duplicate ids - all authoring
    /// mistakes that should surface immediately at startup.
    pub fn load() -> Self {
        let text = std::fs::read_to_string(MANIFEST_PATH)
            .unwrap_or_else(|e| panic!("failed to read level manifest '{MANIFEST_PATH}': {e}"));
        let manifest: ManifestFile = ron::from_str(&text)
            .unwrap_or_else(|e| panic!("failed to parse level manifest '{MANIFEST_PATH}': {e}"));

        assert!(
            !manifest.levels.is_empty(),
            "level manifest '{MANIFEST_PATH}' lists no levels"
        );
        for (i, entry) in manifest.levels.iter().enumerate() {
            if let Some(dup) = manifest.levels[..i].iter().find(|e| e.id == entry.id) {
                panic!(
                    "level manifest '{MANIFEST_PATH}' lists duplicate level id '{}'",
                    dup.id
                );
            }
        }

        LevelCatalog {
            levels: manifest.levels,
        }
    }

    #[cfg(test)]
    pub fn from_entries(levels: Vec<LevelEntry>) -> Self {
        LevelCatalog { levels }
    }

    pub fn levels(&self) -> &[LevelEntry] {
        &self.levels
    }

    pub fn len(&self) -> usize {
        self.levels.len()
    }

    pub fn get(&self, index: usize) -> Option<&LevelEntry> {
        self.levels.get(index)
    }
}
