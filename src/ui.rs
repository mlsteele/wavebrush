use glium::{
    backend::{Context, Facade},
    texture::{ClientFormat, RawImage2d},
    Texture2d,
};
use imgui::{FontGlyphRange, ImFontConfig, ImGui, Ui, ImString};
use imgui_winit_support;
use std::rc::Rc;
use std::time::Instant;
use std::borrow::Cow;
use image;
use crate::control::*;
use std::collections::*;

const CLEAR_COLOR: [f32; 4] = [0.01, 0.01, 0.01, 1.];

type SpectroImage = image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>;

pub fn run(ctl: CtlUI, spectrogram: SpectroImage) {
    use glium::glutin;
    use glium::{Display, Surface};
    use imgui_glium_renderer::Renderer;
    let img_scale = 1.5f32;

    let mut events_loop = glutin::EventsLoop::new();
    let hidpi_factor = events_loop.get_primary_monitor().get_hidpi_factor();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let builder = glutin::WindowBuilder::new()
        .with_title("Wavebrush")
        .with_dimensions(glutin::dpi::LogicalSize::new(
            spectrogram.width() as f64 * img_scale as f64 + 100.,
            spectrogram.height() as f64 * img_scale as f64 + 500.));
}

fn texture_from_image<F>(img: &SpectroImage, gl_ctx: &F) -> Texture2d
where F: Facade {
    let raw = RawImage2d {
        data: Cow::Owned(img.clone().into_raw()),
        width: img.width(),
        height: img.height(),
        format: ClientFormat::U8U8U8,
    };
    Texture2d::new(gl_ctx, raw).unwrap()
}

struct SliderBankEntry {
    label: ImString,
    value: f32,
    min: f32,
    max: f32,
}

struct SliderBank {
    event: bool,
    map: HashMap<String,SliderBankEntry>,
}

impl SliderBank {
    pub fn new() -> Self {
        Self {
            event: false,
            map: Default::default()
        }
    }
    pub fn add(&mut self, key: &str, label: &str, default: f64, min: f64, max: f64) {
        self.map.insert(key.to_owned(), SliderBankEntry{
            label: ImString::new(label),
            value: default as f32,
            min: min as f32,
            max: max as f32,
        });
        self.event = true;
    }

    pub fn get(&self, key: &str) -> Option<f64> {
        self.map.get(key).map(|SliderBankEntry{value,..}| *value as f64)
    }

    /// Return whether new values are ready.
    /// And mark them as read.
    pub fn event(&mut self) -> bool {
        std::mem::replace(&mut self.event, false)
    }

    pub fn draw(&mut self, ui: &imgui::Ui) {
        use imgui::{im_str};
        for (_, slider) in self.map.iter_mut() {
            if ui.slider_float(&slider.label, &mut slider.value, slider.min, slider.max).build() {
                self.event = true;
            }
        }
    }
}
