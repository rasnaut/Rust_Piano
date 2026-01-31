use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::synth::voice::Voice;

pub(crate) struct Synthesizer {
    voices_map: HashMap<char, Voice>,
    sample_rate: f32,
}

impl Synthesizer {
    pub fn new(sample_rate: f32) -> Self {
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

        let mut voices_map = HashMap::new();
        for (key, freq) in key_frequencies {
            voices_map.insert(key, Voice::new(freq));
        }

        Self {
            voices_map,
            sample_rate,
        }
    }

    pub fn note_on(&mut self, key: char) {
        if let Some(voice_element) = self.voices_map.get_mut(&key) {
            voice_element.note_on();
        }
    }

    pub fn note_off(&mut self, key: char) {
        if let Some(voice_element) = self.voices_map.get_mut(&key) {
            voice_element.note_off();
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let mut sample = 0.0;
        let sample_rate = self.sample_rate;
        for voice_element in self.voices_map.values_mut() {
            sample += voice_element.next_sample(sample_rate);
        }
        // Soft clipping to prevent distortion
        sample.tanh()
    }
}

pub type SynthState = Arc<Mutex<Synthesizer>>;