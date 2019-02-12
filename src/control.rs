use crate::spectrogram::*;

pub enum FromUI {
    Prod(x: i32, y: i32),
}

pub enum ToUI {
    Spectrogram(Img),
}
