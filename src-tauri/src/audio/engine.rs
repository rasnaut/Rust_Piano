use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::synth::synthesizer::{Synthesizer, SynthState};
use parking_lot::Mutex;
use std::sync::Arc;

pub fn setup_audio() -> (SynthState, cpal::Stream) {
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