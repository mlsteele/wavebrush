use glium::glutin;

fn main() {
    let events_loop = glutin::EventsLoop::new();
    let context = glutin::ContextBuilder::new();
    let builder = glutin::WindowBuilder::new();
    match glium::Display::new(builder, context, &events_loop) {
        Err(err) => println!("error building display: {:?}", err),
        Ok(_display) => println!("displ-a-ok"),
    }
}
