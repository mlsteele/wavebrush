use num::complex::Complex;
use std::collections::vec_deque::VecDeque;
use std::f64;
use crate::error::*;
use crate::util::*;
use crate::colorramp::*;

type V = Complex<f64>;
type Column = Vec<V>;
pub type Img = image::ImageBuffer<image::Rgb<u8>, Vec<u8>>;

#[derive(Debug, Clone)]
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

    /// Push a column of FFT values.
    pub fn push_back(&mut self, column: Column) -> EResult {
        ensure!(column.len() == self.settings.window_size as usize,
                "unexpected column size {} != {}", column.len(), self.settings.window_size);
        self.data.push_back(column);
        EOK
    }

    /// Drop a column.
    pub fn drop_front(&mut self) {
        self.data.pop_front();
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
