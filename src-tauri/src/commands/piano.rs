use tauri::State;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::synth::synthesizer::Synthesizer;

#[tauri::command]
pub fn note_on(key: char, state: State<'_, Arc<Mutex<Synthesizer>>>) {
    state.lock().note_on(key);
}

#[tauri::command]
pub fn note_off(key: char, state: State<'_, Arc<Mutex<Synthesizer>>>) {
    state.lock().note_off(key);
}

#[tauri::command]
pub fn get_piano_keys() -> Vec<(char, bool)> {
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