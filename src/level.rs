//! Loading a playable level from a hand-drawn Tiled map plus its
//! hand-authored wave file.
//!
//! A level is a pair of files living side by side under `assets/maps/`:
//! `<name>.tmx` (the Tiled map) and `<name>.waves.ron` (the wave list). The
//! `.tmx` must contain two tile layers:
//!
//! - `Ground`: purely visual, painted with whatever tileset tiles look
//!   right (e.g. grass, path swatches).
//! - `Logic`: gameplay data. Each painted cell's tileset tile must carry a
//!   custom string property `kind` set to one of `path`, `build`, `start`
//!   or `end`. Exactly one `start` and one `end` cell must exist, and the
//!   `path` cells (plus start/end) must form a single unbranching chain
//!   from start to end - this is what the snails walk along. `build`
//!   cells mark spots where the player may place a tower.
//!
//! The `Logic` layer is typically hidden in Tiled (it's not meant to be
//! seen by the player) but its tile data is still read regardless of
//! layer visibility.

use crate::wave::{Wave, WaveFile};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// The meaning of a painted cell in the `Logic` layer, resolved from a
/// tileset tile's `kind` custom property.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TileKind {
    Path,
    Build,
    Start,
    End,
}

/// A cell coordinate within the map grid, `(col, row)`, both 0-indexed.
pub type Cell = (i32, i32);

#[derive(Debug, PartialEq, Eq)]
pub enum PathError {
    NoStart,
    MultipleStart(Vec<Cell>),
    NoEnd,
    MultipleEnd(Vec<Cell>),
    /// A path cell has more than one unvisited path-like neighbor, i.e. the
    /// painted path branches instead of forming a single lane.
    Branch(Cell),
    /// The path chain from start didn't reach the end cell, or some
    /// painted path cells were never visited (disconnected from the main
    /// chain).
    Disconnected,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::NoStart => write!(f, "Logic layer has no cell with kind=start"),
            PathError::MultipleStart(cells) => {
                write!(f, "Logic layer has more than one kind=start cell: {cells:?}")
            }
            PathError::NoEnd => write!(f, "Logic layer has no cell with kind=end"),
            PathError::MultipleEnd(cells) => {
                write!(f, "Logic layer has more than one kind=end cell: {cells:?}")
            }
            PathError::Branch(cell) => write!(
                f,
                "path branches at {cell:?}: a path must be a single unbranching lane"
            ),
            PathError::Disconnected => write!(
                f,
                "path is disconnected: it doesn't form one unbroken lane from start to end"
            ),
        }
    }
}

impl std::error::Error for PathError {}

/// Trace the single unbranching chain of path-like cells from the unique
/// `start` cell to the unique `end` cell, following 4-directional
/// adjacency. Returns the ordered list of cells (including start and end)
/// that the snails should walk through.
///
/// This is a pure function over a plain grid of [`TileKind`]s, decoupled
/// from the `tiled` crate, so it can be unit tested directly.
pub fn trace_path(cells: &HashMap<Cell, TileKind>) -> Result<Vec<Cell>, PathError> {
    let starts: Vec<Cell> = cells
        .iter()
        .filter(|(_, k)| **k == TileKind::Start)
        .map(|(c, _)| *c)
        .collect();
    let ends: Vec<Cell> = cells
        .iter()
        .filter(|(_, k)| **k == TileKind::End)
        .map(|(c, _)| *c)
        .collect();

    let start = match starts.as_slice() {
        [] => return Err(PathError::NoStart),
        [only] => *only,
        _ => return Err(PathError::MultipleStart(starts)),
    };
    let end = match ends.as_slice() {
        [] => return Err(PathError::NoEnd),
        [only] => *only,
        _ => return Err(PathError::MultipleEnd(ends)),
    };

    let total_path_like = cells
        .values()
        .filter(|k| matches!(k, TileKind::Path | TileKind::Start | TileKind::End))
        .count();

    let mut ordered = vec![start];
    let mut prev = None;
    let mut current = start;

    while current != end {
        let next_candidates: Vec<Cell> = neighbors(current)
            .into_iter()
            .filter(|c| Some(*c) != prev)
            .filter(|c| matches!(cells.get(c), Some(TileKind::Path | TileKind::End)))
            .collect();

        match next_candidates.as_slice() {
            [] => return Err(PathError::Disconnected),
            [only] => {
                prev = Some(current);
                current = *only;
                ordered.push(current);
            }
            _ => return Err(PathError::Branch(current)),
        }

        if ordered.len() > total_path_like {
            // Safety net: we've visited more cells than exist, so there
            // must be a cycle rather than a simple chain.
            return Err(PathError::Disconnected);
        }
    }

    if ordered.len() != total_path_like {
        return Err(PathError::Disconnected);
    }

    Ok(ordered)
}

