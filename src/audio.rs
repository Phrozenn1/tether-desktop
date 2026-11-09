use std::time::Duration;
use rodio::Source;
use serde::{Deserialize, Serialize};

use crate::flowmodoro::FlowState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundscapeType {
    Off,
    BrownNoise,
    PinkNoise,
    GentleRain,
}

impl SoundscapeType {
    pub fn label_no(&self) -> &'static str {
        match self {
            SoundscapeType::Off => "Av",
            SoundscapeType::BrownNoise => "Brun støy (dyp ro)",
            SoundscapeType::PinkNoise => "Rosa støy (balansert)",
            SoundscapeType::GentleRain => "Mildt regn",
        }
    }

    pub fn label_en(&self) -> &'static str {
        match self {
            SoundscapeType::Off => "Off",
            SoundscapeType::BrownNoise => "Brown Noise (deep calm)",
            SoundscapeType::PinkNoise => "Pink Noise (balanced)",
            SoundscapeType::GentleRain => "Gentle Rain",
        }
    }
}

pub struct ProceduralNoiseSource {
    noise_type: SoundscapeType,
    sample_rate: u32,
    last_brown: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
    rain_burst: f32,
    rng_state: u64,
}

impl ProceduralNoiseSource {
    pub fn new(noise_type: SoundscapeType) -> Self {
        Self {
            noise_type,
            sample_rate: 44100,
            last_brown: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
            rain_burst: 0.0,
            rng_state: 0x853c49e6748fea9b,
        }
    }

    fn next_f32(&mut self) -> f32 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let x = ((self.rng_state >> 32) as u32) as f32;
        (x / (u32::MAX as f32)) * 2.0 - 1.0
    }
}

impl Iterator for ProceduralNoiseSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let white = self.next_f32();
        match self.noise_type {
            SoundscapeType::Off => Some(0.0),
            SoundscapeType::BrownNoise => {
                self.last_brown = (self.last_brown + (0.022 * white)) / 1.022;
                Some((self.last_brown * 3.8).clamp(-1.0, 1.0))
            }
            SoundscapeType::PinkNoise => {
                self.b0 = 0.99886 * self.b0 + white * 0.0555179;
                self.b1 = 0.99332 * self.b1 + white * 0.0750759;
                self.b2 = 0.96900 * self.b2 + white * 0.1538520;
                self.b3 = 0.86650 * self.b3 + white * 0.3104856;
                self.b4 = 0.55000 * self.b4 + white * 0.5329522;
                self.b5 = -0.7616 * self.b5 - white * 0.0168980;
                let pink = (self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6 + white * 0.5362)
                    * 0.11;
                self.b6 = white * 0.115926;
                Some(pink.clamp(-1.0, 1.0))
            }
            SoundscapeType::GentleRain => {
                self.b0 = 0.99886 * self.b0 + white * 0.0555179;
                self.b1 = 0.99332 * self.b1 + white * 0.0750759;
                self.b2 = 0.96900 * self.b2 + white * 0.1538520;
                self.b3 = 0.86650 * self.b3 + white * 0.3104856;
                self.b4 = 0.55000 * self.b4 + white * 0.5329522;
                let base_pink = (self.b0 + self.b1 + self.b2 + self.b3 + self.b4) * 0.08;

                if white > 0.991 {
                    self.rain_burst = 0.22;
                }
                self.rain_burst *= 0.95;
                let sample = base_pink + (self.rain_burst * self.next_f32());
                Some(sample.clamp(-1.0, 1.0))
            }
        }
    }
}

impl Source for ProceduralNoiseSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub struct SoundscapePlayer {
    _stream: Option<rodio::OutputStream>,
    sink: Option<rodio::Sink>,
    pub current_type: SoundscapeType,
    pub volume: f32,
    pub auto_play_on_focus: bool,
    pub is_playing: bool,
}

impl SoundscapePlayer {
    pub fn new(current_type: SoundscapeType, volume: f32, auto_play_on_focus: bool) -> Self {
        Self {
            _stream: None,
            sink: None,
            current_type,
            volume,
            auto_play_on_focus,
            is_playing: false,
        }
    }

    fn ensure_sink(&mut self) -> bool {
        if self.sink.is_some() {
            return true;
        }
        if let Ok((stream, handle)) = rodio::OutputStream::try_default() {
            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                self._stream = Some(stream);
                self.sink = Some(sink);
                return true;
            }
        }
        false
    }

    pub fn play(&mut self) {
        if self.current_type == SoundscapeType::Off {
            self.stop();
            return;
        }
        if !self.ensure_sink() {
            return;
        }
        if let Some(sink) = &self.sink {
            sink.stop();
            sink.set_volume(self.volume);
            sink.append(ProceduralNoiseSource::new(self.current_type));
            sink.play();
            self.is_playing = true;
        }
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            self.is_playing = false;
        }
    }

    #[allow(dead_code)]
    pub fn resume(&mut self) {
        if self.current_type != SoundscapeType::Off {
            if let Some(sink) = &self.sink {
                sink.play();
                self.is_playing = true;
            } else {
                self.play();
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
            self.is_playing = false;
        }
    }

    pub fn toggle_play(&mut self) {
        if self.is_playing {
            self.pause();
        } else {
            if self.current_type == SoundscapeType::Off {
                self.current_type = SoundscapeType::BrownNoise;
            }
            self.play();
        }
    }

    pub fn set_sound(&mut self, sound: SoundscapeType) {
        self.current_type = sound;
        if self.is_playing {
            if sound == SoundscapeType::Off {
                self.stop();
            } else {
                self.play();
            }
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
        }
    }

    pub fn handle_flow_state_transition(&mut self, old_state: FlowState, new_state: FlowState) {
        if !self.auto_play_on_focus || self.current_type == SoundscapeType::Off {
            return;
        }

        match new_state {
            FlowState::Focus | FlowState::MicroStart => {
                if !self.is_playing {
                    self.play();
                }
            }
            FlowState::Break | FlowState::Idle => {
                if self.is_playing && (old_state == FlowState::Focus || old_state == FlowState::MicroStart) {
                    self.pause();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_procedural_noise_samples_valid() {
        for noise_type in [
            SoundscapeType::BrownNoise,
            SoundscapeType::PinkNoise,
            SoundscapeType::GentleRain,
        ] {
            let mut source = ProceduralNoiseSource::new(noise_type);
            assert_eq!(source.channels(), 1);
            assert_eq!(source.sample_rate(), 44100);

            for _ in 0..10_000 {
                let sample = source.next().expect("infinite source should not end");
                assert!(!sample.is_nan(), "Sample should not be NaN for {:?}", noise_type);
                assert!(
                    sample >= -1.0 && sample <= 1.0,
                    "Sample {} out of range for {:?}",
                    sample,
                    noise_type
                );
            }
        }
    }

    #[test]
    fn test_soundscape_labels() {
        assert_eq!(SoundscapeType::BrownNoise.label_no(), "Brun støy (dyp ro)");
        assert_eq!(SoundscapeType::PinkNoise.label_en(), "Pink Noise (balanced)");
    }
}
