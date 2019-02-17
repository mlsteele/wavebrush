fn main() {
    use glium::glutin;

    #[allow(unused_mut)]
    let mut events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let builder = glutin::WindowBuilder::new()
        .with_title("Crash dummy")
        .with_dimensions(glutin::dpi::LogicalSize::new(200., 200.));
    let _display = glium::Display::new(builder, context, &events_loop).unwrap();
}
