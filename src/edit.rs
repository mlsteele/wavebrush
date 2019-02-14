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
        let brush = 14;
        for dx in -brush..brush {
            for dy in -brush..brush {
                let brush_r2 = (dx as f64).powf(2.) + (dy as f64).powf(2.);
                let x = x + dx;
                let y = y + dy;
                self.effect_dual(x, y, |v, _| {
                    let (mut r, theta) = v.to_polar();
                    r += 5. * 1. / (brush_r2+1.);
                    *v = Complex::from_polar(&r, &theta);
                });
            }
        }
    }
}
