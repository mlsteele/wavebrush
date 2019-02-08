extern crate hound;
extern crate stft;
extern crate image;

use stft::{STFT, WindowType};

fn main() {
    let reader = hound::WavReader::open("mono.wav").unwrap();
    let reader_spec = reader.spec().clone();
    println!("spec: {:?}", reader_spec);

    let window_type: WindowType = WindowType::Hanning;
    let window_size: usize = 1024;
    let step_size: usize = window_size / 2;
    let mut stft = STFT::new(window_type, window_size, step_size);
    println!("stft output size: {:?}", stft.output_size());

    let mut spectrogram_column: Vec<f64> =
        std::iter::repeat(0.).take(stft.output_size()).collect();

    let mut imgbuf = image::ImageBuffer::new(4096, stft.output_size() as u32);
    let mut img_x = 0;

    let mut out_spec = reader.spec().clone();
    out_spec.channels = 1;
    let mut writer = hound::WavWriter::create("tmp/out.wav", out_spec).unwrap();
    // Scan one channel of the audio.
    for sample in reader.into_samples().step_by(reader_spec.channels as usize) {
        let sample: i32 = sample.unwrap();
        stft.append_samples(&[sample as f64]);
        while stft.contains_enough_to_compute() {
            if img_x >= imgbuf.width() {
                break
            }
            stft.compute_column(&mut spectrogram_column[..]);
            for (i, &sv) in spectrogram_column.iter().enumerate() {
                let pixel = imgbuf.get_pixel_mut(img_x, i as u32);
                *pixel = image::Rgb([
                    (sv * 30.) as u8,
                    (sv * 20.) as u8,
                    (sv * 10.) as u8,
                ]);
            }
            stft.move_to_next_column();
            img_x += 1;
        }
        // writer.write_sample(sample);
    }
    imgbuf.save("tmp/out.png").unwrap();
    writer.finalize().unwrap();
}
