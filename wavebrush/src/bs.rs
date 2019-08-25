use std::f64;

// https://www.reddit.com/r/rust/comments/3fg0xr/how_do_i_find_the_max_value_in_a_vecf64/
pub trait F64IterExt {
    fn f64_min(&mut self) -> f64;
    fn f64_max(&mut self) -> f64;
}

impl<T> F64IterExt for T where T: Iterator<Item=f64> {
    fn f64_max(&mut self) -> f64 {
        self.fold(f64::NAN, f64::max)
    }
    
    fn f64_min(&mut self) -> f64 {
        self.fold(f64::NAN, f64::min)
    }
}

pub trait F64IterExt2<T> {
    fn f64_max_by_key<F>(self, f: F) -> Option<T>
        where F: FnMut(&T) -> f64;
}

impl<I,T> F64IterExt2<T> for I where I: Iterator<Item=T> {
    fn f64_max_by_key<F>(self, mut f: F) -> Option<T>
        where F: FnMut(&T) -> f64 {
        self.fold::<Option<(f64,T)>,_>(None, |acc, x| if let Some((best, sidecar)) = acc {
            let other = f(&x);
            // https://doc.rust-lang.org/src/core/num/f64.rs.html#387-397
            if best.is_nan() || best < other {
                Some((other, x))
            } else {
                Some((best, sidecar))
            }
        } else {
            Some((f(&x), x))
        }).map(|(_,x)| x)
    }
}
