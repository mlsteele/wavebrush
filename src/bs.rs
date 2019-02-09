// https://www.reddit.com/r/rust/comments/3fg0xr/how_do_i_find_the_max_value_in_a_vecf64/

use std::f64;

pub trait FloatIterExt {
    fn float_min(&mut self) -> f64;
    fn float_max(&mut self) -> f64;
}

impl<T> FloatIterExt for T where T: Iterator<Item=f64> {
    fn float_max(&mut self) -> f64 {
        self.fold(f64::NAN, f64::max)
    }
    
    fn float_min(&mut self) -> f64 {
        self.fold(f64::NAN, f64::min)
    }
}

pub trait FloatIterExt2<T> {
    fn max_by_key<F>(self, f: F) -> Option<T>
        where F: FnMut(&T) -> f64;
}

impl<I,T> FloatIterExt2<T> for I where I: Iterator<Item=T> {
    fn max_by_key<F>(self, f: F) -> Option<T>
        where F: FnMut(&T) -> f64 {
        self.fold(None, |acc, x| if let Some((best, sidecar)) = acc {
            xxx("todo finish this")
            f64::max(best, f(x))
        } else {
            (f(x), x)
        })
    }
}
