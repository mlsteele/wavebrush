pub fn rescale(v: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    return ((v - in_min) / (in_max - in_min) * (out_max - out_min)) + out_min;
}

pub fn within(x: f64, y: f64, threshold: f64) -> bool {
    (x - y).abs() <= threshold
}

pub fn fft_freq(i: usize, sample_rate: usize, fft_size: usize) -> f64 {
    let base = i as f64 * sample_rate as f64 / fft_size as f64;
    if i <= fft_size / 2 {
        base
    } else {
        sample_rate as f64 - base
    }
}
