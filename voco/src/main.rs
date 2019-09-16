#[macro_use] extern crate failure;

use lib::error::*;
use lib::spectrogram::*;
use lib::shredder::*;
use lib::sample::{SampleConvert,*};

use cpal::{Host, Device, Format, StreamData};
use cpal::traits::{DeviceTrait,HostTrait,EventLoopTrait};

fn main() {
    if let Err(err) = main2() {
        // eprintln!("{}", pretty_error(&err));
        eprint_error(&err);
        std::process::exit(1);
    }
}

fn eprint_error(err: &failure::Error) {
    eprintln!("");
    for err in err.iter_chain() {
        eprintln!("{}", err);
    }
    eprintln!("\n{:?}", err.backtrace());
}

// https://github.com/BurntSushi/imdb-rename/blob/f2a40bf/src/main.rs#L345-L355
/// Return a prettily formatted error, including its entire causal chain.
#[allow(dead_code)]
fn pretty_error(err: &failure::Error) -> String {
    let mut pretty = err.to_string();
    let mut prev = err.as_fail();
    while let Some(next) = prev.cause() {
        pretty.push_str(": ");
        pretty.push_str(&next.to_string());
        prev = next;
    }
    pretty
}

#[allow(unused_variables)]
fn main2() -> EResult {
    let host = cpal::default_host();
    let event_loop = host.event_loop();
    let (device_input, format_input) = get_input(&host)?;
    let input_stream_id = event_loop.build_input_stream(&device_input, &format_input)?;
    let (device_output, format_output) = get_output(&host)?;
    let output_stream_id = event_loop.build_output_stream(&device_output, &format_output)?;
    println!("event loop");

    let window_size: usize = (2 as usize).pow(14); // When this isn't a power of two garbage comes out.
    let step_size: usize = window_size / 2;
    let settings = Settings{
        sample_rate: format_input.sample_rate.0,
        window_size: window_size as u32,
        step_size: step_size as u32,
    };
    let mut shredder = Shredder::new(settings);

    event_loop.play_stream(output_stream_id.clone())?; // unclear if this is necessary or works

    event_loop.run(move |stream_id, stream_result| {
        let data = match stream_result {
            Ok(data) => data,
            Err(err) => {
                eprintln!("stream error [{:?}]: {}", stream_id, err);
                return;
            }
        };
        match data {
            StreamData::Input{ buffer: cpal::UnknownTypeInputBuffer::F32(buffer) } => {
                if stream_id != input_stream_id { return }
                println!("+ input {}", buffer.len());
                let converted: Vec<f64> = buffer.iter().map(|x| SampleConvert::convert(*x)).collect();
                shredder.append_samples(&converted).expect("processing input samples");
                println!("  sg size {}", shredder.sg.data.len());
            },
            StreamData::Output{ buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                if stream_id != output_stream_id { return }
                println!("- output {}", buffer.len());
                if buffer.len() == 0 {
                    println!("skipping empty output buffer");
                    return
                }
                let mut unshredder = Unshredder2::new(settings);
                let mut buf = vec![Default::default(); buffer.len()];
                let have = unshredder.output(&mut shredder.sg, &mut buf).expect("unshredder.output");
                if have {
                    for (sample, buf_sample) in buffer.iter_mut().zip(buf) {
                        *sample = SampleConvert::convert(buf_sample);
                    }
                } else {
                    println!("  no sg data ready")
                }
            },
            _ => eprintln!("unrecognized stream data"),
        }
    });
}

fn get_input(host: &Host) -> Result<(Device, Format)> {
    let device = host.default_input_device().ok_or(format_err!("no input device"))?;
    // let formats: Vec<_> = device.supported_input_formats()?.collect();
    // for format in formats.iter() { println!("{:?}", format); }
    // let format = formats.iter().next().ok_or(format_err!("no input stream format"))?.clone().with_max_sample_rate();
    let format = device.default_input_format()?;
    println!("selected input format: {:?}", format);
    Ok((device, format))
}

fn get_output(host: &Host) -> Result<(Device, Format)> {
    let device = host.default_output_device().ok_or(format_err!("no input device"))?;
    let format = device.default_output_format()?;
    println!("selected output format: {:?}", format);
    Ok((device, format))
}