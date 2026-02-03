// waveform_consumer.rs
use std::{
    cell::UnsafeCell,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
        OnceLock,
    },
    thread,
    time::{Duration, Instant},
};

use super::waveform_ring_buffer::WaveformRingBuffer;

pub const FRAME_SIZE: usize = 512;
pub const FPS: u64 = 60;
pub const FRAME_DURATION_NS: u64 = 1_000_000_000u64 / FPS;

/// Lock-free double buffer for frames.
pub struct DoubleFrameBuffer {
    frames: [UnsafeCell<[f32; FRAME_SIZE]>; 2],
    active: AtomicUsize,
}

unsafe impl Sync for DoubleFrameBuffer {} // обосновано выше

impl DoubleFrameBuffer {
    pub fn new() -> Self {
        Self {
            frames: [
                UnsafeCell::new([0.0; FRAME_SIZE]),
                UnsafeCell::new([0.0; FRAME_SIZE]),
            ],
            active: AtomicUsize::new(0),
        }
    }

    #[inline]
    pub fn active_index(&self) -> usize {
        self.active.load(Ordering::Acquire) & 1
    }

    #[inline]
    pub fn read_active_frame(&self) -> &[f32; FRAME_SIZE] {
        let idx = self.active_index();
        unsafe { &*self.frames[idx].get() }
    }

    #[inline]
    pub fn write_inactive_frame(&self) -> (&mut [f32; FRAME_SIZE], usize) {
        let active = self.active.load(Ordering::Relaxed) & 1;
        let write_idx = active ^ 1;
        let buf = unsafe { &mut *self.frames[write_idx].get() };
        (buf, write_idx)
    }

    #[inline]
    pub fn publish(&self, write_idx: usize) {
        self.active.store(write_idx & 1, Ordering::Release);
    }
}

pub static LATEST_FRAMES: OnceLock<Arc<DoubleFrameBuffer>> = OnceLock::new();
static STOP_FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();

/// Start consumer thread. One time call. 
pub fn start_waveform_consumer(rb: Arc<WaveformRingBuffer>) -> Arc<DoubleFrameBuffer> {
    let frames = LATEST_FRAMES.get_or_init(|| Arc::new(DoubleFrameBuffer::new())).clone();
    let stop = STOP_FLAG.get_or_init(|| Arc::new(AtomicBool::new(false))).clone();

    thread::Builder::new()
        .name("waveform-consumer".into())
        .spawn({
            let frames = frames.clone();
            let stop = stop.clone();
            move || consumer_loop(rb, frames, stop)
        })
        .expect("failed to spawn waveform consumer thread");

    frames
}

pub fn stop_waveform_consumer() {
    if let Some(flag) = STOP_FLAG.get() {
        flag.store(true, Ordering::Relaxed);
    }
}

fn consumer_loop(
    rb: Arc<WaveformRingBuffer>,
    frames: Arc<DoubleFrameBuffer>,
    stop: Arc<AtomicBool>,
) {
    let tick = Duration::from_nanos(FRAME_DURATION_NS);
    let mut next = Instant::now() + tick;

    while !stop.load(Ordering::Relaxed) {
        
        let snapshot = rb.snapshot(); // Vec<f32> len=SNAPSHOT_SIZE

        let (out, write_idx) = frames.write_inactive_frame();
        build_frame_512_from_snapshot(&snapshot, out);
        frames.publish(write_idx);

        let now = Instant::now();
        if now < next {
            thread::sleep(next - now);
            next += tick;
        } else {
            next = now + tick;
        }
    }
}

/// 2048 samples -> 512 points.
fn build_frame_512_from_snapshot(snapshot: &[f32], out: &mut [f32; FRAME_SIZE]) {
    debug_assert!(snapshot.len() >= FRAME_SIZE);

    let step = snapshot.len() / FRAME_SIZE; // при 2048/512 = 4
    for i in 0..FRAME_SIZE {
        let start = i * step;
        let end = (start + step).min(snapshot.len());

        let mut best = 0.0f32;
        let mut best_abs = 0.0f32;

        for &s in &snapshot[start..end] {
            let a = s.abs();
            if a > best_abs {
                best_abs = a;
                best = s;
            }
        }
        out[i] = best;
    }
}
