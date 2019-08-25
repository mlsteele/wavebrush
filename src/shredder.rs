use lib::*;
use crate::spectrogram::*;
use lib::stft::{STFT, WindowType};
use crate::error::*;
use crate::util::*;
use rustfft::{FFTplanner,FFT};
use num::complex::Complex;
use std::sync::Arc;
use std::collections::vec_deque::VecDeque;

type Column = Vec<Complex<f64>>;

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
pub struct Unshredder {
    settings: Settings,
    ifft: Arc<dyn FFT<f64>>,
    src: VecDeque<Column>,
    buf_overlap: Vec<Complex<f64>>,
    scratch: Vec<Complex<f64>>,
}

impl Unshredder {
    pub fn new(sg: Spectrogram) -> Self {
        let (settings, src) = sg.explode();
        let ws = settings.window_size as usize;
        let overlap_size = ws - settings.step_size as usize;
        Self {
            settings: settings,
            ifft: FFTplanner::<f64>::new(true).plan_fft(ws as usize),
            src: src,
            buf_overlap: vec![Default::default(); overlap_size],
            scratch: vec![Default::default(); ws],
        }
    }

    pub fn output_size(&self) -> usize {
        self.settings.step_size as usize
    }

    pub fn allocate_output_buf(&self) -> Vec<f64> {
        vec![Default::default(); self.output_size()]
    }

    fn ws(&self) -> usize { return self.settings.window_size as usize }

    /// Output to a buffer of size `self.output_size()`.
    /// Returns whether any output was written.
    /// If false, no more output will ever come.
    pub fn output(&mut self, buf_out: &mut [f64]) -> Result<bool> {
        ensure!(buf_out.len() == self.output_size(), "output buf size");
        if let Some(mut column) = self.src.pop_front() {
            Self::filter(&mut column);
            self.ifft.process(&mut column, &mut self.scratch);
            // overlap += scratch[..w-s];
            for i in 0..self.ws()-self.settings.step_size as usize {
                self.buf_overlap[i] += self.scratch[i];
            }
            // shipit(overlap);
            for (sample, out) in self.buf_overlap.iter().zip(buf_out.iter_mut()) {
                // The overlap-add of the window at hop-size is equal numerically
                // to the dc gain of the window divided by the step size.
                *out = sample.re / self.ws() as f64;
            };
            // overlap = scratch[w-s..];
            self.buf_overlap.copy_from_slice(&self.scratch[self.settings.step_size as usize..]);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// For corrupting the signal.
    fn filter(column: &mut [Complex<f64>]) {
        let _ = column;
        // Lose phase information
        // for sample in column.iter_mut() {
        //     let (mut r, mut theta) = sample.to_polar();
        //     theta = 0.;
        //     *sample = Complex::from_polar(&r, &theta);
        // }
    }
}
