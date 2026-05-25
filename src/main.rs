mod graph;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::prelude64::*;

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
        _ => panic!("unsupported sample format"),
    }
}

fn run<T: SizedSample + FromSample<f32>>(device: &cpal::Device, config: &cpal::StreamConfig) {
    let sample_rate = config.sample_rate as f64;
    let channels = config.channels as usize;

    let mut graph = graph::build();

    graph.set_sample_rate(sample_rate);
    graph.allocate();

    let mut next_value = move || graph.get_stereo();

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value);
            },
            |err| eprintln!("stream error: {err}"),
            None,
        )
        .unwrap();

    stream.play().unwrap();
    println!("Playing 440 Hz sine through low-pass filter. Press Enter to stop.");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

fn write_data<T: SizedSample + FromSample<f32>>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> (f32, f32),
) {
    for frame in output.chunks_mut(channels) {
        let (left, right) = next_sample();
        for (ch, sample) in frame.iter_mut().enumerate() {
            *sample = if ch & 1 == 0 {
                T::from_sample(left)
            } else {
                T::from_sample(right)
            };
        }
    }
}
