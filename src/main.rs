extern crate hound;
extern crate stft;
extern crate image;

use std::f64;
use stft::{STFT, WindowType};
use rustfft::{FFTplanner};
use num::complex::Complex;
use image::{DynamicImage};

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

fn main() {
    let reader = hound::WavReader::open("mono.wav").unwrap();
    let reader_spec = reader.spec().clone();
    println!("spec: {:?}", reader_spec);
    assert_eq!(reader_spec.bits_per_sample, 16); // type inference is used to convert samples

    let mut out_spec = reader.spec().clone();
    out_spec.channels = 1;
    let mut writer = hound::WavWriter::create("tmp/out.wav", out_spec).unwrap();

    let mut buf: Vec<Complex<f64>> = Vec::new();

    // Read the whole wav
    for sample in reader.into_samples().step_by(reader_spec.channels as usize) {
        let sample: i16 = sample.unwrap();
        let sample_f64: f64 = SampleConvertImpl::convert(sample);
        buf.push(Complex::new(sample_f64, 0.));
    }
    println!("buf len: {:?}", buf.len());

    // Zero pad (but why?)
    // buf.extend(vec![Complex::default(); buf.len()]);

    let mut buf2: Vec<Complex<f64>> = vec![Default::default(); buf.len()];

    let fft = FFTplanner::<f64>::new(false).plan_fft(buf.len());
    let ifft = FFTplanner::<f64>::new(true).plan_fft(buf.len());

    fft.process(&mut buf, &mut buf2);
    // for sample in &buf2 {
    //     println!("{:?}", sample.norm());
    // }
    ifft.process(&mut buf2, &mut buf);

    for sample in &buf {
        // println!("{:?}", sample);
        writer.write_sample(SampleConvertImpl::convert(sample.re / 2000000.)).unwrap();
    };
    writer.finalize().unwrap();
}

fn main2() {
    let reader = hound::WavReader::open("shortspeech.wav").unwrap();
    let reader_spec = reader.spec().clone();
    println!("spec: {:?}", reader_spec);
    assert_eq!(reader_spec.bits_per_sample, 16); // type inference is used to convert samples

    let mut out_spec = reader.spec().clone();
    out_spec.channels = 1;
    let mut writer = hound::WavWriter::create("tmp/out.wav", out_spec).unwrap();

    let window_type: WindowType = WindowType::Hanning;
    let window_size: usize = 1024; // When this isn't a power of two garbage comes out.
    // let window_size: usize = 1024;
    // let window_size: usize = reader_spec.sample_rate as usize / 100;
    // let step_size: usize = window_size / 2;
    let step_size: usize = 32;
    let mut stft = STFT::new(window_type, window_size, step_size);
    println!("window_size: {:?}", window_size);
    println!("step_size: {:?}", step_size);
    println!("stft output size: {:?}", stft.output_size());

    // let ifft = FFTplanner::<f64>::new(true).plan_fft(stft.output_size());

    let mut buf: Vec<Complex<f64>>  = vec![Default::default(); stft.output_size()];
    // let mut buf2: Vec<Complex<f64>> = vec![Default::default(); stft.output_size()];
    // let mut buf_carryover: Vec<f64> = vec![Default::default(); window_size - step_size];

    let mut imgbuf = image::ImageBuffer::new(1028, stft.output_size() as u32);
    // let mut imgbuf = image::DynamicImage::new_rgb8(1028, stft.output_size() as u32);
    let mut img_x = 0;

    // Scan one channel of the audio.
    for sample in reader.into_samples().step_by(reader_spec.channels as usize) {
        let sample: i16 = sample.unwrap();
        let sample_f64: f64 = SampleConvertImpl::convert(sample);

        stft.append_samples(&[sample_f64]);
        while stft.contains_enough_to_compute() {
            stft.compute_complex_column(&mut buf[..]);

            if img_x < imgbuf.width() {
                // let morphed: Vec<_> = buf.iter().map(|v| v.norm().log10()).collect();
                let morphed: Vec<_> = buf.iter().map(|v| (v.norm() + 0.).log10())
                    .map(|v| if v > 0. {v} else {0.}).collect();
                let min = morphed.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max = morphed.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                // if img_x == 800 {
                    println!("{:?} -> {:?}", min, max);
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

            // ifft.process(&mut buf, &mut buf2);
            // for sample in &buf2 {
            //     writer.write_sample(SampleConvertImpl::convert(sample.re)).unwrap();
            // };

            stft.move_to_next_column();
            img_x += 1;
        }
    }
    // imgbuf.crop(0, 0, 100, 100).save("tmp/out.png").unwrap();
    let w = imgbuf.width();
    let h = imgbuf.height();
    let factor = 1;
    DynamicImage::ImageRgb8(imgbuf).crop(0, h-(h/factor), w, h/factor).save("tmp/out.png").unwrap();
    writer.finalize().unwrap();
}
