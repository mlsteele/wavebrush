// Convert between sample representations.

pub trait SampleConvertTrait<X,Y> {
    fn convert(x: X) -> Y;
}

pub struct SampleConvert {}

impl SampleConvertTrait<i32, f64> for SampleConvert {
    fn convert(x: i32) -> f64 {
        match x as f64 / std::i32::MAX as f64 {
            y if y > 1. => 1.,
            y if y < -1. => -1.,
            y => y,
        }
    }
}

impl SampleConvertTrait<i16, f64> for SampleConvert {
    fn convert(x: i16) -> f64 {
        match x as f64 / std::i16::MAX as f64 {
            y if y > 1. => 1.,
            y if y < -1. => -1.,
            y => y,
        }
    }
}

impl SampleConvertTrait<f64, i16> for SampleConvert {
    fn convert(x: f64) -> i16 {
        let max = std::i16::MAX;
        match (x * max as f64) as i16 {
            y if y > max => max,
            y if y < -max => -max,
            y => y,
        }
    }
}

impl SampleConvertTrait<f32, f64> for SampleConvert {
    fn convert(x: f32) -> f64 {
        x as f64
    }
}

impl SampleConvertTrait<f64, f32> for SampleConvert {
    fn convert(x: f64) -> f32 {
        x as f32
    }
}
