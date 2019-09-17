use crate::spectrogram::*;
use crate::stft::{STFT, WindowType};
use crate::error::*;
use crate::ensure_eq;
use rustfft::{FFTplanner,FFT};
use num::complex::Complex;
use std::sync::Arc;
use strider::{SliceRing, SliceRingImpl};
use crate::util::*;
use std::f64::consts::PI;

/// Build a spectrogram from audio samples.
pub struct Shredder {
    pub sg: Spectrogram,
    stft: STFT,
}

impl Shredder {
    pub fn new(settings: Settings) -> Self {
        Self {
            stft: STFT::new(WindowType::Hanning,
                            settings.window_size as usize, settings.step_size as usize),
            sg: Spectrogram::new(settings),
        }
    }

    /// Returns whether a new column was processed.
    pub fn append_samples(&mut self, input: &[f64]) -> EResult {
        self.stft.append_samples(input);
        while self.stft.contains_enough_to_compute() {
            self.stft.compute_into_complex_output();
            self.sg.push_back(self.stft.complex_output.clone())?;
            self.stft.move_to_next_column();
        }
        EOK
    }
}

/// Output audio from a spectrogram.
/// Compatibility layer for Unshredder2.
pub struct Unshredder {
    inner: Unshredder2,
    src: Spectrogram,
}

impl Unshredder {
    pub fn new(sg: Spectrogram) -> Self {
        Self {
            inner: Unshredder2::new(sg.settings),
            src: sg,
        }
    }

    pub fn output_size(&self) -> usize {
        self.inner.settings.step_size as usize
    }

    pub fn allocate_output_buf(&self) -> Vec<f64> {
        vec![Default::default(); self.output_size()]
    }

    /// Output to a buffer of size `self.output_size()`.
    /// Returns whether any output was written.
    /// If false, no more output will ever come.
    pub fn output(&mut self, buf_out: &mut [f64]) -> Result<bool> {
        ensure_eq!(buf_out.len(), self.output_size(), "output buf size");
        self.inner.output(&mut self.src, buf_out)
    }
}

/// Output audio from a spectrogram.
pub struct Unshredder2 {
    settings: Settings,
    ifft: Arc<dyn FFT<f64>>,
    buf_overlap: Vec<Complex<f64>>,
    scratch: Vec<Complex<f64>>,
    counter: usize,
    out_frame: Vec<f64>,
    out_collector: strider::SliceRingImpl<f64>,
}

impl Unshredder2 {
    pub fn new(settings: Settings) -> Self {
        let ws = settings.window_size as usize;
        let overlap_size = ws - settings.step_size as usize;
        Self {
            settings: settings,
            ifft: FFTplanner::<f64>::new(true).plan_fft(ws as usize),
            buf_overlap: vec![Default::default(); overlap_size],
            scratch: vec![Default::default(); ws],
            counter: 0,
            out_frame: vec![Default::default(); settings.step_size as usize],
            out_collector: SliceRingImpl::with_capacity(settings.step_size as usize),
        }
    }

    /// Output a frame if there is enough buffered for one.
    fn output_frame(&mut self, src: &mut Spectrogram) -> Result<bool> {
        if let Some(mut column) = src.pop_front() {
            let ws = self.settings.window_size as usize;
            self.filter(&mut column);
            self.counter += 1;
            self.ifft.process(&mut column, &mut self.scratch);
            // overlap += scratch[..w-s];
            for i in 0..ws-self.settings.step_size as usize {
                self.buf_overlap[i] += self.scratch[i];
            }
            // shipit(overlap);
            for (sample, out) in self.buf_overlap.iter().zip(self.out_frame.iter_mut()) {
                // The overlap-add of the window at hop-size is equal numerically
                // to the dc gain of the window divided by the step size.
                *out = sample.re / ws as f64;
            };
            self.out_collector.push_many_back(&self.out_frame);
            // overlap = scratch[w-s..];
            self.buf_overlap.copy_from_slice(&self.scratch[self.settings.step_size as usize..]);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Output audio samples to a buffer (of any size).
    /// Returns whether output was written.
    pub fn output(&mut self, src: &mut Spectrogram, buf_out: &mut [f64]) -> Result<bool> {
        while self.out_collector.len() < buf_out.len() {
            if !(self.output_frame(src)?) {
                // Not enough data to fill buf_out.
                return Ok(false);
            }
        }
        let n_read = self.out_collector.read_many_front(buf_out);
        assert_eq!(n_read, buf_out.len(), "unshredder ringer buffer output");
        self.out_collector.drop_many_front(n_read);
        Ok(true)
    }

    /// For corrupting the signal.
    fn filter(&self, column: &mut [Complex<f64>]) {
        let _ = column;

        // // Lose phase information
        // for sample in column.iter_mut() {
        //     let (mut r, mut theta) = sample.to_polar();
        //     theta = 0.;
        //     *sample = Complex::from_polar(&r, &theta);
        // }

        // let column_len = column.len();
        // for (i, sample) in column.iter_mut().enumerate() {
        //     let (mut r, mut theta) = sample.to_polar();
        //     theta += (column_len as f64 / 2. - i as f64).sin() * PI / 2.;
        //     *sample = Complex::from_polar(&r, &theta);
        // }

        for sample in column.iter_mut() {
            sample.re *= sample.im;
        }

        // println!("counter {}", self.counter);
        // for (y, sample) in column.iter_mut().enumerate() {
        //     let freq = fft_freq(y, self.settings.sample_rate as usize, self.settings.window_size as usize);
        //     let (mut r, mut theta) = sample.to_polar();
        //     let mid_freqs = [200., 400., 700., 1300.];
        //     let widths = [100., 200., 300., 400.];
        //     let speeds = [1., 0.3, 0.8, 0.2];
        //     for (mid_freq, width, speed) in izip!(mid_freqs.iter(), widths.iter(), speeds.iter()) {
        //         let target_freq = mid_freq + (self.counter as f64 * speed).sin() * width;
        //         r /= 1. + ((freq - target_freq).abs() / 1000.);
        //     }
        //     *sample = Complex::from_polar(&r, &theta);
        // }

    }
}
