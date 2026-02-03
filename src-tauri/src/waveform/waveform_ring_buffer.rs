/// Lock-free ring buffer for waveform visualization.
///
/// - Single writer (audio callback)
/// - Multiple readers (IPC / UI)
/// - Stores mono `f32` samples
/// - Overwrites old samples when capacity is exceeded
///
/// Synchronization:
/// - Writer publishes samples with `Release` store to `head`
/// - Readers acquire `head` with `Acquire` load before reading samples
///
/// This buffer does NOT expose references to internal storage.
/// All snapshots return owned data to avoid lifetime issues.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use super::SNAPSHOT_SIZE;

pub const DEFAULT_CAPACITY: usize = 16384;

pub struct WaveformRingBuffer {
    buffer: Box<[AtomicU32]>,   // fix size storage
    mask: usize,                // capacity - 1
    head_pointer: AtomicUsize,  // sequence number
}

impl Default for WaveformRingBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

impl WaveformRingBuffer {
    /// Creates a new ring buffer with the given capacity.
    ///
    /// # Panics
    /// Panics if `capacity` is not a power of two or is zero.
    pub fn new(capacity: usize) -> Self {
        assert!(
            capacity >= SNAPSHOT_SIZE,
            "capacity ({}) must be >= SNAPSHOT_SIZE ({})",
            capacity,
            SNAPSHOT_SIZE
        );
        assert!(capacity.is_power_of_two(), "capacity must be a power of two");

        let buffer: Box<[AtomicU32]> =
            (0..capacity).map(|_| AtomicU32::new(0)).collect();

        Self {
            buffer,
            mask: capacity - 1,
            head_pointer: AtomicUsize::new(0),
        }
    }

    /// Pushes a single mono sample into the ring buffer.
    ///
    /// This method is real-time safe:
    /// - no allocations
    /// - no locks
    /// - O(1)
    ///
    /// Must be called from the audio thread (single writer).
    #[cfg_attr(not(debug_assertions), no_panic::no_panic)]
    #[inline(always)]
    pub fn push(&self, sample: f32) {
        let (insert_index, head_position) = self.load_index_and_head();
        self.store_new_sample_by_index(insert_index, sample);
        self.save_new_position_in_head_pointer(head_position + 1);
    }

    /// Returns a fixed-size snapshot of the last `SNAPSHOT_SIZE` mono samples.
    ///
    /// The snapshot contains the last `SNAPSHOT_SIZE` samples.
    /// If fewer samples are available, the leading part is zero-padded.
    ///
    /// This method allocates and is intended to be called from a non-realtime
    /// thread (e.g. IPC / UI).
    pub fn snapshot(&self) -> Vec<f32> {
        let mut result = vec![0.0f32; SNAPSHOT_SIZE];
        self.snapshot_into(&mut result);
        result
    }
}

impl WaveformRingBuffer {

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.buffer.len()
    }

    #[inline(always)]
    fn load_index_and_head(&self) -> (usize, usize) {
        let pos = self.head_pointer.load(Ordering::Relaxed);
        let index = pos & self.mask;
        (index, pos)
    }

    #[inline(always)]
    fn store_new_sample_by_index(&self, index: usize, sample: f32) {
        self.buffer[index].store(sample.to_bits(), Ordering::Relaxed);
    }

    #[inline(always)]
    fn save_new_position_in_head_pointer(&self, new_position: usize) {
        self.head_pointer.store(new_position, Ordering::Release);
    }

    fn snapshot_into(&self, out: &mut [f32]) {
        let occupied_size = self.clamp_len_and_zero_tail(out);
        
        // Fast-path: occupied_size == 0
        if occupied_size == 0 { return; }

        let current_head_pointer = self.head_pointer.load(Ordering::Acquire);
        self.copy_samples_to_snapshot(out, current_head_pointer, occupied_size);

        let (ok, actual_head_pointer) = self.check_head_pointer(current_head_pointer);
        if !ok {
            self.copy_samples_to_snapshot(out, actual_head_pointer, occupied_size);
        }
    }

    fn clamp_len_and_zero_tail(&self, out: &mut [f32]) -> usize {
        let occupied_size = out.len().min(self.capacity());
        out[occupied_size..].fill(0.0);
        occupied_size
    }

    fn copy_samples_from_ring_buffer_to_slice(
        &self,
        out: &mut [f32],
        current_head_pointer: usize,
        available_elements: usize,
        pad: usize,
    ) {
        let start = current_head_pointer - available_elements;
        for (i, seq) in (start..current_head_pointer).enumerate() {
            let index = seq & self.mask;
            let bits = self.buffer[index].load(Ordering::Relaxed);
            out[pad + i] = f32::from_bits(bits);
        }
    }

    fn out_array_preparation(&self, out: &mut [f32], occupied_size: usize, current_head_pointer: usize) -> (usize, usize) {
        let available_elements = current_head_pointer.min(occupied_size);
        let left_zero_amount = occupied_size - available_elements; 

        for element in &mut out[..left_zero_amount] {
            *element = 0.0;
        }
        (available_elements, left_zero_amount)
    }

    fn check_head_pointer(&self, current_head_pointer: usize) -> (bool, usize) {
        let actual_head_pointer = self.head_pointer.load(Ordering::Acquire);
        (actual_head_pointer == current_head_pointer, actual_head_pointer)
    }

    fn copy_samples_to_snapshot(&self, out: &mut [f32], current_head_pointer: usize, occupied_size: usize) {
        let (available, pad) = self.out_array_preparation(out, occupied_size, current_head_pointer);

        self.copy_samples_from_ring_buffer_to_slice(
            out,
            current_head_pointer,
            available,
            pad,
        );
    }
}
