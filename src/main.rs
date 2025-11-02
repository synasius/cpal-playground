mod synth;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Ok;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample, Stream, StreamConfig};

use crossterm::event::{Event, KeyCode, KeyEvent, poll, read};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use synth::{Synth, WaveType};

// import commonly used items from the prelude:
use rand::prelude::*;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

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
    }?;

    disable_raw_mode()?;

    Ok(())
}

fn run<T>(device: &cpal::Device, config: &StreamConfig) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;

    let synth = Synth {
        wave_type: WaveType::Triangle,
        sample_rate: config.sample_rate.0 as f32,
        frequency: 440.0,
        sample_index: 0.0,
        rng: StdRng::from_os_rng(),
    };

    let synth = Arc::new(Mutex::new(synth));
    let synth_clone = synth.clone();

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value: T = T::from_sample(synth.lock().unwrap().tick());
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

    stream.pause()?;

    print_events(&stream, synth_clone)?;

    Ok(())
}

fn print_events(stream: &Stream, synth: Arc<Mutex<Synth>>) -> Result<(), anyhow::Error> {
    let mut is_playing = false;

    loop {
        if is_playing {
            stream.play()?;
        } else {
            stream.pause()?;
        }

        if poll(Duration::from_secs(0))? {
            // It's guaranteed that read() won't block if `poll` returns `Ok(true)`
            let event = read()?;

            if let Event::Key(KeyEvent { code, .. }) = event {
                match code {
                    KeyCode::Esc => break,

                    KeyCode::Char('q') => synth.lock().unwrap().set_frequency(261.63),
                    KeyCode::Char('w') => synth.lock().unwrap().set_frequency(293.66),
                    KeyCode::Char('e') => synth.lock().unwrap().set_frequency(329.63),
                    KeyCode::Char('r') => synth.lock().unwrap().set_frequency(349.23),
                    KeyCode::Char('t') => synth.lock().unwrap().set_frequency(392.00),
                    KeyCode::Char('y') => synth.lock().unwrap().set_frequency(440.00),
                    KeyCode::Char('u') => synth.lock().unwrap().set_frequency(493.88),
                    KeyCode::Char('i') => synth.lock().unwrap().set_frequency(523.25),

                    KeyCode::Char('1') => synth.lock().unwrap().set_wave(WaveType::Sine),
                    KeyCode::Char('2') => synth.lock().unwrap().set_wave(WaveType::Square),
                    KeyCode::Char('3') => synth.lock().unwrap().set_wave(WaveType::SquareClassic),
                    KeyCode::Char('4') => synth.lock().unwrap().set_wave(WaveType::Triangle),
                    KeyCode::Char('5') => synth.lock().unwrap().set_wave(WaveType::Sawtooth),
                    KeyCode::Char('6') => synth.lock().unwrap().set_wave(WaveType::Noise),

                    KeyCode::Char('z') => {
                        is_playing = !is_playing;
                    }

                    _ => {
                        continue;
                    }
                }
            }
        }
    }

    Ok(())
}
