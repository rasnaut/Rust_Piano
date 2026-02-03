use super::waveform_ring_buffer::WaveformRingBuffer;
use super::SNAPSHOT_SIZE;

fn push_range(rb: &WaveformRingBuffer, from: usize, to_exclusive: usize) {
    for i in from..to_exclusive {
        rb.push(i as f32);
    }
}

#[test]
fn snapshot_wrap_around_last_samples() {
    let rb = WaveformRingBuffer::new(SNAPSHOT_SIZE);

    // Запишем 12 сэмплов: 0..12
    push_range(&rb, 0, 12);

    // Последние 8 (вся ёмкость) должны быть 4..11
    let snap = rb.snapshot();
    assert_eq!(&snap[SNAPSHOT_SIZE-8..SNAPSHOT_SIZE], &[4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0]);
}

#[test]
fn snapshot_is_fixed_size_and_returns_latest_samples() {
    let rb = WaveformRingBuffer::default();

    // Запишем 3000 значений
    push_range(&rb, 0, 3000);

    let snap = rb.snapshot(); // или rb.snapshot(999) если сигнатура с параметром
    assert_eq!(snap.len(), SNAPSHOT_SIZE);

    // Последние 2048 из диапазона 0..3000 это 952..2999
    let start = 3000 - SNAPSHOT_SIZE;
    assert_eq!(snap[0], start as f32);
    assert_eq!(snap[SNAPSHOT_SIZE - 1], 2999.0);
}

#[test]
fn snapshot_padding_when_not_enough_data() {
    let rb = WaveformRingBuffer::new(SNAPSHOT_SIZE);

    rb.push(1.0);
    rb.push(2.0);
    rb.push(3.0);

    let snap = rb.snapshot();
    assert_eq!(snap[SNAPSHOT_SIZE - 5..SNAPSHOT_SIZE], vec![0.0, 0.0, 1.0, 2.0, 3.0]);
}

#[test]
fn snapshot_order_is_monotonic_for_sequential_pushes() {
    let rb = WaveformRingBuffer::new(SNAPSHOT_SIZE);
    push_range(&rb, 100, 120); // 20 samples

    // Возьмём последние 10: 110..119
    let snap = rb.snapshot();
    assert_eq!(&snap[SNAPSHOT_SIZE-10..SNAPSHOT_SIZE], &[110.0, 111.0, 112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0]);
}

#[test]
fn snapshot_is_zero_padded_when_not_enough_data() {
    let rb = WaveformRingBuffer::default();

    rb.push(1.0);
    rb.push(2.0);
    rb.push(3.0);

    let snap = rb.snapshot();
    assert_eq!(snap.len(), SNAPSHOT_SIZE);

    // первые SNAPSHOT_SIZE-3 нули, хвост 1,2,3
    assert!(snap[..SNAPSHOT_SIZE - 3].iter().all(|&x| x == 0.0));
    assert_eq!(&snap[SNAPSHOT_SIZE - 3..], &[1.0, 2.0, 3.0]);
}

#[test]
fn snapshot_wrap_around() {
    let rb = WaveformRingBuffer::default();

    // Запишем больше capacity, чтобы точно перезаписать
    push_range(&rb, 0, 20000);

    let snap = rb.snapshot();
    assert_eq!(snap.len(), SNAPSHOT_SIZE);

    // Последние 2048 из 0..20000 это 17952..19999
    assert_eq!(snap[0], (20000 - SNAPSHOT_SIZE) as f32);
    assert_eq!(snap[SNAPSHOT_SIZE - 1], 19999.0);
}

use std::sync::{
    Arc,
    Barrier,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

#[test]
fn stress_single_writer_multi_readers_snapshot_is_consistent() {
    let rb = Arc::new(WaveformRingBuffer::default());

    // Прогрев: чтобы snapshot почти всегда был без padding нулями
    for i in 0..(SNAPSHOT_SIZE * 2) {
        rb.push(i as f32);
    }

    let stop = Arc::new(AtomicBool::new(false));
    let start = Arc::new(Barrier::new(1 + 4)); // 1 writer + 4 readers

    // Writer
    let rb_w = rb.clone();
    let stop_w = stop.clone();
    let start_w = start.clone();
    let writer = thread::spawn(move || {
        let mut i = (SNAPSHOT_SIZE * 2) as u32;
        start_w.wait();
        while !stop_w.load(Ordering::Relaxed) {
            rb_w.push(i as f32);
            i = i.wrapping_add(1);

            // небольшая уступка планировщику, чтобы тест не жёг 100% CPU
            if (i & 0x3FF) == 0 {
                thread::yield_now();
            }
        }
    });

    // Readers
    let mut readers = Vec::new();
    for _ in 0..4 {
        let rb_r = rb.clone();
        let stop_r = stop.clone();
        let start_r = start.clone();

        readers.push(thread::spawn(move || {
            let mut last_tail = f32::NEG_INFINITY;

            start_r.wait();
            while !stop_r.load(Ordering::Relaxed) {
                let snap = rb_r.snapshot();
                assert_eq!(snap.len(), SNAPSHOT_SIZE);

                // 1) никаких NaN/Inf
                for &x in snap.iter() {
                    assert!(x.is_finite(), "snapshot contains non-finite value: {x}");
                }

                // 2) находим первый ненулевой элемент (если padding присутствует)
                let first_non_zero = snap.iter().position(|&x| x != 0.0).unwrap_or(SNAPSHOT_SIZE);

                // 3) после него значения должны идти подряд: x[i+1] == x[i] + 1
                // (если все нули — просто пропускаем)
                if first_non_zero < SNAPSHOT_SIZE {
                    for k in (first_non_zero + 1)..SNAPSHOT_SIZE {
                        let prev = snap[k - 1];
                        let cur  = snap[k];
                        assert!(
                            cur == prev + 1.0,
                            "non-consecutive snapshot sequence at idx {}: prev={}, cur={}",
                            k, prev, cur
                        );
                    }

                    // 4) хвост (последний элемент) в рамках одного reader не должен убывать
                    let tail = snap[SNAPSHOT_SIZE - 1];
                    assert!(
                        tail >= last_tail,
                        "tail went backwards: was {}, now {}",
                        last_tail, tail
                    );
                    last_tail = tail;
                }
            }
        }));
    }

    // Запускаем на короткое время
    thread::sleep(Duration::from_millis(350));
    stop.store(true, Ordering::Relaxed);

    writer.join().unwrap();
    for r in readers {
        r.join().unwrap();
    }
}
