use std::sync::Arc;

use super::waveform_consumer::{self, FRAME_SIZE};
use super::waveform_ring_buffer::WaveformRingBuffer;

pub struct WaveformGuard;

impl WaveformGuard {
    pub fn new(ring_buffer: Arc<WaveformRingBuffer>) -> Self {
        waveform_consumer::start_waveform_consumer(ring_buffer);
        Self
    }
}

impl Drop for WaveformGuard {
    fn drop(&mut self) {
        waveform_consumer::stop_waveform_consumer();
    }
}

fn _get_moc_waveform_data() -> Vec<f32> {
    // Mock waveform data for testing
    let mut waveform = Vec::new();
    for i in 0..512 {
        let value = (i as f32 / 512.0 * std::f32::consts::PI * 2.0).sin();
        waveform.push(value);
    }
    waveform
}

#[tauri::command]
pub fn get_waveform_data() -> Vec<f32> {
    let Some(frames) = waveform_consumer::LATEST_FRAMES.get() else {
        return vec![0.0; FRAME_SIZE];
    };

    frames.read_active_frame().to_vec()
}