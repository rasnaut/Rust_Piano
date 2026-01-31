fn get_moc_waveform_data() -> Vec<f32> {
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
    get_moc_waveform_data()
}