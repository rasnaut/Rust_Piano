const ATTACK_TIME: f32 = 0.01;
const RELEASE_TIME: f32 = 0.3;

#[derive(Clone)]
pub(crate)struct Voice {
    frequency: f32,
    phase: f32,
    amplitude: f32,
    target_amplitude: f32,
    active: bool,
}

impl Voice {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            phase: 0.0,
            amplitude: 0.0,
            target_amplitude: 0.0,
            active: false,
        }
    }

    pub fn note_on(&mut self) {
        self.target_amplitude = 0.3;
        self.active = true;
    }

    pub fn note_off(&mut self) {
        self.target_amplitude = 0.0;
    }

    pub fn next_sample(&mut self, sample_rate: f32) -> f32 {
        if !self.active && self.amplitude < 0.001 {
            return 0.0;
        }

        // Envelope (attack/release)
        if self.amplitude < self.target_amplitude {
            self.amplitude += 1.0 / (ATTACK_TIME * sample_rate);
            if self.amplitude > self.target_amplitude {
                self.amplitude = self.target_amplitude;
            }
        } else if self.amplitude > self.target_amplitude {
            self.amplitude -= 1.0 / (RELEASE_TIME * sample_rate);
            if self.amplitude < 0.0 {
                self.amplitude = 0.0;
                self.active = false;
            }
        }

        // Generate sine wave
        let sample = (self.phase * 2.0 * std::f32::consts::PI).sin() * self.amplitude;

        // Advance phase
        self.phase += self.frequency / sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }
}