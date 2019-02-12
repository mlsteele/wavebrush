use crate::spectrogram::*;
use num::complex::Complex;

type V = Complex<f64>;

pub struct Wrapper<'a> {
    sp: &'a mut Spectrogram
}

impl<'a> Wrapper<'a> {
    pub fn new(sp: &'a mut Spectrogram) -> Self {
        Self {
            sp: sp,
        }
    }

    pub fn airbrush(&mut self, x: i32, y: i32) {
        for dx in -10..10 {
            for dy in -10..10 {
                let brush_r2 = (dx as f64).powf(2.) + (dy as f64).powf(2.);
                let x = x + dx;
                let y = y + dy;
                if let Some(v) = self.sp.get_mut(x, y) {
                    let (mut r, theta) = v.to_polar();
                    r += 40. * 1. / (brush_r2+1.);
                    *v = Complex::from_polar(&r, &theta);
                }
            }
        }
    }
}