fn neighbors(cell: Cell) -> [Cell; 4] {
    let (x, y) = cell;
    [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
}

/// Everything needed to build the playable [`crate::map::Map`] for a
/// level, plus its hand-authored waves.
pub struct Level {
    pub waypoints: Vec<Vec2>,
    pub build_spots: Vec<Vec2>,
    /// The `Ground` layer's tile grid, `[row][col]`, holding each cell's
    /// local tile id within the tileset (or `None` for an empty cell).
    pub ground_tiles: Vec<Vec<Option<u32>>>,
    pub cols: usize,
    pub rows: usize,
    /// Size (in world/screen pixels) of one gameplay grid cell, i.e.
    /// [`crate::map::TILE`]. Waypoints and build spots are laid out in
    /// units of this size.
    pub tile_size: f32,
    /// Pixel size of one tile *inside the tileset source image*, read
    /// from the `.tmx`'s `tilewidth`/`tileheight`. This is independent of
    /// `tile_size`/[`crate::map::TILE`]: a 32px-tile tileset is scaled up
    /// (or a larger one scaled down) to fill each `tile_size` grid cell
    /// when drawn, so swapping tileset resolutions doesn't require the
    /// map's logical grid to change.
    pub source_tile_size: f32,
    /// Path to the tileset image, relative to the current working
    /// directory, to be loaded as a texture by the caller (texture loading
    /// is async in macroquad, so it can't happen inside this sync loader).
    pub tileset_image_path: PathBuf,
    pub waves: Vec<Wave>,
}

impl Level {
    /// Load `assets/maps/<name>.tmx` and `assets/maps/<name>.waves.ron`.
    ///
    /// Panics with a descriptive message on any structural problem with
    /// the map (missing layers, malformed path, bad wave file, etc.) -
    /// these are authoring mistakes that should be caught immediately
    /// rather than silently misbehave in-game.
    pub fn load(name: &str) -> Level {
        let tmx_path = format!("assets/maps/{name}.tmx");
        let waves_path = format!("assets/maps/{name}.waves.ron");

        let mut loader = tiled::Loader::new();
        let map = loader
            .load_tmx_map(&tmx_path)
            .unwrap_or_else(|e| panic!("failed to load map '{tmx_path}': {e}"));

        let logic_layer = map
            .layers()
            .find(|l| l.name == "Logic")
            .unwrap_or_else(|| panic!("map '{tmx_path}' has no 'Logic' layer"));
        let logic_tiles = logic_layer
            .as_tile_layer()
            .unwrap_or_else(|| panic!("'Logic' layer in '{tmx_path}' is not a tile layer"));

        let ground_layer = map
            .layers()
            .find(|l| l.name == "Ground")
            .unwrap_or_else(|| panic!("map '{tmx_path}' has no 'Ground' layer"));
        let ground_tile_layer = ground_layer
            .as_tile_layer()
            .unwrap_or_else(|| panic!("'Ground' layer in '{tmx_path}' is not a tile layer"));

        let cols = map.width as usize;
        let rows = map.height as usize;
        let tile_size = map.tile_width as f32;
        assert_eq!(
            map.tile_width, map.tile_height,
            "map '{tmx_path}' must use square tiles"
        );

        let mut cells: HashMap<Cell, TileKind> = HashMap::new();
        for y in 0..rows as i32 {
            for x in 0..cols as i32 {
                let Some(layer_tile) = logic_tiles.get_tile(x, y) else {
                    continue;
                };
                let Some(tile) = layer_tile.get_tile() else {
                    continue;
                };
                let Some(kind) = tile_kind(&tile) else {
                    continue;
                };
                cells.insert((x, y), kind);
            }
        }

        let ordered_path = trace_path(&cells).unwrap_or_else(|e| {
            panic!("invalid path in '{tmx_path}' Logic layer: {e}");
        });
        let waypoints = ordered_path
            .into_iter()
            .map(|c| cell_center(c, tile_size))
            .collect();

        let build_spots = cells
            .iter()
            .filter(|(_, k)| **k == TileKind::Build)
            .map(|(c, _)| cell_center(*c, tile_size))
            .collect();

        let mut ground_tiles = vec![vec![None; cols]; rows];
        for y in 0..rows as i32 {
            for x in 0..cols as i32 {
                ground_tiles[y as usize][x as usize] =
                    ground_tile_layer.get_tile(x, y).map(|t| t.id());
            }
        }

        let tileset = map
            .tilesets()
            .first()
            .unwrap_or_else(|| panic!("map '{tmx_path}' has no tileset"));
        let image = tileset
            .image
            .as_ref()
            .unwrap_or_else(|| panic!("tileset in '{tmx_path}' has no image"));
        let tileset_image_path = image.source.clone();
        assert_eq!(
            tileset.tile_width, tileset.tile_height,
            "tileset in '{tmx_path}' must use square tiles"
        );
        // The tileset's own tile pixel size, independent of the map's
        // grid `tile_size` above: a 32px-tile tileset image is scaled up
        // to fill each `tile_size` grid cell (see `Map::draw`), so it
        // doesn't need to match the map's logical grid size.
        let source_tile_size = tileset.tile_width as f32;

        let wave_ron = std::fs::read_to_string(&waves_path)
            .unwrap_or_else(|e| panic!("failed to read wave file '{waves_path}': {e}"));
        let wave_file: WaveFile = ron::from_str(&wave_ron)
            .unwrap_or_else(|e| panic!("failed to parse wave file '{waves_path}': {e}"));

        Level {
            waypoints,
            build_spots,
            ground_tiles,
            cols,
            rows,
            tile_size,
            source_tile_size,
            tileset_image_path,
            waves: wave_file.waves,
        }
    }
}

fn tile_kind(tile: &tiled::Tile) -> Option<TileKind> {
    match tile.properties.get("kind") {
        Some(tiled::PropertyValue::StringValue(s)) => match s.as_str() {
            "path" => Some(TileKind::Path),
            "build" => Some(TileKind::Build),
            "start" => Some(TileKind::Start),
            "end" => Some(TileKind::End),
            _ => None,
        },
        _ => None,
    }
}

fn cell_center(cell: Cell, tile_size: f32) -> Vec2 {
    let (x, y) = cell;
    vec2(
        (x as f32 + 0.5) * tile_size,
        (y as f32 + 0.5) * tile_size + crate::map::TOP_BAR,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(cells: &[(Cell, TileKind)]) -> HashMap<Cell, TileKind> {
        cells.iter().cloned().collect()
    }

    #[test]
    fn straight_line() {
        let cells = grid(&[
            ((0, 0), TileKind::Start),
            ((1, 0), TileKind::Path),
            ((2, 0), TileKind::Path),
            ((3, 0), TileKind::End),
        ]);
        assert_eq!(
            trace_path(&cells).unwrap(),
            vec![(0, 0), (1, 0), (2, 0), (3, 0)]
        );
    }

    #[test]
    fn path_with_turns() {
        let cells = grid(&[
            ((0, 0), TileKind::Start),
            ((1, 0), TileKind::Path),
            ((1, 1), TileKind::Path),
            ((1, 2), TileKind::Path),
            ((0, 2), TileKind::End),
        ]);
        assert_eq!(
            trace_path(&cells).unwrap(),
            vec![(0, 0), (1, 0), (1, 1), (1, 2), (0, 2)]
        );
    }

    #[test]
    fn branch_is_rejected() {
        let cells = grid(&[
            ((0, 0), TileKind::Start),
            ((1, 0), TileKind::Path),
            ((2, 0), TileKind::Path),
            ((2, 1), TileKind::Path),
            ((2, -1), TileKind::Path),
            ((3, 0), TileKind::End),
        ]);
        assert_eq!(trace_path(&cells), Err(PathError::Branch((2, 0))));
    }

    #[test]
    fn disconnected_extra_path_is_rejected() {
        let cells = grid(&[
            ((0, 0), TileKind::Start),
            ((1, 0), TileKind::End),
            // Unreachable stray path cell, not connected to start/end.
            ((5, 5), TileKind::Path),
        ]);
        assert_eq!(trace_path(&cells), Err(PathError::Disconnected));
    }

    #[test]
    fn dead_end_before_reaching_end_is_rejected() {
        let cells = grid(&[
            ((0, 0), TileKind::Start),
            ((1, 0), TileKind::Path),
            // No connection onward to an End cell.
            ((5, 5), TileKind::End),
        ]);
        assert_eq!(trace_path(&cells), Err(PathError::Disconnected));
    }

    #[test]
    fn missing_start_is_rejected() {
        let cells = grid(&[((1, 0), TileKind::Path), ((2, 0), TileKind::End)]);
        assert_eq!(trace_path(&cells), Err(PathError::NoStart));
    }

    #[test]
    fn missing_end_is_rejected() {
        let cells = grid(&[((0, 0), TileKind::Start), ((1, 0), TileKind::Path)]);
        assert_eq!(trace_path(&cells), Err(PathError::NoEnd));
    }

    #[test]
    fn multiple_start_is_rejected() {
        let cells = grid(&[
            ((0, 0), TileKind::Start),
            ((5, 5), TileKind::Start),
            ((1, 0), TileKind::End),
        ]);
        assert!(matches!(trace_path(&cells), Err(PathError::MultipleStart(_))));
    }
}
