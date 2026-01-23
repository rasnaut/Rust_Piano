use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

const ATTACK_TIME: f32 = 0.01;
const RELEASE_TIME: f32 = 0.3;

#[derive(Clone)]
struct Voice {
    frequency: f32,
    phase: f32,
    amplitude: f32,
    target_amplitude: f32,
    active: bool,
}

impl Voice {
    fn new(frequency: f32) -> Self {
        Self {
            frequency,
            phase: 0.0,
            amplitude: 0.0,
            target_amplitude: 0.0,
            active: false,
        }
    }

    fn note_on(&mut self) {
        self.target_amplitude = 0.3;
        self.active = true;
    }

    fn note_off(&mut self) {
        self.target_amplitude = 0.0;
    }

    fn next_sample(&mut self, sample_rate: f32) -> f32 {
        if !self.active && self.amplitude < 0.001 {
            return 0.0;
        }

        // Envelope (attack/release)
        if self.amplitude < self.target_amplitude {
            self.amplitude += 1.0 / (ATTACK_TIME * sample_rate);
            if self.amplitude > self.target_amplitude {
                self.amplitude = self.target_amplitude;
            }
        } else if self.amplitude > self.target_amplitude {
            self.amplitude -= 1.0 / (RELEASE_TIME * sample_rate);
            if self.amplitude < 0.0 {
                self.amplitude = 0.0;
                self.active = false;
            }
        }

        // Generate sine wave
        let sample = (self.phase * 2.0 * std::f32::consts::PI).sin() * self.amplitude;

        // Advance phase
        self.phase += self.frequency / sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }
}

struct Synthesizer {
    voices: HashMap<char, Voice>,
    sample_rate: f32,
}

impl Synthesizer {
    fn new(sample_rate: f32) -> Self {
        let key_frequencies: Vec<(char, f32)> = vec![
            ('a', 261.63), // C4
            ('w', 277.18), // C#4
            ('s', 293.66), // D4
            ('e', 311.13), // D#4
            ('d', 329.63), // E4
            ('f', 349.23), // F4
            ('t', 369.99), // F#4
            ('g', 392.00), // G4
            ('y', 415.30), // G#4
            ('h', 440.00), // A4
            ('u', 466.16), // A#4
            ('j', 493.88), // B4
            ('k', 523.25), // C5
            ('o', 554.37), // C#5
            ('l', 587.33), // D5
            ('p', 622.25), // D#5
        ];

        let mut voices = HashMap::new();
        for (key, freq) in key_frequencies {
            voices.insert(key, Voice::new(freq));
        }

        Self {
            voices,
            sample_rate,
        }
    }

    fn note_on(&mut self, key: char) {
        if let Some(voice) = self.voices.get_mut(&key) {
            voice.note_on();
        }
    }

    fn note_off(&mut self, key: char) {
        if let Some(voice) = self.voices.get_mut(&key) {
            voice.note_off();
        }
    }

    fn next_sample(&mut self) -> f32 {
        let mut sample = 0.0;
        let sample_rate = self.sample_rate;
        for voice in self.voices.values_mut() {
            sample += voice.next_sample(sample_rate);
        }
        // Soft clipping to prevent distortion
        sample.tanh()
    }
}

type SynthState = Arc<Mutex<Synthesizer>>;

struct AudioState {
    _stream: cpal::Stream,
}

#[tauri::command]
fn note_on(key: char, state: State<'_, Arc<Mutex<Synthesizer>>>) {
    state.lock().note_on(key);
}

#[tauri::command]
fn note_off(key: char, state: State<'_, Arc<Mutex<Synthesizer>>>) {
    state.lock().note_off(key);
}

#[tauri::command]
fn get_piano_keys() -> Vec<(char, bool)> {
    // Returns (key, is_black_key) for UI rendering
    vec![
        ('a', false), // C4
        ('w', true),  // C#4
        ('s', false), // D4
        ('e', true),  // D#4
        ('d', false), // E4
        ('f', false), // F4
        ('t', true),  // F#4
        ('g', false), // G4
        ('y', true),  // G#4
        ('h', false), // A4
        ('u', true),  // A#4
        ('j', false), // B4
        ('k', false), // C5
        ('o', true),  // C#5
        ('l', false), // D5
        ('p', true),  // D#5
    ]
}

#[tauri::command]
fn get_waveform_data() -> Vec<f32> {
    todo!("This function gets already called in the Front-End here we need to pass the data.");
}

fn setup_audio() -> (SynthState, cpal::Stream) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");

    // Get the default output config from the device
    let supported_config = device
        .default_output_config()
        .expect("No default output config");

    let sample_rate = supported_config.sample_rate().0 as f32;
    let channels = supported_config.channels() as usize;

    let synth = Arc::new(Mutex::new(Synthesizer::new(sample_rate)));
    let synth_clone = synth.clone();

    let config: cpal::StreamConfig = supported_config.into();

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut synth = synth_clone.lock();
                for frame in data.chunks_mut(channels) {
                    let sample = synth.next_sample();
                    for channel_sample in frame.iter_mut() {
                        *channel_sample = sample;
                    }
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )
        .expect("Failed to build output stream");

    stream.play().expect("Failed to play stream");

    (synth, stream)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (synth, stream) = setup_audio();

    let _audio_state = AudioState { _stream: stream };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // https://v2.tauri.app/develop/state-management/
        .manage(synth)
        .invoke_handler(tauri::generate_handler![note_on, note_off, get_piano_keys,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
