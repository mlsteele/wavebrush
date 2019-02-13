use crate::spectrogram::*;
use crossbeam::channel::*;

pub enum ToBackend {
    // Coordinate in image space.
    Prod{x: i32, y: i32},
}

pub enum ToUI {
    Spectrogram(Img),
}

pub struct CtlUI {
    pub s: Sender<ToBackend>,
    pub r: Receiver<ToUI>,
}

pub struct CtlBackend {
    pub s: Sender<ToUI>,
    pub r: Receiver<ToBackend>,
}

pub fn new_ctl() -> (CtlUI, CtlBackend) {
    let (s1, r1) = bounded::<ToUI>(10);
    let (s2, r2) = bounded::<ToBackend>(10);
    return (CtlUI{
        s: s2,
        r: r1,
    }, CtlBackend{
        s: s1,
        r: r2,
    }) 
}
