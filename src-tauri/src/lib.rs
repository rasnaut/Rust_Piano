mod commands;
mod audio;
mod synth;
mod waveform;

use crate::audio::engine::*;
use crate::waveform::waveform_ring_buffer::WaveformRingBuffer;
//use crate::synth::synthesizer::Synthesizer;
use crate::commands::piano::*;
use std::{sync::Arc, thread, time::Duration, time::Instant};
use std::sync::atomic::{AtomicU64, Ordering};

//use crate::app::state::AppState;
struct AudioState {
    _stream: cpal::Stream,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (synth, stream, ring_audio_buffer) = setup_audio();

    let _audio_state = AudioState { _stream: stream };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // https://v2.tauri.app/develop/state-management/
        .manage(synth)
        .invoke_handler(tauri::generate_handler![
            note_on, 
            note_off, 
            get_piano_keys, 
            crate::waveform::waveform::get_waveform_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
