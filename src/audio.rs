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

/// Independent filter & RNG state per audio channel (enables decorrelated spatial stereo).
struct ChannelState {
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

impl ChannelState {
    fn new(seed: u64) -> Self {
        Self {
            last_brown: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
            rain_burst: 0.0,
            rng_state: seed,
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

    fn sample(&mut self, noise_type: SoundscapeType) -> f32 {
        let white = self.next_f32();
        match noise_type {
            SoundscapeType::Off => 0.0,
            SoundscapeType::BrownNoise => {
                self.last_brown = (self.last_brown + (0.022 * white)) / 1.022;
                (self.last_brown * 3.8).clamp(-1.0, 1.0)
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
                pink.clamp(-1.0, 1.0)
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
                let s = base_pink + (self.rain_burst * self.next_f32());
                s.clamp(-1.0, 1.0)
            }
        }
    }
}

/// Procedural stereo noise generator (2 channels: independent left and right streams).
pub struct ProceduralNoiseSource {
    noise_type: SoundscapeType,
    sample_rate: u32,
    left: ChannelState,
    right: ChannelState,
    channel_turn: bool,
}

impl ProceduralNoiseSource {
    pub fn new(noise_type: SoundscapeType) -> Self {
        Self {
            noise_type,
            sample_rate: 44100,
            left: ChannelState::new(0x853c49e6748fea9b),
            right: ChannelState::new(0xda3e39cb94b95bdb),
            channel_turn: false,
        }
    }
}

impl Iterator for ProceduralNoiseSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = if !self.channel_turn {
            self.channel_turn = true;
            self.left.sample(self.noise_type)
        } else {
            self.channel_turn = false;
            self.right.sample(self.noise_type)
        };
        Some(sample)
    }
}

impl Source for ProceduralNoiseSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2 // True stereo: decorrelated L and R channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChimeType {
    FocusStart, // Gentle, uplifting harmonic singing bowl / marimba chime (528 Hz)
    BreakStart, // Grounding, warm Tibetan meditation gong / bong (220 Hz)
}

/// Synthesizes organic, pleasant acoustic chime & gong transitions with zero clicks.
pub struct ChimeSource {
    chime_type: ChimeType,
    sample_rate: u32,
    current_sample: usize,
    total_samples: usize,
    volume: f32,
}

impl ChimeSource {
    pub fn new(chime_type: ChimeType, volume: f32) -> Self {
        let sample_rate = 44100;
        let duration_secs = match chime_type {
            ChimeType::FocusStart => 1.4,
            ChimeType::BreakStart => 2.0,
        };
        let total_samples = (sample_rate as f32 * duration_secs) as usize;
        Self {
            chime_type,
            sample_rate,
            current_sample: 0,
            total_samples,
            volume: volume.clamp(0.0, 1.0),
        }
    }
}

impl Iterator for ChimeSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.total_samples {
            return None;
        }

        let t = self.current_sample as f32 / self.sample_rate as f32;
        self.current_sample += 1;

        let sample = match self.chime_type {
            ChimeType::FocusStart => {
                // Harmonic singing bowl chime: 528 Hz (fundamental), 1056 Hz (octave), 792 Hz (fifth)
                // Soft attack 25ms, smooth exponential decay
                let attack = (t / 0.025).min(1.0);
                let decay = (-t * 3.2).exp();
                let env = attack * decay;

                let f0 = 528.0;
                let s1 = (2.0 * std::f32::consts::PI * f0 * t).sin() * 0.65;
                let s2 = (2.0 * std::f32::consts::PI * (f0 * 2.0) * t).sin() * 0.22;
                let s3 = (2.0 * std::f32::consts::PI * (f0 * 1.5) * t).sin() * 0.13;

                env * (s1 + s2 + s3) * self.volume * 0.55
            }
            ChimeType::BreakStart => {
                // Warm, grounding Tibetan meditation bong: 220 Hz with rich sub-harmonics
                // Soft attack 35ms, slow resonant exponential decay
                let attack = (t / 0.035).min(1.0);
                let decay = (-t * 2.1).exp();
                let env = attack * decay;

                let f0 = 220.0;
                let s1 = (2.0 * std::f32::consts::PI * f0 * t).sin() * 0.70;
                let s2 = (2.0 * std::f32::consts::PI * (f0 * 0.5) * t).sin() * 0.15; // gentle sub-harmonic
                let s3 = (2.0 * std::f32::consts::PI * (f0 * 2.0) * t).sin() * 0.15;

                env * (s1 + s2 + s3) * self.volume * 0.60
            }
        };

        Some(sample.clamp(-1.0, 1.0))
    }
}

impl Source for ChimeSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.total_samples.saturating_sub(self.current_sample))
    }

    fn channels(&self) -> u16 {
        1 // Mono source automatically routed to stereo output
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(self.total_samples as f32 / self.sample_rate as f32))
    }
}

pub struct SoundscapePlayer {
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
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
            stream_handle: None,
            sink: None,
            current_type,
            volume,
            auto_play_on_focus,
            is_playing: false,
        }
    }

    fn ensure_audio(&mut self) -> bool {
        if self._stream.is_some() && self.stream_handle.is_some() {
            return true;
        }
        if let Ok((stream, handle)) = rodio::OutputStream::try_default() {
            self._stream = Some(stream);
            self.stream_handle = Some(handle);
            return true;
        }
        false
    }

    fn ensure_sink(&mut self) -> bool {
        if self.sink.is_some() {
            return true;
        }
        if self.ensure_audio() {
            if let Some(handle) = &self.stream_handle {
                if let Ok(sink) = rodio::Sink::try_new(handle) {
                    self.sink = Some(sink);
                    return true;
                }
            }
        }
        false
    }

    pub fn play_chime(&mut self, chime: ChimeType) {
        if !self.ensure_audio() {
            return;
        }
        if let Some(handle) = &self.stream_handle {
            let chime_vol = self.volume.max(0.4);
            let _ = handle.play_raw(ChimeSource::new(chime, chime_vol));
        }
    }

    pub fn play_focus_chime(&mut self) {
        self.play_chime(ChimeType::FocusStart);
    }

    pub fn play_break_bong(&mut self) {
        self.play_chime(ChimeType::BreakStart);
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
        // Trigger soft chime / bong on state transitions
        match new_state {
            FlowState::Focus | FlowState::MicroStart => {
                if old_state != FlowState::Focus && old_state != FlowState::MicroStart {
                    self.play_focus_chime();
                }
            }
            FlowState::Break => {
                if old_state != FlowState::Break {
                    self.play_break_bong();
                }
            }
            _ => {}
        }

        // Handle background noise auto-play
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
            assert_eq!(source.channels(), 2, "Noise generator should output true stereo");
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
    fn test_chime_samples_valid() {
        for chime_type in [ChimeType::FocusStart, ChimeType::BreakStart] {
            let mut chime = ChimeSource::new(chime_type, 0.8);
            assert_eq!(chime.sample_rate(), 44100);
            let mut sample_count = 0;
            while let Some(sample) = chime.next() {
                assert!(!sample.is_nan(), "Chime sample should not be NaN");
                assert!(sample >= -1.0 && sample <= 1.0, "Chime sample out of range");
                sample_count += 1;
            }
            assert!(sample_count > 10_000, "Chime should have valid duration");
        }
    }

    #[test]
    fn test_soundscape_labels() {
        assert_eq!(SoundscapeType::BrownNoise.label_no(), "Brun støy (dyp ro)");
        assert_eq!(SoundscapeType::PinkNoise.label_en(), "Pink Noise (balanced)");
    }
}
