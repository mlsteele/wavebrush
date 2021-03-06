use crate::spectrogram::*;
use crossbeam::channel::*;
use num::Complex;

#[derive(Debug)]
pub enum ToBackend {
    // Coordinate in image space.
    Info{x: i32, y: i32},
    Prod{x: i32, y: i32},
    Erase{x: i32, y: i32},
    Sliders(Sliders),
    Play,
    Save,
    Reset,
    Nuke,
    Quit,
}

#[derive(Debug)]
pub enum ToUI {
    Spectrogram(Img),
    Info{
        freq: f64,
        a: Complex<f64>,
        b: Complex<f64>,
    },
}

pub type CtlUI = FullDuplex<ToUI, ToBackend>;
pub type CtlBackend = FullDuplex<ToBackend, ToUI>;

#[derive(Debug, Default)]
pub struct Sliders {
    pub weight: f64,
    pub size: f64,
    pub fade_exp: f64,
    pub copies: i32,
    pub distance_linear: f64,
    pub distance_exp: f64,
}

pub struct FullDuplex<In,Out> {
    pub r: Receiver<In>,
    pub s: Sender<Out>,
}

impl<In,Out> FullDuplex<In,Out> {
    fn new_pair() -> (Self, FullDuplex<Out,In>) {
        let bound = 50;
        let (s1, r1) = bounded::<In>(bound);
        let (s2, r2) = bounded::<Out>(bound);
        (FullDuplex{
            r: r1,
            s: s2,
        }, FullDuplex{
            r: r2,
            s: s1,
        }) 
    }

    pub fn send(&self, msg: Out) {
        if let Err(err) = self.s.try_send(msg) {
            println!("error sending message: {}", err);
        }
    }
}

impl<In,Out> Clone for FullDuplex<In,Out> {
    fn clone(&self) -> Self {
        Self {
            r: self.r.clone(),
            s: self.s.clone(),
        }
    }
}

pub fn new_ctl() -> (CtlUI, CtlBackend) {
    FullDuplex::new_pair()
}

// let _ = ctl.s.try_send(ToBackend::Prod{x: mouse_image_pos.0 as i32, y: mouse_image_pos.1 as i32});
