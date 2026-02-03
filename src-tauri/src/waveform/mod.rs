pub mod waveform;
pub mod waveform_ring_buffer;
mod waveform_consumer;

#[cfg(test)]
mod tests;
//#[cfg(test)]
//mod test_consumer;

/// Fixed snapshot size for waveform visualization (60 FPS)
pub const SNAPSHOT_SIZE: usize = 2048;