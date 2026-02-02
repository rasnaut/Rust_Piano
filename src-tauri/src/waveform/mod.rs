pub mod waveform;
mod waveform_ring_buffer;

#[cfg(test)]
mod tests;

/// Fixed snapshot size for waveform visualization (60 FPS)
pub const SNAPSHOT_SIZE: usize = 2048;