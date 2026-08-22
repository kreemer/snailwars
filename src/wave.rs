//! Wave definitions: which enemies spawn, in what order, and how quickly,
//! for each wave of the game.

use crate::enemy::EnemyType;

pub const TOTAL_WAVES: u32 = 10;

/// A single enemy spawn entry within a wave.
pub struct SpawnEntry {
    pub kind: EnemyType,
    pub delay_after_previous: f32,
}

pub struct Wave {
    pub spawns: Vec<SpawnEntry>,
}

/// Build the spawn list for the given wave number (1-indexed).
pub fn build_wave(wave_number: u32) -> Wave {
    let mut spawns = Vec::new();
    let snail_count = 4 + wave_number;
    let slug_count = wave_number.saturating_sub(1) * 2;

    for i in 0..snail_count {
        spawns.push(SpawnEntry {
            kind: EnemyType::Snail,
            delay_after_previous: if i == 0 { 0.0 } else { 0.7 },
        });
    }
    for i in 0..slug_count {
        spawns.push(SpawnEntry {
            kind: EnemyType::Slug,
            delay_after_previous: if i == 0 && snail_count == 0 { 0.0 } else { 0.45 },
        });
    }
    // Boss every 5th wave.
    if wave_number.is_multiple_of(5) {
        spawns.push(SpawnEntry { kind: EnemyType::BigSnail, delay_after_previous: 1.5 });
    }

    Wave { spawns }
}
