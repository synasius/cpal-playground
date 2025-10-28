use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample, StreamConfig};

// import commonly used items from the prelude:
use rand::prelude::*;

pub enum WaveType {
    Sine,
    Square,
    SquareClassic,
    Sawtooth,
    Triangle,
    Noise,
}

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
        (period * 2.0 * std::f32::consts::PI).sin()
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

    fn tick(&mut self) -> f32 {
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
}

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("No output device found"))?;
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config: {config:?}");

    match config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::I32 => run::<i32>(&device, &config.into()),
        cpal::SampleFormat::I64 => run::<i64>(&device, &config.into()),
        cpal::SampleFormat::U8 => run::<u8>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
        cpal::SampleFormat::U32 => run::<u32>(&device, &config.into()),
        cpal::SampleFormat::U64 => run::<u64>(&device, &config.into()),
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::F64 => run::<f64>(&device, &config.into()),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

fn run<T>(device: &cpal::Device, config: &StreamConfig) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;

    let mut synth = Synth {
        wave_type: WaveType::Triangle,
        sample_rate: config.sample_rate.0 as f32,
        frequency: 440.0,
        sample_index: 0.0,
        rng: StdRng::from_os_rng(),
    };

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value: T = T::from_sample(synth.tick());
                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        },
        move |err| {
            eprintln!("an error occurred on the output audio stream: {}", err);
        },
        None,
    )?;

    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(1000));
    Ok(())
}
