#![allow(unused_imports)]

extern crate hound;
extern crate rodio;
extern crate image;
#[macro_use] extern crate failure;
use failure::*;

mod sample;
#[allow(dead_code)]
mod util;

#[allow(dead_code)]
mod stft;
use stft::{STFT, WindowType};

#[allow(dead_code)]
mod control;
use control::*;

mod bs;
use bs::*;

mod ui;
use ui::*;

#[allow(dead_code)]
mod colorramp;

#[allow(dead_code)]
mod spectrogram;
use spectrogram::*;

#[allow(dead_code)]
mod shredder;
use shredder::*;

#[allow(dead_code)]
mod edit;
use edit::*;

mod error;
use error::*;

use std::f64;
use rustfft::{FFTplanner};
use num::complex::Complex;
use image::{DynamicImage};
use std::f64::consts::PI;
use sample::{SampleConvert,*};
use util::*;
use std::thread;
use std::env;

fn main() {
    if let Err(err) = main2() {
        // eprintln!("{}", pretty_error(&err));
        eprint_error(&err);
        std::process::exit(1);
    }
}

fn eprint_error(err: &failure::Error) {
    eprintln!("");
    for err in err.iter_chain() {
        eprintln!("{}", err);
    }
    eprintln!("\n{:?}", err.backtrace());
}

// https://github.com/BurntSushi/imdb-rename/blob/f2a40bf/src/main.rs#L345-L355
/// Return a prettily formatted error, including its entire causal chain.
#[allow(dead_code)]
fn pretty_error(err: &failure::Error) -> String {
    let mut pretty = err.to_string();
    let mut prev = err.as_fail();
    while let Some(next) = prev.cause() {
        pretty.push_str(": ");
        pretty.push_str(&next.to_string());
        prev = next;
    }
    pretty
}

#[allow(unused_variables)]
fn main2() -> EResult {
    use glium::glutin;
    use glium::{Display, Surface};

    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let builder = glutin::WindowBuilder::new()
        .with_title("Wavebrush")
        .with_dimensions(glutin::dpi::LogicalSize::new(200., 200.));
    let display = Display::new(builder, context, &events_loop).unwrap();
    EOK
}
