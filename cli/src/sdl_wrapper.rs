use std::time::{Duration, Instant};

use nes_cpu::{controller::Button, Nes};
use sdl2::{event::Event, keyboard::{Keycode, Scancode}, rect::Rect};

pub struct SDLWrapper{
    nes: Nes,
    previous_keyboard_state: [bool; 8]
}

impl SDLWrapper {
    pub fn new(nes: Nes) -> Self {
        SDLWrapper{
            nes,
            previous_keyboard_state: [false; 8]
        }
    }

    pub fn run(&mut self){
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let scale = 3;
        let window = video_subsystem
            .window("nes-emulator", 256 * scale, 240 * scale)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut renderer = window.into_canvas().accelerated().present_vsync().build().unwrap();
        let texture_creator = renderer.texture_creator();

        let mut texture = texture_creator.create_texture(
            sdl2::pixels::PixelFormatEnum::RGB24,
            sdl2::render::TextureAccess::Streaming,
            256,
            240,
        ).unwrap();

        let mut event_pump = sdl.event_pump().unwrap();

        const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60); // 60 FPS
        let mut last_frame_time = Instant::now();
        let mut frame_start: Instant;
        
        'running: loop {
            frame_start = Instant::now();

            // Handle input once per frame
            if !self.handle_input(&mut event_pump) {
                break 'running;
            }

            // Run the NES until we have a new frame
            loop {
                self.nes.step();
                if self.nes.poll_frame() {
                    break;
                }
            }

            // Render the frame
            renderer.clear();
            texture.update(None, &self.nes.frame(), 256 * 3).unwrap();
            
            // Get current window size for proper scaling
            let (window_width, window_height) = renderer.output_size().unwrap();
            let scale_x = window_width as f32 / 256.0;
            let scale_y = window_height as f32 / 240.0;
            let scale = scale_x.min(scale_y);

            let scaled_width = (256.0 * scale) as u32;
            let scaled_height = (240.0 * scale) as u32;
            let x_offset = (window_width - scaled_width) / 2;
            let y_offset = (window_height - scaled_height) / 2;

            let dst = Rect::new(
                x_offset as i32,
                y_offset as i32,
                scaled_width,
                scaled_height,
            );

            renderer.copy(&texture, None, Some(dst)).unwrap();
            renderer.present();

            // Frame timing
            let frame_duration = frame_start.elapsed();
            if frame_duration < FRAME_TIME {
                std::thread::sleep(FRAME_TIME - frame_duration);
            }

            last_frame_time = frame_start;
        }
    }

    fn handle_input(&mut self, event_pump: &mut sdl2::EventPump) -> bool {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return false,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Escape => return false,
                    Keycode::Num1 => {
                        println!("Dumping nametables...");
                        if let Err(e) = self.nes.dump_ppu() {
                            println!("Failed to dump nametables: {}", e);
                        } else {
                            println!("Nametables dumped successfully!");
                        }
                    }
                    Keycode::Backspace => {
                        self.nes.reset();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        
        let keyboard_state = event_pump.keyboard_state();
        
        const KEY_MAPPINGS: [(Scancode, Button, usize); 8] = [
            (Scancode::Up, Button::Up, 0),
            (Scancode::Down, Button::Down, 1),
            (Scancode::Left, Button::Left, 2),
            (Scancode::Right, Button::Right, 3),
            (Scancode::X, Button::A, 4),
            (Scancode::Z, Button::B, 5),
            (Scancode::Return, Button::Start, 6),
            (Scancode::LShift, Button::Select, 7),
        ];

        // Check each key and update controller only if state changed
        for &(scancode, button, index) in KEY_MAPPINGS.iter() {
            let is_pressed = keyboard_state.is_scancode_pressed(scancode);
            if is_pressed != self.previous_keyboard_state[index] {
                self.nes.set_button(button, is_pressed);
                self.previous_keyboard_state[index] = is_pressed;
            }
        }

        true
    }

    
}