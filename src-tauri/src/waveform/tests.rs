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
    assert_eq!(&snap[0..8], &[4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0]);
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
    assert_eq!(snap[0..10], vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.0, 0.0]);
}

#[test]
fn snapshot_order_is_monotonic_for_sequential_pushes() {
    let rb = WaveformRingBuffer::new(SNAPSHOT_SIZE);
    push_range(&rb, 100, 120); // 20 samples

    // Возьмём последние 10: 110..119
    let snap = rb.snapshot();
    assert_eq!(snap[6..16], vec![110.0, 111.0, 112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0]);
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


