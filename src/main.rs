#![allow(unused_imports)]

extern crate hound;
extern crate image;

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
    let reader = hound::WavReader::open("speech.wav").unwrap();
    let reader_spec = reader.spec().clone();
    println!("spec: {:?}", reader_spec);
    assert_eq!(reader_spec.bits_per_sample, 16); // type inference is used to convert samples

    let mut out_spec = reader.spec().clone();
    out_spec.channels = 1;
    let mut writer = hound::WavWriter::create("tmp/out.wav", out_spec).unwrap();

    let window_type: WindowType = WindowType::Hanning;
    // let window_size: usize = 1024; // When this isn't a power of two garbage comes out.
    let window_size: usize = (2 as usize).pow(9); // When this isn't a power of two garbage comes out.
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

    let ifft = FFTplanner::<f64>::new(true).plan_fft(window_size);

    let mut buf: Vec<Complex<f64>>  = vec![Default::default(); window_size];
    let mut buf2: Vec<Complex<f64>> = vec![Default::default(); window_size];
    let overlap_size = window_size - step_size;
    let mut buf_overlap: Vec<Complex<f64>> = vec![Default::default(); overlap_size];

    let mut imgbuf = image::ImageBuffer::new(1028, window_size as u32 / 2);
    // let mut imgbuf = image::DynamicImage::new_rgb8(1028, window_size as u32);
    let mut img_x = 0;

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

            if img_x < imgbuf.width() {
                // let morphed: Vec<_> = buf.iter().map(|v| v.norm().log10()).collect();
                let morphed: Vec<_> = buf[..window_size/2].iter().map(|v| (v.norm() + 0.).log10())
                    .map(|v| if v > 0. {v} else {0.}).collect();
                let min = morphed.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = morphed.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                // if img_x == 800 {
                //     println!("{:?} -> {:?}", min, max);
                // }
                for (i, &v) in morphed.iter().enumerate() {
                    let sv = rescale(v, min, max, 0., 1.);
                    // let sv = rescale(v, 0., 2., 0., 1.);
                    let pixel = imgbuf.get_pixel_mut(img_x, imgbuf.height()-1-i as u32);
                    // if img_x == 800 {
                    //     println!("{:?}", sv);
                    // }
                    *pixel = image::Rgb([
                        rescale(sv, 0., 1., 0., 245.) as u8 + 10,
                        rescale(sv, 0., 1., 0., 190.) as u8,
                        rescale(sv, 0., 1., 0., 80.) as u8 + 20,
                    ]);
                }
            }

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
            img_x += 1;
            frame+=1;
        }
    }
    // imgbuf.crop(0, 0, 100, 100).save("tmp/out.png").unwrap();
    let w = imgbuf.width();
    let h = imgbuf.height();
    let factor = 1;
    DynamicImage::ImageRgb8(imgbuf.clone()).crop(0, h-(h/factor), w, h/factor).save("tmp/out.png").unwrap();
    writer.finalize().unwrap();
    imgbuf
}
