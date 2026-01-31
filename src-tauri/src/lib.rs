mod commands;
mod audio;
mod synth;
mod waveform;

use crate::audio::engine::setup_audio;
//use crate::synth::synthesizer::Synthesizer;
use crate::commands::piano::*;

//use crate::app::state::AppState;
struct AudioState {
    _stream: cpal::Stream,
}

// fn get_moc_waveform_data() -> Vec<f32> {
//     // Mock waveform data for testing
//     let mut waveform = Vec::new();
//     for i in 0..512 {
//         let value = (i as f32 / 512.0 * std::f32::consts::PI * 2.0).sin();
//         waveform.push(value);
//     }
//     waveform
// }

// #[tauri::command]
// fn get_waveform_data() -> Vec<f32> {
//     get_moc_waveform_data()
// }



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (synth, stream) = setup_audio();

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
