use rand::prelude::*;

pub enum WaveType {
    Sine,
    Square,
    SquareClassic,
    Sawtooth,
    Triangle,
    Noise,
}

const TWO_PI: f32 = 2.0 * std::f32::consts::PI;

pub struct Synth {
    pub wave_type: WaveType,
    pub sample_rate: f32,
    pub frequency: f32,
    pub sample_index: f32,
    pub rng: StdRng,
}

impl Synth {
    fn sine_wave(&self, frequency: f32) -> f32 {
        let period = self.sample_index * frequency / self.sample_rate;
        (period * TWO_PI).sin()
    }

    fn additive_harmonics(&self, step: f32, exponent: f32) -> f32 {
        let mut output = 0.0;
        let mut i = 1.0;
        while self.frequency * i < (self.sample_rate / 2.0) {
            let gain = self.sine_wave(self.frequency * i) / i.powf(exponent);
            output += gain;
            i += step;
        }

        output
    }

    fn triangle_wave(&self) -> f32 {
        self.additive_harmonics(2.0, 2.0)
    }

    fn saw_wave(&self) -> f32 {
        self.additive_harmonics(1.0, 1.0)
    }

    fn square_by_additive_harmonics(&self) -> f32 {
        self.additive_harmonics(2.0, 1.0)
    }

    fn square_classic(&self) -> f32 {
        let period = self.sample_rate / self.frequency;
        if self.sample_index / period <= 0.5 {
            -0.5
        } else {
            0.5
        }
    }

    fn noise(&mut self) -> f32 {
        let noise = self.rng.random::<f32>() * 2.0 - 1.0;
        noise
    }

    pub fn tick(&mut self) -> f32 {
        // here we advance to the next sample index
        let period = self.sample_rate / self.frequency;
        self.sample_index = (self.sample_index + 1.0) % period;

        match self.wave_type {
            WaveType::Sine => self.sine_wave(self.frequency),
            WaveType::Square => self.square_by_additive_harmonics(),
            WaveType::SquareClassic => self.square_classic(),
            WaveType::Sawtooth => self.saw_wave(),
            WaveType::Triangle => self.triangle_wave(),
            WaveType::Noise => self.noise(),
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn set_wave(&mut self, wave: WaveType) {
        self.wave_type = wave;
    }
}
