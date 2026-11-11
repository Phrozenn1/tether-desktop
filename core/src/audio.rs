use std::io::Cursor;
use std::time::Duration;
use rodio::Source;
use serde::{Deserialize, Serialize};

use crate::flowmodoro::FlowState;

const RAIN_BYTES: &[u8] = include_bytes!("../../assets/sounds/rain.ogg");
const WAVES_BYTES: &[u8] = include_bytes!("../../assets/sounds/waves.ogg");
const BROOK_BYTES: &[u8] = include_bytes!("../../assets/sounds/brook.ogg");
const WIND_BYTES: &[u8] = include_bytes!("../../assets/sounds/wind.ogg");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundscapeType {
    Off,
    BrownNoise,
    PinkNoise,
    GentleRain,
    GreenNoise,
    Gamma40Hz,
    OceanWaves,
    MountainBrook,
    ForestWind,
}

impl SoundscapeType {
    pub const ALL: [SoundscapeType; 9] = [
        SoundscapeType::Off,
        SoundscapeType::BrownNoise,
        SoundscapeType::PinkNoise,
        SoundscapeType::GentleRain,
        SoundscapeType::GreenNoise,
        SoundscapeType::Gamma40Hz,
        SoundscapeType::OceanWaves,
        SoundscapeType::MountainBrook,
        SoundscapeType::ForestWind,
    ];

    pub fn to_index(self) -> u32 {
        match self {
            SoundscapeType::Off => 0,
            SoundscapeType::BrownNoise => 1,
            SoundscapeType::PinkNoise => 2,
            SoundscapeType::GentleRain => 3,
            SoundscapeType::GreenNoise => 4,
            SoundscapeType::Gamma40Hz => 5,
            SoundscapeType::OceanWaves => 6,
            SoundscapeType::MountainBrook => 7,
            SoundscapeType::ForestWind => 8,
        }
    }

    pub fn from_index(idx: u32) -> Self {
        match idx {
            1 => SoundscapeType::BrownNoise,
            2 => SoundscapeType::PinkNoise,
            3 => SoundscapeType::GentleRain,
            4 => SoundscapeType::GreenNoise,
            5 => SoundscapeType::Gamma40Hz,
            6 => SoundscapeType::OceanWaves,
            7 => SoundscapeType::MountainBrook,
            8 => SoundscapeType::ForestWind,
            _ => SoundscapeType::Off,
        }
    }

    pub fn is_sampled(&self) -> bool {
        matches!(
            self,
            SoundscapeType::GentleRain
                | SoundscapeType::OceanWaves
                | SoundscapeType::MountainBrook
                | SoundscapeType::ForestWind
        )
    }

    pub fn sample_bytes(&self) -> Option<&'static [u8]> {
        match self {
            SoundscapeType::GentleRain => Some(RAIN_BYTES),
            SoundscapeType::OceanWaves => Some(WAVES_BYTES),
            SoundscapeType::MountainBrook => Some(BROOK_BYTES),
            SoundscapeType::ForestWind => Some(WIND_BYTES),
            _ => None,
        }
    }

    pub fn label_no(&self) -> &'static str {
        match self {
            SoundscapeType::Off => "Av",
            SoundscapeType::BrownNoise => "Brun støy (dyp ro)",
            SoundscapeType::PinkNoise => "Rosa støy (balansert)",
            SoundscapeType::GentleRain => "Mildt regn (reelt opptak)",
            SoundscapeType::GreenNoise => "Grønn støy (skånsom/skog)",
            SoundscapeType::Gamma40Hz => "40 Hz Gamma-puls (dyp fokus)",
            SoundscapeType::OceanWaves => "Rullende havbølger (reelt opptak)",
            SoundscapeType::MountainBrook => "Sildrende fjellbekk (reelt opptak)",
            SoundscapeType::ForestWind => "Skogsvind (reelt opptak)",
        }
    }

    pub fn label_en(&self) -> &'static str {
        match self {
            SoundscapeType::Off => "Off",
            SoundscapeType::BrownNoise => "Brown Noise (deep calm)",
            SoundscapeType::PinkNoise => "Pink Noise (balanced)",
            SoundscapeType::GentleRain => "Gentle Rain (field recording)",
            SoundscapeType::GreenNoise => "Green Noise (gentle nature band)",
            SoundscapeType::Gamma40Hz => "40 Hz Gamma Pulse (deep focus)",
            SoundscapeType::OceanWaves => "Rolling Ocean Waves (field recording)",
            SoundscapeType::MountainBrook => "Mountain Brook (field recording)",
            SoundscapeType::ForestWind => "Forest Wind (field recording)",
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
    // Green noise (bandpass ~200 - 2000 Hz)
    green_lp: f32,
    green_hp: f32,
    green_prev: f32,
    // 40 Hz Gamma entrainment modulation
    gamma_phase: f32,
    // Ocean waves
    wave_phase: f32,
    wave_filter: f32,
    // Mountain brook (cascading stream & resonant water droplets)
    brook_hp: f32,
    brook_lp: f32,
    brook_prev: f32,
    brook_ripple_phase: f32,
    droplet_phase: f32,
    droplet_freq: f32,
    droplet_env: f32,
    // Forest wind (gusty brown rumble & treetop rustle)
    wind_gust: f32,
    wind_gust_target: f32,
    wind_tree_lp: f32,
    wind_tree_hp: f32,
    wind_tree_prev: f32,
}

