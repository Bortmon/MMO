use winit::{
    event::{Event, WindowEvent, KeyEvent, ElementState},
    event_loop::{EventLoop, ControlFlow}, 
    window::WindowBuilder,
    keyboard::{PhysicalKey, KeyCode},
};

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new()
        .with_title("MMO")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)
        .unwrap();

    println!("Venster is gemaakt. Druk op ESC of sluit het venster om te stoppen.");

   
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    println!("Sluitverzoek ontvangen, programma stopt.");
                    elwt.exit();
                },
                WindowEvent::KeyboardInput { event: KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    state: ElementState::Pressed,
                    ..
                }, .. } => {
                    println!("Escape ingedrukt, programma stopt.");
                    elwt.exit();
                },
                _ => (),
            },
            _ => (),
        }
    }).unwrap();
}