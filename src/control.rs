use crate::spectrogram::*;
use crossbeam::channel::*;

#[derive(Debug)]
pub enum ToBackend {
    // Coordinate in image space.
    Prod{x: i32, y: i32},
    Erase{x: i32, y: i32},
    Play,
    Save,
    Reset,
    Quit,
}

#[derive(Debug)]
pub enum ToUI {
    Spectrogram(Img),
}

pub type CtlUI = FullDuplex<ToUI, ToBackend>;
pub type CtlBackend = FullDuplex<ToBackend, ToUI>;

pub struct FullDuplex<In,Out> {
    pub r: Receiver<In>,
    pub s: Sender<Out>,
}

impl<In,Out> FullDuplex<In,Out> {
    fn new_pair() -> (Self, FullDuplex<Out,In>) {
        let (s1, r1) = bounded::<In>(10);
        let (s2, r2) = bounded::<Out>(10);
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

pub fn new_ctl() -> (CtlUI, CtlBackend) {
    FullDuplex::new_pair()
}

// let _ = ctl.s.try_send(ToBackend::Prod{x: mouse_image_pos.0 as i32, y: mouse_image_pos.1 as i32});
