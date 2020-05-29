mod display;
mod interpreter;
mod keyboard;
mod memory;
mod timer;

use interpreter::Interpreter;
use pixels::{wgpu::Surface, Pixels, SurfaceTexture};
use std::fs::File;
use std::io::Read;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::{
    dpi::LogicalSize,
    event::{self, Event},
    window::WindowBuilder,
};

const WINDOW_WIDTH: u32 = 512;
const WINDOW_HEIGHT: u32 = 256;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    args.next();
    let game_path_relative = args.next().ok_or("Must enter path to a game")?;
    let game_path = std::env::current_dir()?.join(game_path_relative);
    println!("Loading game from {:?}...", game_path);
    let mut game_data = Vec::new();
    let mut game_file = File::open(game_path)?;
    game_file.read_to_end(&mut game_data)?;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("CHIP 8")
        .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)?;

    let mut pixels = {
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(WINDOW_WIDTH, WINDOW_HEIGHT, surface);
        Pixels::new(64, 32, surface_texture)?
    };

    let mut processor = Interpreter::new(&game_data);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                processor.run_cycle();

                let display_buffer = processor.get_display_buffer();
                render(display_buffer, &mut pixels).unwrap();
            }
            Event::WindowEvent {
                event:
                    event::WindowEvent::KeyboardInput {
                        input:
                            event::KeyboardInput {
                                virtual_keycode: Some(key_code),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if key_code == event::VirtualKeyCode::Escape {
                    *control_flow = ControlFlow::Exit;
                } else {
                    processor.handle_input(key_code, state == event::ElementState::Pressed);
                }
            }
            Event::WindowEvent {
                event: event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn render(display_buffer: &[bool], pixels: &mut Pixels) -> Result<(), pixels::Error> {
    let pixel_buffer = pixels.get_frame();
    for (dp, pixel) in display_buffer.iter().zip(pixel_buffer.chunks_exact_mut(4)) {
        let rgba = match dp {
            true => [255, 255, 255, 255],
            _ => [0, 0, 0, 255],
        };

        pixel.copy_from_slice(&rgba);
    }

    pixels.render()
}
