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
    let step_size: usize = 512;
    let mut stft = STFT::new(window_type, window_size, step_size);
    println!("stft output size: {:?}", stft.output_size());

    let mut spectrogram_column: Vec<f64> =
        std::iter::repeat(0.).take(stft.output_size()).collect();

    let mut imgbuf = image::ImageBuffer::new(800, stft.output_size() as u32);
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

fn stft_example() {
    let sample_rate: usize = 44100;
    let seconds: usize = 10;
    let sample_count = sample_rate * seconds;
    let all_samples = (0..sample_count).map(|x| x as f64).collect::<Vec<f64>>();

    // let's initialize our short-time fourier transform
    let window_type: WindowType = WindowType::Hanning;
    let window_size: usize = 1024;
    let step_size: usize = 512;
    let mut stft = STFT::new(window_type, window_size, step_size);

    // we need a buffer to hold a computed column of the spectrogram
    let mut spectrogram_column: Vec<f64> =
        std::iter::repeat(0.).take(stft.output_size()).collect();

    // iterate over all the samples in chunks of 3000 samples.
    // in a real program you would probably read from something instead.
    for some_samples in (&all_samples[..]).chunks(3000) {
        // append the samples to the internal ringbuffer of the stft
        stft.append_samples(some_samples);

        // as long as there remain window_size samples in the internal
        // ringbuffer of the stft
        while stft.contains_enough_to_compute() {
            // compute one column of the stft by
            // taking the first window_size samples of the internal ringbuffer,
            // multiplying them with the window,
            // computing the fast fourier transform,
            // taking half of the symetric complex outputs,
            // computing the norm of the complex outputs and
            // taking the log10
            stft.compute_column(&mut spectrogram_column[..]);

            println!("{:?}", spectrogram_column);

            // here's where you would do something with the
            // spectrogram_column...

            // drop step_size samples from the internal ringbuffer of the stft
            // making a step of size step_size
            stft.move_to_next_column();
        }
    }
}
