#![allow(unused_imports)]

extern crate hound;
extern crate rodio;
extern crate image;
#[macro_use] extern crate failure;
use failure::*;

mod sample;
#[allow(dead_code)]
mod util;

#[allow(dead_code)]
mod stft;
use stft::{STFT, WindowType};

#[allow(dead_code)]
mod control;
use control::*;

mod bs;
use bs::*;

mod ui;
use ui::*;

#[allow(dead_code)]
mod colorramp;

#[allow(dead_code)]
mod spectrogram;
use spectrogram::*;

#[allow(dead_code)]
mod shredder;
use shredder::*;

#[allow(dead_code)]
mod edit;
use edit::*;

mod error;
use error::*;

use std::f64;
use rustfft::{FFTplanner};
use num::complex::Complex;
use image::{DynamicImage};
use std::f64::consts::PI;
use sample::{SampleConvert,*};
use util::*;
use std::thread;
use std::env;

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
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).map(|x| x.to_owned()).unwrap_or_else(|| "card.wav".to_owned());

    let reader = hound::WavReader::open(filename).context("reading input wav")?;
    let reader_spec = reader.spec().clone();
    println!("spec: {:?}", reader_spec);
    ensure_eq!(reader_spec.bits_per_sample, 16, "sample size"); // type inference is used to convert samples

    let mut out_spec = reader.spec().clone();
    out_spec.channels = 1;

    let window_type: WindowType = WindowType::Hanning;
    let window_size: usize = (2 as usize).pow(9); // When this isn't a power of two garbage comes out.
    // let window_size: usize = 1024;
    // let window_size: usize = reader_spec.sample_rate as usize / 100;
    // let step_size: usize = (window_size / 2) / 8;
    let step_size: usize = window_size / 2;
    // let step_size: usize = 16;
    // let step_size: usize = reader_spec.sample_rate as usize / 4;
    // let step_size: usize = 32;
    println!("window_size: {:?}", window_size);
    println!("window_sec: {:?}", window_size as f64 / reader_spec.sample_rate as f64);
    println!("step_size: {:?}", step_size);

    let settings = Settings{
        sample_rate: reader_spec.sample_rate,
        window_size: window_size as u32,
        step_size: step_size as u32,
    };

    let mut shredder = Shredder::new(settings);
    // Scan one channel of the audio.
    let mut ax = 0;
    for sample in reader.into_samples().step_by(reader_spec.channels as usize) {
        ax += 1;
        let sample: i16 = sample?;
        let sample_f64: f64 = SampleConvert::convert(sample);
        let freq: f64 = 775.2;
        // let freq: f64 = 689.1;
        // let freq: f64 = (689.1 + 775.2) / 2.;
        let sample_f64: f64 = (ax as f64 / settings.sample_rate as f64 * 2. * PI * freq).sin() * 0.8; // sin wave
        // dbg!(ax);
        // dbg!(sample_f64);
        // let sample_f64: f64 = (ax as f64 / 1000.).sin();
        shredder.append_samples(&[sample_f64])?;
    }
    println!("input length : {:?}", ax);

    let mut sg = shredder.sg;

    // Rewrite the whole sg.
    // Wrapper::new(&mut sg, &Default::default()).nuke();
    // for x in 0..sg.width() {
    //     Wrapper::new(&mut sg, &Default::default()).effect_dual(x, 4, |v, flop| {
    //         let r = 50.9;
    //         let theta = 0.;
    //         // let theta = PI * (x as f64);
    //         *v = Complex::from_polar(&r, &theta);
    //         if flop {*v = v.conj()};
    //     });
    //     // for y in 0..settings.window_size {
    //     //     let y = y as i32;
    //     //     let freq = sg.freq(y);
    //     //     let v = sg.get_mut(x, y).expect("sg mod");
    //     //     let (mut r, mut theta) = v.to_polar();
    //     //     if y == 5 {
    //     //         r = 50.9;
    //     //         theta = PI * (x as f64);
    //     //     } else {
    //     //         r = 0.;
    //     //         theta = 0.;
    //     //     }
    //     //     *v = Complex::from_polar(&r, &theta);
    //     // }
    // }

    // Print a column
    for y in 0..settings.window_size {
        let y = y as i32;
        let v = sg.get(100, y).expect("sg inspect");
        let (r, theta) = v.to_polar();
        println!("y:{:3} freq:{:9.2} r:{:5.1} theta:{:5.2}", y, sg.freq(y), r, theta);
    }
    println!("end column");

    // Print some rows..
    for x in 0..sg.width() {
        for y in 7..11 {
            let v = sg.get(x, y).expect("sg inspect");
            let (r, theta) = v.to_polar();
            let freq = sg.freq(y);
            println!("x:{:4} y:{:3} freq:{:9.2} r:{:5.1} theta:{:5.2} {}",
                     x, y, sg.freq(y), r, theta, rad_clock(theta));
        }
        println!("----------");
    }

    let sg_reset = sg.clone();

    let mut unshredder = Unshredder::new(sg.clone());
    let mut buf = unshredder.allocate_output_buf();
    let mut ay = 0;
    while unshredder.output(&mut buf)? {
        for sample in &buf {
            ay += 1;
        };
    }
    println!("output length: {:?}", ay);

    // Wrapper::new(&mut sg, &Default::default()).scratch();
    let imgbuf = sg.image()?;

    let (uictl, ctl) = new_ctl();

    // The UI can only run on the main thread on macos.
    use control::ToBackend::*;

    let xxxctl = uictl.clone();
    thread::spawn(move || {
        xxxctl.send(Save);
        xxxctl.send(Play);
    });

    let h = thread::spawn(move || {
        use control::Sliders;
        let mut sliders: Sliders = Default::default();
        for msg in ctl.r.iter() {match msg {
            Info{x, y} => {
                if let Some((freq, a, b)) = sg.get_dual(x, y) {
                    ctl.send(ToUI::Info{freq, a: *a, b: *b});
                }
            },
            Prod{x, y} => {
                Wrapper::new(&mut sg, &sliders).airbrush(x, y);
                ctl.send(ToUI::Spectrogram(sg.image().expect("render image")));
            },
            Erase{x, y} => {
                Wrapper::new(&mut sg, &sliders).erase(x, y);
                ctl.send(ToUI::Spectrogram(sg.image().expect("render image")));
            },
            Nuke => {
                Wrapper::new(&mut sg, &sliders).nuke();
                ctl.send(ToUI::Spectrogram(sg.image().expect("render image")));
            },
            Sliders(s) => {
                sliders = s;
            },
            Play => {
                println!("<- play");
                if let Err(err) = play(sg.clone()) {
                    println!("play failed: {:?}", err);
                }
            },
            Save => {
                println!("<- save");

                // Sav wav
                let writer = hound::WavWriter::create("tmp/out.wav", out_spec).unwrap();
                if let Err(err) = save(sg.clone(), writer) {
                    println!("save failed: {:?}", err);
                } else {
                    println!("save complete");
                }

                // Save image
                // imgbuf.crop(0, 0, 100, 100).save("tmp/out.png").unwrap();
                let imgbuf = sg.image().expect("image");
                let w = imgbuf.width();
                let h = imgbuf.height();
                println!("sg dimensions {} {}", w, h);
                // let factor = 1;
                // DynamicImage::ImageRgb8(imgbuf.clone()).crop(0, h-(h/factor), w, h/factor).save("tmp/out.png").unwrap();
                imgbuf.clone().save("tmp/out.png").unwrap();
            },
            Reset => {
                sg = sg_reset.clone();
                ctl.send(ToUI::Spectrogram(sg.image().expect("render image")));
            },
            Quit => break,
        }}
    });

    ui::run(uictl, imgbuf);

    h.join().expect("ui thread join");
    EOK
}

