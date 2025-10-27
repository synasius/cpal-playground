use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample, StreamConfig};

// import commonly used items from the prelude:
use rand::prelude::*;

pub enum WaveType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Noise,
}

pub struct Synth {
    pub wave_type: WaveType,
    pub sample_rate: f32,
    pub frequency: f32,
    pub sample_index: f32,
    pub sample_position: f32,
}

impl Synth {
    fn sine_wave(&self, frequency: f32) -> f32 {
        let period = self.sample_index * frequency / self.sample_rate;
        (period * 2.0 * std::f32::consts::PI).sin()
    }

    fn square_by_additive_harmonics(&self) -> f32 {
        let mut output = 0.0;
        let mut i = 1.0;
        while self.frequency * i < (self.sample_rate / 2.0) {
            let gain = self.sine_wave(self.frequency * i) / i;
            output += gain;
            i += 2.0;
        }

        output
    }

    fn square_classic(&self) -> f32 {
        let period = self.sample_rate / self.frequency;
        if self.sample_index % period / period <= 0.5 {
            // if self.sample_position <= 0.5 {
            -0.5
        } else {
            0.5
        }
    }

    fn tick(&mut self) -> f32 {
        // here we advance to the next sample index
        self.sample_index = (self.sample_index + 1.0) % self.sample_rate;

        self.sample_position += self.frequency / self.sample_rate;
        if self.sample_position >= 1.0 {
            println!(
                "index {0} and position {1}",
                self.sample_index, self.sample_position
            );
            self.sample_position -= 1.0;
        }

        match self.wave_type {
            WaveType::Sine => self.sine_wave(self.frequency),
            // WaveType::Square => self.square_by_additive_harmonics(),
            WaveType::Square => self.square_classic(),
            WaveType::Sawtooth => todo!(),
            WaveType::Triangle => todo!(),
            WaveType::Noise => todo!(),
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
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut rng = rand::rng();

    let frequency = 440.0;
    let period_speed = frequency / sample_rate;
    let mut noise_buffer: [f32; 32] = [0.0; 32];
    let wave_type = WaveType::Square;

    let mut synth = Synth {
        wave_type,
        sample_rate,
        frequency,
        sample_index: 0.0,
        sample_position: 0.0,
    };

    // init the noise buffer
    for i in 0..32 {
        noise_buffer[i] = rng.random::<f32>() * 2.0 - 1.0;
    }

    // let mut next_value = move || {
    //     if sample_index > sample_rate {
    //         // refresh the noise buffer
    //         let mut rng = rand::rng();
    //         for i in 0..32 {
    //             noise_buffer[i] = rng.random::<f32>() * 2.0 - 1.0;
    //         }
    //     }

    //     sample_index = (sample_index + 1.0) % sample_rate;

    //     let position_in_period = sample_index * period_speed;
    //     match wave_type {
    //         // Produce a square wave
    //         WaveType::Square => {
    //             if position_in_period <= 0.5 {
    //                 -0.5
    //             } else {
    //                 0.5
    //             }
    //         }
    //         WaveType::Sawtooth => 1.0 - position_in_period * 2.0,
    //         // Produce a sinusoid of maximum amplitude.
    //         WaveType::Sine => (position_in_period * 2.0 * std::f32::consts::PI).sin(),
    //         WaveType::Noise => noise_buffer[(position_in_period * 31.0) as usize],
    //         _ => 0.0,
    //     }
    // };

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
