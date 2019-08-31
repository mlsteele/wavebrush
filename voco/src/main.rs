#[macro_use] extern crate failure;

use lib::error::*;

use cpal::{Host, Device, Format, StreamData, UnknownTypeInputBuffer};
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
    // let output_stream_id = event_loop.build_output_stream();
    println!("event loop");
    event_loop.run(move |stream_id, stream_result| {
        if stream_id != input_stream_id {
            return
        }
        let data = match stream_result {
            Ok(data) => data,
            Err(err) => {
                eprintln!("stream error [{:?}]: {}", stream_id, err);
                return;
            }
        };
        match data {
            StreamData::Input{ buffer: UnknownTypeInputBuffer::F32(buffer) } => {
                for v in buffer.iter() {
                    if *v != 0.0 {
                        println!("stream data {:?}", v);
                    }
                }
                // if let Err(err) = on_input(&format_input, &*buffer) {
                //     eprintln!("stream error [{:?}]: {}", stream_id, err);
                // }
            }
            _ => eprintln!("unrecognized stream data"),
        }
    });
}

fn get_input(host: &Host) -> Result<(Device, Format)> {
    let device = host.default_input_device().ok_or(format_err!("no input device"))?;
    let formats: Vec<_> = device.supported_input_formats()?.collect();
    for format in formats.iter() { println!("{:?}", format); }
    let format = formats.iter().next().ok_or(format_err!("no input stream format"))?.clone().with_max_sample_rate();
    println!("selected input format: {:?}", format);
    Ok((device, format))
}

fn on_input(format: &cpal::Format, buf: &[f32]) -> EResult {
    use rodio::*;
    let device = rodio::default_output_device().ok_or(format_err!("output device"))?;
    let src = rodio::buffer::SamplesBuffer::new(1, format.sample_rate.0, buf);
    rodio::play_raw(&device, src.convert_samples());
    EOK
}

//     let settings = Settings{
//         sample_rate: reader_spec.sample_rate,
//         window_size: window_size as u32,
//         step_size: step_size as u32,
//     };

//     let mut shredder = Shredder::new(settings);
//     // Scan one channel of the audio.
//     let mut ax = 0;
//     for sample in reader.into_samples().step_by(reader_spec.channels as usize) {
//         ax += 1;
//         let sample: i16 = sample?;
//         let sample_f64: f64 = SampleConvert::convert(sample);
//         dont! {{
//             let freq: f64 = 775.2;
//             // let freq: f64 = 689.1;
//             // let freq: f64 = (689.1 + 775.2) / 2.;
//             let sample_f64: f64 = (ax as f64 / settings.sample_rate as f64 * 2. * PI * freq).sin() * 0.8; // sin wave
//             // dbg!(ax);
//             // dbg!(sample_f64);
//             // let sample_f64: f64 = (ax as f64 / 1000.).sin();
//         }}
//         shredder.append_samples(&[sample_f64])?;
//     }
//     println!("input length : {:?}", ax);

//     let mut sg = shredder.sg;

//     let mut unshredder = Unshredder::new(sg.clone());
//     let mut buf = unshredder.allocate_output_buf();
//     let mut ay = 0;
//     while unshredder.output(&mut buf)? {
//         for sample in &buf {
//             ay += 1;
//         };
//     }
//     println!("output length: {:?}", ay);


// }


// fn play(sg: Spectrogram) -> EResult {
//     use rodio::*;
//     let device = rodio::default_output_device().ok_or(format_err!("output device"))?;
//     let mut unshredder = Unshredder::new(sg.clone());
//     let mut buf = unshredder.allocate_output_buf();
//     let mut buf_all = Vec::new();
//     while unshredder.output(&mut buf)? {
//         buf_all.extend(buf.iter().map(|&x| x as f32));
//     }
//     let src = rodio::buffer::SamplesBuffer::new(1, sg.settings.sample_rate, buf_all);
//     rodio::play_raw(&device, src.convert_samples());
//     EOK
// }

// fn save<T>(sg: Spectrogram, mut writer: hound::WavWriter<T>) -> EResult
//     where T: std::io::Write + std::io::Seek
// {
//     let mut unshredder = Unshredder::new(sg.clone());
//     let mut buf = unshredder.allocate_output_buf();
//     while unshredder.output(&mut buf).unwrap() {
//         for sample in &buf {
//             writer.write_sample(SampleConvert::convert(*sample)).unwrap();
//         };
//     }
//     writer.finalize().unwrap();
//     EOK
// }
