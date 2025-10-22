extern crate anyhow;
extern crate cpal;

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample, StreamConfig};

// import commonly used items from the prelude:
use rand::prelude::*;

pub enum WaveType {
    Square = 0,
    Sawtooth = 1,
    Sine = 2,
    Noise = 3,
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

    let frequency = 310.0;
    let period_speed = frequency / sample_rate;
    let mut period_position = 0.0;
    let mut noise_buffer: [f32; 32] = [0.0; 32];
    let wave_type = WaveType::Noise;

    // init the noise buffer
    for i in 0..32 {
        noise_buffer[i] = rng.random::<f32>() * 2.0 - 1.0;
    }

    let mut next_value = move || {
        period_position += period_speed;
        if period_position >= 1.0 {
            period_position -= 1.0;
            let mut rng = rand::rng();
            for i in 0..32 {
                noise_buffer[i] = rng.random::<f32>() * 2.0 - 1.0;
            }
        }
        match wave_type {
            // Produce a square wave
            WaveType::Square => {
                if period_position <= 0.5 {
                    -0.5
                } else {
                    0.5
                }
            }
            WaveType::Sawtooth => 1.0 - period_position * 2.0,
            // Produce a sinusoid of maximum amplitude.
            WaveType::Sine => (period_position * 2.0 * std::f32::consts::PI).sin(),
            WaveType::Noise => noise_buffer[(period_position * 31.0) as usize],
            _ => 0.0,
        }
    };

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value: T = T::from_sample(next_value());
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
