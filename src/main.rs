#![allow(unused_imports)]

extern crate hound;
extern crate image;
#[macro_use] extern crate failure;

mod sample;
#[allow(dead_code)]
mod util;

#[allow(dead_code)]
mod stft;
use stft::{STFT, WindowType};

mod bs;
use bs::*;

mod ui;
use ui::*;

#[allow(dead_code)]
mod spectrogram;
use spectrogram::*;

mod error;

use std::f64;
use rustfft::{FFTplanner};
use num::complex::Complex;
use image::{DynamicImage};
use std::f64::consts::PI;
use sample::{SampleConvert,*};
use util::*;

fn main() {
    ui::run(main2());
}

type SpectroImage = image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>;

#[allow(unused_variables, dead_code)]
fn main2() -> SpectroImage {
    let reader = hound::WavReader::open("shortspeech.wav").unwrap();
    let reader_spec = reader.spec().clone();
    println!("spec: {:?}", reader_spec);
    assert_eq!(reader_spec.bits_per_sample, 16); // type inference is used to convert samples

    let mut out_spec = reader.spec().clone();
    out_spec.channels = 1;
    let mut writer = hound::WavWriter::create("tmp/out.wav", out_spec).unwrap();

    let window_type: WindowType = WindowType::Hanning;
    // let window_size: usize = 1024; // When this isn't a power of two garbage comes out.
    let window_size: usize = (2 as usize).pow(8); // When this isn't a power of two garbage comes out.
    // let window_size: usize = 1024;
    // let window_size: usize = reader_spec.sample_rate as usize / 100;
    let step_size: usize = window_size / 2;
    // let step_size: usize = 16;
    // let step_size: usize = reader_spec.sample_rate as usize / 4;
    // let step_size: usize = 32;
    let mut stft = STFT::new(window_type, window_size, step_size);
    println!("window_size: {:?}", window_size);
    println!("window_sec: {:?}", window_size as f64 / reader_spec.sample_rate as f64);
    println!("step_size: {:?}", step_size);

    let settings = Settings{
        sample_rate: reader_spec.sample_rate,
        window_size: window_size as u32,
    };
    let mut sp = Spectrogram::new(settings);

    let ifft = FFTplanner::<f64>::new(true).plan_fft(window_size);

    let mut buf: Vec<Complex<f64>>  = vec![Default::default(); window_size];
    let mut buf2: Vec<Complex<f64>> = vec![Default::default(); window_size];
    let overlap_size = window_size - step_size;
    let mut buf_overlap: Vec<Complex<f64>> = vec![Default::default(); overlap_size];

    // Scan one channel of the audio.
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

            sp.push_front(buf.clone()).expect("push column");

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
    // imgbuf.crop(0, 0, 100, 100).save("tmp/out.png").unwrap();
    let imgbuf = sp.image().expect("image");
    let w = imgbuf.width();
    let h = imgbuf.height();
    let factor = 1;
    DynamicImage::ImageRgb8(imgbuf.clone()).crop(0, h-(h/factor), w, h/factor).save("tmp/out.png").unwrap();
    writer.finalize().unwrap();
    imgbuf
}