fn play(sg: Spectrogram) -> EResult {
    use rodio::*;
    let device = rodio::default_output_device().ok_or(format_err!("output device"))?;
    let mut unshredder = Unshredder::new(sg.clone());
    let mut buf = unshredder.allocate_output_buf();
    let mut buf_all = Vec::new();
    while unshredder.output(&mut buf)? {
        buf_all.extend(buf.iter().map(|&x| x as f32));
    }
    let src = rodio::buffer::SamplesBuffer::new(1, sg.settings.sample_rate, buf_all);
    rodio::play_raw(&device, src.convert_samples());
    EOK
}

fn save<T>(sg: Spectrogram, mut writer: hound::WavWriter<T>) -> EResult
    where T: std::io::Write + std::io::Seek
{
    let mut unshredder = Unshredder::new(sg.clone());
    let mut buf = unshredder.allocate_output_buf();
    while unshredder.output(&mut buf).unwrap() {
        for sample in &buf {
            writer.write_sample(SampleConvert::convert(*sample)).unwrap();
        };
    }
    writer.finalize().unwrap();
    EOK
}

/*
// The old loop
let mut frame = 0;
for sample in reader.into_samples().step_by(reader_spec.channels as usize) {
    let sample: i16 = sample.unwrap();
    let sample_f64: f64 = SampleConvert::convert(sample);

    stft.append_samples(&[sample_f64]);
    while stft.contains_enough_to_compute() {
        stft.compute_into_complex_output();
        buf.copy_from_slice(&stft.complex_output);
        // let buf_len = buf.len();
        // for sample in buf.iter_mut() {
        //     *sample /= buf_len as f64;
        // }

        sg.push_front(buf.clone()).expect("push column");

        // Effect in frequency space.
        // for sample in buf.iter_mut() {
        //     sample.re *= sample.im;
        // }

        // if frame == 500 {
        //     for (i, sample) in buf.iter().enumerate() {
        //         if sample.norm() > 0.5 {
        //             println!("{}: {} ({} hz)", i, sample.norm(),
        //                      fft_freq(i, reader_spec.sample_rate as usize, window_size));
        //         }
        //     }
        // }

        // Loudest frequency
        // if let Some((i, _)) = buf.iter().enumerate().f64_max_by_key(|(_, sample)| sample.norm()) {
        //     println!("loudest freq: {}", fft_freq(i, reader_spec.sample_rate as usize, window_size));
        // }

        // Band pass
        // for i in 0..window_size {
        //     let freq = fft_freq(i, reader_spec.sample_rate as usize, window_size);
        //     buf[i] /= (freq-1000.).abs() + 1.;
        //     if within(freq, 800., 100.) {
        //     } else {
        //         buf[i] *= 0.0;
        //     }
        // }

        // Lose phase information
        // for (i, sample) in buf.iter_mut().enumerate() {
        //     let (mut r, mut theta) = sample.to_polar();
        //     theta = 0.;
        //     *sample = Complex::from_polar(&r, &theta);
        // }

        // Simulate phase recovery.
        // for (i, sample) in buf.iter_mut().enumerate() {
        //     let (mut r, mut theta) = sample.to_polar();
        //     let freq = fft_freq(i, reader_spec.sample_rate as usize, window_size);
        //     theta = -2. * PI * frame as f64 / reader_spec.sample_rate as f64 * freq;
        //     *sample = Complex::from_polar(&r, &theta);
        // }

        // buf2.copy_from_slice(&buf);
        // let hw = window_size / 2;
        // for i in 0..window_size {
        //     let (mut r, mut theta) = buf[i].to_polar();
        //     // r *= (i as i32 - hw as i32).abs() as f64 / hw as f64;
        //     theta *= (i as i32 - hw as i32).abs() as f64 / hw as f64;
        //     buf[i] = Complex::from_polar(&r, &theta);
        // }

        ifft.process(&mut buf, &mut buf2);
        // overlap += buf2[..w-s];
        for i in 0..window_size-step_size {
            buf_overlap[i] += buf2[i];
        }
        // shipit(overlap);
        for sample in &buf_overlap {
            writer.write_sample(SampleConvert::convert(sample.re / buf.len() as f64)).unwrap();
        };
        // overlap = buf2[w-s..];
        buf_overlap.copy_from_slice(&buf2[step_size..]);

        stft.move_to_next_column();
        frame+=1;
    }
}
*/
