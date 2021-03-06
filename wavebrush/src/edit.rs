use crate::spectrogram::*;
use num::complex::Complex;
use crate::control::Sliders;
use crate::util::*;
use std::f64::consts::*;

type V = Complex<f64>;

pub struct Wrapper<'a> {
    sp: &'a mut Spectrogram,
    sliders: &'a Sliders,
}

impl<'a> Wrapper<'a> {
    pub fn new(sp: &'a mut Spectrogram, sliders: &'a Sliders) -> Self {
        Self {
            sp: sp,
            sliders: sliders,
        }
    }

    // Effect two pixels. The one at (x, y1) and its dual
    // on the other half of the column.
    // These are the buds who'd would be complex conjugates
    // right after FFT of the real-valued audio signal.
    pub fn effect_dual<F>(&mut self, x: i32, y1: i32, mut f: F) -> bool
        where F: FnMut(&mut V, bool)
    {
        let ws = self.sp.settings.window_size as i32;
        if y1 > ws/2 {
            return false;
        }
        if let Some(v1) = self.sp.get_mut(x, y1) {
            f(v1, false);
            let y2 = ws - y1;
            self.sp.get_mut(x, y2).map(|v2| f(v2, true));
            true
        } else {
            false
        }
    }

    pub fn airbrush(&mut self, x: i32, y: i32) {
        self.multibrush(x, y, 2.);
    }

    pub fn erase(&mut self, x: i32, y: i32) {
        self.multibrush(x, y, -1.);
    }

    fn multibrush(&mut self, x: i32, y: i32, factor: f64) {
        let weight = self.sliders.weight * factor;
        let harmonics = self.sliders.copies;
        for i in 0..harmonics {
            self.airbrush2(
                // x, (y as f64 * 1.9f64.powf(i as f64)) as i32,
                x, (y as f64 + (self.sliders.distance_linear * i as f64
                                * self.sliders.distance_exp.powf(i as f64)) as f64) as i32,
                weight / self.sliders.fade_exp.powf(i as f64));
        }
    }

    fn airbrush2(&mut self, x: i32, y: i32, weight: f64) {
        let size = self.sliders.size as i32;
        for dx in -size..size {
            for dy in -size..size {
                let brush_r2 = (dx as f64).powf(2.) + (dy as f64).powf(2.);
                let x = x + dx;
                let y = y + dy;
                self.effect_dual(x, y, |v, _| {
                    let r = weight / (brush_r2+1.);
                    let theta = PI * x as f64;
                    *v += Complex::from_polar(&r, &theta);
                });
            }
        }
    }

    pub fn nuke(&mut self) {
        let st = self.sp.settings.clone();
        for (frame, column) in self.sp.data_mut().iter_mut().enumerate() {
            for (i, v) in column.iter_mut().enumerate() {
                // Simulate phase recovery.
                let freq = fft_freq(i,
                    st.sample_rate as usize, st.window_size as usize);
                // let r = if i == st.window_size as usize - 5 {
                //     100.
                // } else {
                //     0.
                // };
                let r = 0.;
                let theta = -2. * PI * frame as f64 / st.sample_rate as f64 * freq;
                *v = Complex::from_polar(&r, &theta);
            }
        }
    }

    // pub fn scratch(&mut self) {
    //     let st = self.sp.settings.clone();
    //     for (frame, column) in self.sp.data_mut().iter_mut().enumerate() {
    //         for (i, v) in column.iter_mut().enumerate() {
    //             let freq = fft_freq(i,
    //                 st.sample_rate as usize, st.window_size as usize);
    //             let (mut r, theta) = v.to_polar();
    //             // dbg!(freq);
    //             // dbg!(r);
    //             // dbg!(theta);
    //         }
    //     }
    // }

    // pub fn scratch2(&mut self) {
    //     let st = self.sp.settings.clone();
    //     for (frame, column) in self.sp.data_mut().iter_mut().enumerate() {
    //         for (i, v) in column.iter_mut().enumerate() {
    //             // Simulate phase recovery.
    //             let freq = fft_freq(i,
    //                 st.sample_rate as usize, st.window_size as usize);
    //             let r = if i == st.window_size as usize - 5 {
    //                 3500.
    //             } else {
    //                 0.
    //             };
    //             // let columns_per_second: f64 = st.sample_rate as f64 * 2.;
    //             // let theta = 2. * PI * frame as f64 / columns_per_second * freq;
    //             let theta = frame as f64 / 10.;
    //             *v = Complex::from_polar(&r, &theta);
    //         }
    //     }
    // }

}
