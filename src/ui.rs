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
    let display = Display::new(builder, context, &events_loop).unwrap();
    let window = display.gl_window();

    let mut imgui = ImGui::init();
    imgui.set_ini_filename(None);

    // let hidpi_factor = window.get_hidpi_factor().round();
    // let hidpi_factor = 2.;

    let font_size = (18.0 * hidpi_factor) as f32;

    // For info on how to configure fonts:
    // https://github.com/ocornut/imgui/blob/master/imgui.h#L1909
    // imgui.fonts().add_font_with_config(
    //     include_bytes!("../resources/OpenSans/OpenSans-Regular.ttf"),
    //     ImFontConfig::new()
    //         .oversample_h(1)
    //         .pixel_snap_h(true)
    //         .size_pixels(font_size)
    //         .glyph_extra_spacing([0.9, 0.]),
    //     &FontGlyphRange::default(),
    // );

    // imgui.fonts().add_default_font_with_config(
    //     ImFontConfig::new()
    //         .oversample_h(1)
    //         .pixel_snap_h(true)
    //         .size_pixels(font_size),
    // );

    // imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);

    let mut renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    imgui_winit_support::configure_keys(&mut imgui);

    // let texture_id = renderer.textures().insert(texture_from_image(&spectrogram, &display));

    let mut last_frame = Instant::now();
    let mut quit = false;

    // // Mouse position in imgui screen coordinates
    // let mut mouse_pos;
    // let mut mouse_down: [bool; 5] = Default::default();
    // // Mouse position in image coordinates.
    // let mut mouse_image_pos: Option<(f32, f32)> = None;

    // let mut sliders = SliderBank::new();
    // sliders.add("weight", "Weight", 2.1, 0.1, 20.);
    // sliders.add("size", "Brush size", 8., 1., 40.);
    // sliders.add("fade_exp", "Fade factor", 1.9, 1., 10.);
    // sliders.add("copies", "Copies", 20., 1., 100.);
    // sliders.add("distance_linear", "Distance (lin)", 30., 1., 200.);
    // sliders.add("distance_exp", "Distance (exp)", 1.1, 1., 10.);

    // let mut point_info = None;

    loop {
        // for msg in ctl.r.try_iter() {
        //     use ToUI::*;
        //     match msg {
        //         Spectrogram(img) => {
        //             let _ = renderer.textures().replace(texture_id, texture_from_image(&img, &display));
        //         },
        //         x@Info{..} => point_info = Some(x),
        //     }
        // }

        // events_loop.poll_events(|event| {
        //     use glium::glutin::{Event, WindowEvent, WindowEvent::CloseRequested, VirtualKeyCode, KeyboardInput};
        //     use glium::glutin::Event::*;
        //     use glium::glutin::ElementState::*;
        //     // println!("{:?}", event);

        //     imgui_winit_support::handle_event(
        //         &mut imgui,
        //         &event,
        //         window.get_hidpi_factor(),
        //         hidpi_factor,
        //     );

        //     // if let Event::WindowEvent { event, .. } = event {
        //     //     match event {
        //     //         CloseRequested => quit = true,
        //     //         WindowEvent::KeyboardInput{
        //     //             input: KeyboardInput{
        //     //                 state: Pressed, virtual_keycode:
        //     //                 Some(key), ..}, ..} if key == VirtualKeyCode::Escape => quit = true,
        //     //         WindowEvent::CursorMoved{..} => {
        //     //             if let Some((x, y)) = mouse_image_pos {
        //     //                 let x = (x / img_scale) as i32;
        //     //                 let y = spectrogram.height() as i32 - (y / img_scale) as i32;
        //     //                 if mouse_down[0] {
        //     //                     ctl.send(ToBackend::Prod{x, y});
        //     //                 } else if mouse_down[1] {
        //     //                     ctl.send(ToBackend::Erase{x, y});
        //     //                 }
        //     //                 ctl.send(ToBackend::Info{x, y});
        //     //             }
        //     //         },
        //     //         _ => (),
        //     //     }
        //     // }
        // });

        // if sliders.event() {
        //     ctl.send(ToBackend::Sliders(Sliders{
        //         weight: sliders.get("weight").unwrap(),
        //         size: sliders.get("size").unwrap(),
        //         fade_exp: sliders.get("fade_exp").unwrap(),
        //         copies: sliders.get("copies").unwrap() as i32,
        //         distance_linear: sliders.get("distance_linear").unwrap(),
        //         distance_exp: sliders.get("distance_exp").unwrap(),
        //     }));
        // }

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;

        imgui_winit_support::update_mouse_cursor(&imgui, &window);

        let frame_size = imgui_winit_support::get_frame_size(&window, hidpi_factor).unwrap();

        let ui = imgui.frame(frame_size, delta_s);
        // mouse_pos = ui.imgui().mouse_pos();
        // mouse_down = ui.imgui().mouse_down();

        let cond = ImGuiCond::FirstUseEver;
        use imgui::{im_str,ImGuiCond};
        // ui.window(im_str!("Wavebrush"))
        //     .position((5.,5.), cond)
        //     .size((spectrogram.width() as f32 * img_scale + 50.,
        //            spectrogram.height() as f32 * img_scale + 500.), cond)
        //     .build(|| {
        //         if ui.small_button(im_str!("Play")) {
        //             ctl.send(ToBackend::Play);
        //         }
        //         if ui.small_button(im_str!("Reset")) {
        //             ctl.send(ToBackend::Reset);
        //         }
        //         if ui.small_button(im_str!("Nuke")) {
        //             ctl.send(ToBackend::Nuke);
        //         }
        //         if ui.small_button(im_str!("Save")) {
        //             ctl.send(ToBackend::Save);
        //         }
        //         sliders.draw(&ui);
        //         ui.separator();
        //         let cursor_pos = ui.get_cursor_pos();
        //         let cursor_screen_pos = ui.get_cursor_screen_pos();
        //         ui.child_frame(im_str!("subwindow"),
        //                        (spectrogram.width() as f32 * img_scale,
        //                         spectrogram.height() as f32 * img_scale))
        //             .movable(false)
        //             .build(|| {
        //                 mouse_image_pos = Some((mouse_pos.0 - cursor_screen_pos.0,
        //                                         mouse_pos.1 - cursor_screen_pos.1));
        //                 ui.image(texture_id, (spectrogram.width() as f32 * img_scale,
        //                                     spectrogram.height() as f32 * img_scale)).build();
        //             });
        //         if let Some(ToUI::Info{freq, a, b}) = point_info {
        //             ui.text(im_str!("Frequency: {:.1}", freq));
        //             macro_rules! label_complex { ($label:expr, $complex:expr) => (
        //                 let (r, theta) = $complex.to_polar();
        //                 ui.text(im_str!("{}: rad {:2.1}  angle {:3.1}°)", $label, r, theta.to_degrees()));
        //             )}
        //             label_complex!("A", a);
        //             label_complex!("B", b);
        //             ui.separator();
        //         }
        //         ui.separator();
        //         ui.text(im_str!(
        //             "Mouse Position: ({:.1},{:.1})",
        //             mouse_pos.0,
        //             mouse_pos.1
        //         ));
        //         ui.text(im_str!(
        //             "Cursor Position (Window): ({:.1},{:.1})",
        //             cursor_pos.0,
        //             cursor_pos.1
        //         ));
        //         ui.text(im_str!(
        //             "Cursor Position (Screen): ({:.1},{:.1})",
        //             cursor_screen_pos.0,
        //             cursor_screen_pos.1
        //         ));
        //         ui.text(im_str!(
        //             "Mouse Position (Image): ({:.1},{:.1})",
        //             mouse_image_pos.unwrap().0,
        //             mouse_image_pos.unwrap().1
        //         ));
        //     });

        // let mut target = display.draw();
        // target.clear_color(
        //     CLEAR_COLOR[0],
        //     CLEAR_COLOR[1],
        //     CLEAR_COLOR[2],
        //     CLEAR_COLOR[3],
        // );
        // renderer.render(&mut target, ui).expect("Rendering failed");
        // target.finish().unwrap();

        if quit {
            break;
        }
    }
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
