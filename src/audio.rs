use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, stop_sound};

const SAMPLE_RATE: u32 = 22_050;
const EFFECT_VOLUME: f32 = 0.45;
const MUSIC_VOLUME: f32 = 0.12;

#[derive(Clone, Copy)]
pub enum AudioEvent {
    UiClick,
    TowerFire,
    Impact,
    EnemyEscaped,
    WaveStarted,
    Victory,
    Defeat,
}

impl AudioEvent {
    fn index(self) -> usize {
        match self {
            AudioEvent::UiClick => 0,
            AudioEvent::TowerFire => 1,
            AudioEvent::Impact => 2,
            AudioEvent::EnemyEscaped => 3,
            AudioEvent::WaveStarted => 4,
            AudioEvent::Victory => 5,
            AudioEvent::Defeat => 6,
        }
    }

    fn samples(self) -> Vec<i16> {
        let notes: &[(f32, f32)] = match self {
            AudioEvent::UiClick => &[(880.0, 0.045)],
            AudioEvent::TowerFire => &[(440.0, 0.045), (660.0, 0.05)],
            AudioEvent::Impact => &[(260.0, 0.035), (150.0, 0.06)],
            AudioEvent::EnemyEscaped => &[(392.0, 0.08), (262.0, 0.08), (175.0, 0.12)],
            AudioEvent::WaveStarted => &[(440.0, 0.09), (587.0, 0.09), (784.0, 0.15)],
            AudioEvent::Victory => &[(523.0, 0.1), (659.0, 0.1), (784.0, 0.1), (1047.0, 0.24)],
            AudioEvent::Defeat => &[(392.0, 0.12), (330.0, 0.12), (262.0, 0.18), (196.0, 0.24)],
        };
        render_notes(notes, 0.35)
    }
}

pub struct AudioManager {
    effects: Vec<Sound>,
    music: Sound,
    music_playing: bool,
}

impl AudioManager {
    pub async fn new() -> Self {
        let events = [
            AudioEvent::UiClick,
            AudioEvent::TowerFire,
            AudioEvent::Impact,
            AudioEvent::EnemyEscaped,
            AudioEvent::WaveStarted,
            AudioEvent::Victory,
            AudioEvent::Defeat,
        ];
        let mut effects = Vec::with_capacity(events.len());
        for event in events {
            effects.push(load_wav(event.samples()).await);
        }
        let music = load_wav(music_samples()).await;
        AudioManager {
            effects,
            music,
            music_playing: false,
        }
    }

    pub fn play(&self, event: AudioEvent) {
        play_sound(
            &self.effects[event.index()],
            PlaySoundParams {
                looped: false,
                volume: EFFECT_VOLUME,
            },
        );
    }

    pub fn start_music(&mut self) {
        if self.music_playing {
            stop_sound(&self.music);
        }
        play_sound(
            &self.music,
            PlaySoundParams {
                looped: true,
                volume: MUSIC_VOLUME,
            },
        );
        self.music_playing = true;
    }

    pub fn stop_music(&mut self) {
        if self.music_playing {
            stop_sound(&self.music);
            self.music_playing = false;
        }
    }
}

async fn load_wav(samples: Vec<i16>) -> Sound {
    load_sound_from_bytes(&encode_wav(&samples))
        .await
        .unwrap_or_else(|error| panic!("failed to load generated audio: {error}"))
}

fn render_notes(notes: &[(f32, f32)], amplitude: f32) -> Vec<i16> {
    let mut samples = Vec::new();
    for &(frequency, duration) in notes {
        let count = (SAMPLE_RATE as f32 * duration) as usize;
        for sample_index in 0..count {
            let progress = sample_index as f32 / count as f32;
            let envelope = (progress / 0.08)
                .min((1.0 - progress) / 0.18)
                .clamp(0.0, 1.0);
            let phase =
                std::f32::consts::TAU * frequency * sample_index as f32 / SAMPLE_RATE as f32;
            let value = if phase.sin() >= 0.0 { 1.0 } else { -1.0 };
            samples.push((value * envelope * amplitude * i16::MAX as f32) as i16);
        }
    }
    samples
}

fn music_samples() -> Vec<i16> {
    let melody = [
        523.0, 659.0, 784.0, 659.0, 587.0, 440.0, 523.0, 392.0, 523.0, 659.0, 880.0, 784.0, 659.0,
        587.0, 440.0, 392.0,
    ];
    let notes: Vec<_> = melody
        .into_iter()
        .map(|frequency| (frequency, 0.16))
        .collect();
    render_notes(&notes, 0.18)
}

fn encode_wav(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_audio_is_mono_pcm_wav() {
        let samples = AudioEvent::WaveStarted.samples();
        let wav = encode_wav(&samples);

        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(
            u32::from_le_bytes(wav[24..28].try_into().unwrap()),
            SAMPLE_RATE
        );
        assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(
            u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize,
            samples.len() * 2
        );
    }

    #[test]
    fn music_placeholder_contains_a_loopable_melody() {
        let samples = music_samples();

        assert_eq!(samples.len(), (SAMPLE_RATE as f32 * 0.16 * 16.0) as usize);
        assert!(samples.iter().any(|sample| *sample != 0));
    }
}
