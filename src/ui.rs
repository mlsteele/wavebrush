use glium::{
    backend::{Context, Facade},
    texture::{ClientFormat, RawImage2d},
    Texture2d,
};
use imgui::{FontGlyphRange, ImFontConfig, ImGui, Ui};
use imgui_winit_support;
use std::rc::Rc;
use std::time::Instant;
use std::borrow::Cow;
use image;

const CLEAR_COLOR: [f32; 4] = [0.01, 0.01, 0.01, 1.];

type SpectroImage = image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>;

pub fn run(spectrogram: SpectroImage) {
    use glium::glutin;
    use glium::{Display, Surface};
    use imgui_glium_renderer::Renderer;

    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let builder = glutin::WindowBuilder::new()
        .with_title("Wavebrush")
        .with_dimensions(glutin::dpi::LogicalSize::new(
            spectrogram.width() as f64 + 100., spectrogram.height() as f64 + 100.));
    let display = Display::new(builder, context, &events_loop).unwrap();
    let window = display.gl_window();

    let mut imgui = ImGui::init();
    imgui.set_ini_filename(None);

    // let hidpi_factor = window.get_hidpi_factor().round();
    let hidpi_factor = 1.;

    let font_size = (13.0 * hidpi_factor) as f32;

    imgui.fonts().add_default_font_with_config(
        ImFontConfig::new()
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(font_size),
    );

    imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);

    let mut renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    imgui_winit_support::configure_keys(&mut imgui);

    let texture_id = renderer.textures().insert(texture_from_image(&spectrogram, &display));

    let mut last_frame = Instant::now();
    let mut quit = false;

    loop {
        events_loop.poll_events(|event| {
            use glium::glutin::{Event, WindowEvent, WindowEvent::CloseRequested, VirtualKeyCode, KeyboardInput};
            use glium::glutin::Event::*;
            use glium::glutin::ElementState::*;

            imgui_winit_support::handle_event(
                &mut imgui,
                &event,
                window.get_hidpi_factor(),
                hidpi_factor,
            );

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    CloseRequested => quit = true,
                    WindowEvent::KeyboardInput{
                        input: KeyboardInput{
                            state: Pressed, virtual_keycode:
                            Some(key), ..}, ..} if key == VirtualKeyCode::Escape => quit = true,
                    _ => (),
                }
            }
        });

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;

        imgui_winit_support::update_mouse_cursor(&imgui, &window);

        let frame_size = imgui_winit_support::get_frame_size(&window, hidpi_factor).unwrap();

        let ui = imgui.frame(frame_size, delta_s);

        let cond = ImGuiCond::FirstUseEver;
        use imgui::{im_str,ImGuiCond};
        ui.window(im_str!("Wavebrush"))
            .position((5.,5.), cond)
            .size((spectrogram.width() as f32 + 100., spectrogram.height() as f32 + 100.), cond)
            .build(|| {
                ui.text(im_str!("Spectrogram"));
                ui.separator();
                let mouse_pos = ui.imgui().mouse_pos();
                ui.text(im_str!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos.0,
                    mouse_pos.1
                ));
                ui.separator();
                ui.image(texture_id, (spectrogram.width() as f32, spectrogram.height() as f32)).build();
            });

        let mut target = display.draw();
        target.clear_color(
            CLEAR_COLOR[0],
            CLEAR_COLOR[1],
            CLEAR_COLOR[2],
            CLEAR_COLOR[3],
        );
        renderer.render(&mut target, ui).expect("Rendering failed");
        target.finish().unwrap();

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

fn generate_texture<F>(gl_ctx: &F) -> Texture2d
where F: Facade {
    // Generate dummy texture
    let (WIDTH, HEIGHT) = (100, 100);
    let mut data = Vec::with_capacity(WIDTH * HEIGHT);
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            // Insert RGB values
            data.push(i as u8);
            data.push(j as u8);
            data.push((i + j) as u8);
        }
    }

    let raw = RawImage2d {
        data: Cow::Borrowed(&data),
        width: WIDTH as u32,
        height: HEIGHT as u32,
        format: ClientFormat::U8U8U8,
    };
    Texture2d::new(gl_ctx, raw).unwrap()
}