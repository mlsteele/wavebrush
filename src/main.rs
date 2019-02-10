extern crate hound;
extern crate image;

mod stft;
mod bs;

use std::f64;
use stft::{STFT, WindowType};
use rustfft::{FFTplanner};
use num::complex::Complex;
use image::{DynamicImage};
use bs::*;

trait SampleConvert<X,Y> {
    fn convert(x: X) -> Y;
}

struct SampleConvertImpl {}

impl SampleConvert<i32, f64> for SampleConvertImpl {
    fn convert(x: i32) -> f64 {
        match x as f64 / std::i32::MAX as f64 {
            y if y > 1. => 1.,
            y if y < -1. => -1.,
            y => y,
        }
    }
}

impl SampleConvert<i16, f64> for SampleConvertImpl {
    fn convert(x: i16) -> f64 {
        match x as f64 / std::i16::MAX as f64 {
            y if y > 1. => 1.,
            y if y < -1. => -1.,
            y => y,
        }
    }
}

impl SampleConvert<f64, i16> for SampleConvertImpl {
    fn convert(x: f64) -> i16 {
        let max = std::i16::MAX;
        match (x * max as f64) as i16 {
            y if y > max => max,
            y if y < -max => -max,
            y => y,
        }
    }
}

fn rescale(v: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    return ((v - in_min) / (in_max - in_min) * (out_max - out_min)) + out_min;
}

fn within(x: f64, y: f64, threshold: f64) -> bool {
    (x - y).abs() <= threshold
}

fn fft_freq(i: usize, sample_rate: usize, fft_size: usize) -> f64 {
    let base = i as f64 * sample_rate as f64 / fft_size as f64;
    if i <= fft_size / 2 {
        base
    } else {
        sample_rate as f64 - base
    }
}

fn main() {
    let reader = hound::WavReader::open("440.wav").unwrap();
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
        let sample_f64: f64 = SampleConvertImpl::convert(sample);

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
            if let Some((i, _)) = buf.iter().enumerate().f64_max_by_key(|(_, sample)| sample.norm()) {
                println!("loudest freq: {}", fft_freq(i, reader_spec.sample_rate as usize, window_size));
            }

            // Band pass
            for i in 0..window_size {
                let freq = fft_freq(i, reader_spec.sample_rate as usize, window_size);
                buf[i] /= (freq-1000.).abs() + 1.;
                if within(freq, 800., 100.) {
                } else {
                    buf[i] *= 0.0;
                }
            }

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
                writer.write_sample(SampleConvertImpl::convert(sample.re / buf.len() as f64)).unwrap();
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
    DynamicImage::ImageRgb8(imgbuf).crop(0, h-(h/factor), w, h/factor).save("tmp/out.png").unwrap();
    writer.finalize().unwrap();
}
