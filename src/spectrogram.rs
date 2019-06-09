use num::complex::Complex;
use std::collections::vec_deque::VecDeque;
use std::f64;
use crate::error::*;
use crate::util::*;
use crate::colorramp::*;

type V = Complex<f64>;
type Column = Vec<V>;
pub type Img = image::ImageBuffer<image::Rgb<u8>, Vec<u8>>;

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// Samples per second
    pub sample_rate: u32,
    /// FFT size in samples
    pub window_size: u32,
    /// Samples between each window start.
    pub step_size: u32,
}

/// More than just a spectrogram.
/// Saves full FFT output including phase information.
#[derive(Debug, Clone)]
pub struct Spectrogram {
    pub settings: Settings,
    data: VecDeque<Column>
}

impl Spectrogram {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: settings,
            data: Default::default(),
        }
    }

    pub fn explode(self) -> (Settings, VecDeque<Column>) {
        (self.settings, self.data)
    }

    pub fn width(&self) -> i32 { self.data.len() as i32 }

    /// Push a column of FFT values.
    pub fn push_back(&mut self, column: Column) -> EResult {
        ensure!(column.len() == self.settings.window_size as usize,
                "unexpected column size {} != {}", column.len(), self.settings.window_size);
        self.data.push_back(column);
        EOK
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut V> {
        if x < 0 || y < 0 {
            return None
        }
        if let Some(column) = self.data.get_mut(x as usize) {
            column.get_mut(y as usize)
        } else {
            None
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&V> {
        if x < 0 || y < 0 {
            return None
        }
        if let Some(column) = self.data.get(x as usize) {
            column.get(y as usize)
        } else {
            None
        }
    }

    pub fn freq(&self, y: i32) -> f64 {
        fft_freq(y as usize, self.settings.sample_rate as usize,
                 self.settings.window_size as usize)
    }

    // Get the value at two points. The one at (x, y1) and its dual
    // on the other half of the column.
    // These are the buds who'd would be complex conjugates
    // right after FFT of the real-valued audio signal.
    /// Returns (freq, point value, dual value)
    pub fn get_dual(&mut self, x: i32, y1: i32) -> Option<(f64, &Complex<f64>, &Complex<f64>)> {
        let ws = self.settings.window_size as i32;
        if y1 > ws/2 {
            return None;
        }
        if let Some(v1) = self.get(x, y1) {
            let y2 = ws - y1;
            if let Some(v2) = self.get(x, y2) {
                Some((self.freq(y1), v1, v2))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn data_mut(&mut self) -> &mut VecDeque<Column> {
        &mut self.data
    }

    pub fn image(&self) -> Result<Img> {
        ensure!(self.data.len() > 0, "cannot create image from empty spectrogram");
        let ws = self.settings.window_size;
        let mut img = image::ImageBuffer::new(self.data.len() as u32, ws / 2);
        for (x,column) in self.data.iter().enumerate() {
            // Show only half the column.
            let morphed: Vec<_> = column[..ws as usize/2].iter().map(|v| v.norm_sqr().log10())
                .map(|v| if v > 0. {v} else {0.}).collect();
            // let min = morphed.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            // let max = morphed.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            // let morphed: Vec<_> = column[..ws as usize/2].iter().map(|v| v.norm_sqr().log10())
            //     .map(|v| if v > 0. {v} else {0.}).collect();
            for (i, &v) in morphed.iter().enumerate() {
                // let sv = rescale(v, min, max, 0., 1.);
                let sv = v;
                let pixel = img.get_pixel_mut(x as u32, img.height()-1-i as u32);
                *pixel = image::Rgb(ramp(sv));
            }
        }
        Ok(img)
    }
}