impl ChannelState {
    fn new(seed: u64, wave_phase_offset: f32) -> Self {
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
            green_lp: 0.0,
            green_hp: 0.0,
            green_prev: 0.0,
            gamma_phase: 0.0,
            wave_phase: wave_phase_offset.fract(),
            wave_filter: 0.0,
            brook_hp: 0.0,
            brook_lp: 0.0,
            brook_prev: 0.0,
            brook_ripple_phase: 0.0,
            droplet_phase: 0.0,
            droplet_freq: 1200.0,
            droplet_env: 0.0,
            wind_gust: 0.2,
            wind_gust_target: 0.3,
            wind_tree_lp: 0.0,
            wind_tree_hp: 0.0,
            wind_tree_prev: 0.0,
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
            SoundscapeType::GreenNoise => {
                // Pink noise base passed through a warm mid-frequency nature band (~200 - 2000 Hz)
                self.b0 = 0.99886 * self.b0 + white * 0.0555179;
                self.b1 = 0.99332 * self.b1 + white * 0.0750759;
                self.b2 = 0.96900 * self.b2 + white * 0.1538520;
                self.b3 = 0.86650 * self.b3 + white * 0.3104856;
                self.b4 = 0.55000 * self.b4 + white * 0.5329522;
                let base_pink = (self.b0 + self.b1 + self.b2 + self.b3 + self.b4) * 0.12;

                // Highpass at ~220 Hz
                self.green_hp = 0.970 * (self.green_hp + base_pink - self.green_prev);
                self.green_prev = base_pink;
                // Lowpass at ~1800 Hz
                self.green_lp = self.green_lp + 0.225 * (self.green_hp - self.green_lp);

                (self.green_lp * 1.95).clamp(-1.0, 1.0)
            }
            SoundscapeType::Gamma40Hz => {
                // Calming warm brown noise carrier modulated by 40 Hz gamma rhythm
                self.last_brown = (self.last_brown + (0.022 * white)) / 1.022;
                let carrier = self.last_brown * 3.4;

                // 40 Hz entrainment phase
                self.gamma_phase = (self.gamma_phase + (40.0 / 44100.0)).fract();
                let modulation = 1.0 + 0.32 * (2.0 * std::f32::consts::PI * self.gamma_phase).sin();

                (carrier * modulation).clamp(-1.0, 1.0)
            }
            SoundscapeType::OceanWaves => {
                // Slow ocean surf swell (~8.5s cycle) with crest wash & foam
                self.wave_phase = (self.wave_phase + (1.0 / (44100.0 * 8.5))).fract();
                let cycle = self.wave_phase;

                // Asymmetric wave swell envelope
                let (env, filter_alpha, foam_amount) = if cycle < 0.45 {
                    // Swell building up
                    let p = cycle / 0.45;
                    let swell = 0.18 + 0.82 * (1.0 - (p * std::f32::consts::PI).cos()) * 0.5;
                    (swell, 0.025 + 0.12 * p, 0.0)
                } else if cycle < 0.60 {
                    // Crest breaking and foaming wash
                    let p = (cycle - 0.45) / 0.15;
                    let swell = 1.0 - 0.25 * p;
                    let foam = (1.0 - (p - 0.5).abs() * 2.0).max(0.0) * 0.16;
                    (swell, 0.145 - 0.045 * p, foam)
                } else {
                    // Receding wash and foam decay
                    let p = (cycle - 0.60) / 0.40;
                    let swell = 0.75 * (-p * 2.6).exp() + 0.18;
                    (swell, 0.10 * (-p * 2.0).exp() + 0.025, 0.0)
                };

                // Brown base + pink water noise
                self.last_brown = (self.last_brown + (0.022 * white)) / 1.022;
                self.b0 = 0.99886 * self.b0 + white * 0.0555179;
                self.b1 = 0.99332 * self.b1 + white * 0.0750759;
                self.b2 = 0.96900 * self.b2 + white * 0.1538520;
                let pink = (self.b0 + self.b1 + self.b2) * 0.12;

                let wave_noise = self.last_brown * 2.2 + pink * 0.9;
                self.wave_filter += filter_alpha * (wave_noise - self.wave_filter);

                let foam = if foam_amount > 0.0 {
                    white.abs() * foam_amount
                } else {
                    0.0
                };

                ((self.wave_filter * env + foam) * 1.35).clamp(-1.0, 1.0)
            }
            SoundscapeType::MountainBrook => {
                // Cascading clear stream: bandpass water bed + resonant bubbling droplets
                self.b0 = 0.99886 * self.b0 + white * 0.0555179;
                self.b1 = 0.99332 * self.b1 + white * 0.0750759;
                self.b2 = 0.96900 * self.b2 + white * 0.1538520;
                self.b3 = 0.86650 * self.b3 + white * 0.3104856;
                let stream_pink = (self.b0 + self.b1 + self.b2 + self.b3) * 0.14;

                // Highpass ~350 Hz, Lowpass ~2600 Hz
                self.brook_hp = 0.955 * (self.brook_hp + stream_pink - self.brook_prev);
                self.brook_prev = stream_pink;
                self.brook_lp += 0.32 * (self.brook_hp - self.brook_lp);

                // Water ripple modulation (~3.3 Hz)
                self.brook_ripple_phase = (self.brook_ripple_phase + (3.3 / 44100.0)).fract();
                let ripple = 0.82 + 0.18 * (2.0 * std::f32::consts::PI * self.brook_ripple_phase).sin();
                let bed = self.brook_lp * ripple * 1.5;

                // Resonant droplet generator (~20 bubbles/sec per channel)
                if self.droplet_env < 0.006 {
                    if (self.rng_state & 0x07ff) == 77 {
                        self.droplet_env = 0.24 + 0.16 * self.next_f32().abs();
                        self.droplet_freq = 950.0 + 1250.0 * self.next_f32().abs();
                        self.droplet_phase = 0.0;
                    }
                }

                let drop_sample = if self.droplet_env >= 0.006 {
                    self.droplet_freq += 14.0; // bubble pinch-off rising pitch
                    self.droplet_phase = (self.droplet_phase + (self.droplet_freq / 44100.0)).fract();
                    let s = (2.0 * std::f32::consts::PI * self.droplet_phase).sin() * self.droplet_env;
                    self.droplet_env *= 0.9955;
                    s
                } else {
                    0.0
                };

                (bed * 0.75 + drop_sample * 0.45).clamp(-1.0, 1.0)
            }
            SoundscapeType::ForestWind => {
                // Organic wind gusts swaying deep brown rumble and rustling treetops
                // Very slow chaotic gust envelope (~4-10s cycles)
                if (self.rng_state & 0x03ff) == 13 {
                    self.wind_gust_target = 0.15 + 0.80 * self.next_f32().abs();
                }
                self.wind_gust += 0.00035 * (self.wind_gust_target - self.wind_gust);

                // Brown base rumble
                self.last_brown = (self.last_brown + (0.022 * white)) / 1.022;
                let rumble = self.last_brown * 2.8 * (0.35 + 0.65 * self.wind_gust);

                // Treetop rustle bandpass (~800 - 2200 Hz)
                self.b0 = 0.99886 * self.b0 + white * 0.0555179;
                self.b1 = 0.99332 * self.b1 + white * 0.0750759;
                self.b2 = 0.96900 * self.b2 + white * 0.1538520;
                let foliage = (self.b0 + self.b1 + self.b2) * 0.15;

                self.wind_tree_hp = 0.89 * (self.wind_tree_hp + foliage - self.wind_tree_prev);
                self.wind_tree_prev = foliage;
                self.wind_tree_lp += 0.28 * (self.wind_tree_hp - self.wind_tree_lp);

                let rustle = self.wind_tree_lp * (0.12 + 0.88 * self.wind_gust * self.wind_gust) * 1.6;

                ((rumble * 0.65 + rustle * 0.48) * 1.4).clamp(-1.0, 1.0)
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
            left: ChannelState::new(0x853c49e6748fea9b, 0.0),
            right: ChannelState::new(0xda3e39cb94b95bdb, 0.08),
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
            if let Some(bytes) = self.current_type.sample_bytes() {
                if let Ok(decoder) = rodio::Decoder::new(Cursor::new(bytes)) {
                    sink.append(decoder.repeat_infinite());
                } else {
                    sink.append(ProceduralNoiseSource::new(self.current_type));
                }
            } else {
                sink.append(ProceduralNoiseSource::new(self.current_type));
            }
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
        for noise_type in SoundscapeType::ALL {
            let mut source = ProceduralNoiseSource::new(noise_type);
            assert_eq!(source.channels(), 2, "Noise generator should output true stereo");
            assert_eq!(source.sample_rate(), 44100);

            let mut sum_abs = 0.0f32;
            for _ in 0..20_000 {
                let sample = source.next().expect("infinite source should not end");
                assert!(!sample.is_nan(), "Sample should not be NaN for {:?}", noise_type);
                assert!(!sample.is_infinite(), "Sample should not be infinite for {:?}", noise_type);
                assert!(
                    sample >= -1.0 && sample <= 1.0,
                    "Sample {} out of range for {:?}",
                    sample,
                    noise_type
                );
                sum_abs += sample.abs();
            }

            if noise_type != SoundscapeType::Off {
                assert!(
                    sum_abs > 10.0,
                    "Noise {:?} should produce audible audio signal",
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
    fn test_soundscape_labels_and_indexing() {
        for sound in SoundscapeType::ALL {
            let idx = sound.to_index();
            assert_eq!(SoundscapeType::from_index(idx), sound);
            assert!(!sound.label_no().is_empty());
            assert!(!sound.label_en().is_empty());
        }
        assert_eq!(SoundscapeType::BrownNoise.label_no(), "Brun støy (dyp ro)");
        assert_eq!(SoundscapeType::PinkNoise.label_en(), "Pink Noise (balanced)");
        assert_eq!(SoundscapeType::GreenNoise.label_no(), "Grønn støy (skånsom/skog)");
        assert_eq!(SoundscapeType::Gamma40Hz.label_no(), "40 Hz Gamma-puls (dyp fokus)");
        assert_eq!(SoundscapeType::GentleRain.label_no(), "Mildt regn (reelt opptak)");
        assert_eq!(SoundscapeType::OceanWaves.label_no(), "Rullende havbølger (reelt opptak)");
        assert_eq!(SoundscapeType::MountainBrook.label_no(), "Sildrende fjellbekk (reelt opptak)");
        assert_eq!(SoundscapeType::ForestWind.label_no(), "Skogsvind (reelt opptak)");
    }

    #[test]
    fn test_real_nature_samples_decode_and_loop() {
        for sound in [
            SoundscapeType::GentleRain,
            SoundscapeType::OceanWaves,
            SoundscapeType::MountainBrook,
            SoundscapeType::ForestWind,
        ] {
            assert!(sound.is_sampled());
            let bytes = sound.sample_bytes().expect("sampled sound must have bytes");
            assert!(bytes.len() > 100_000, "sample audio file must be substantial");

            let decoder = rodio::Decoder::new(Cursor::new(bytes))
                .unwrap_or_else(|e| panic!("Failed to decode {:?}: {:?}", sound, e));
            assert_eq!(decoder.channels(), 2, "{:?} must be stereo", sound);
            assert_eq!(decoder.sample_rate(), 44100, "{:?} must be 44.1 kHz", sound);

            let mut looped = decoder.repeat_infinite();
            let mut non_zero = 0;
            for _ in 0..50_000 {
                let sample = looped.next().expect("infinite loop should never end");
                if sample != 0 {
                    non_zero += 1;
                }
            }
            assert!(non_zero > 1_000, "sample audio must have audio energy for {:?}", sound);
        }
    }
}
