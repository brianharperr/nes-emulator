use nes_cpu::Nes;
use sdl2::{event::Event, keyboard::Keycode};

pub struct SDLWrapper{
    nes: Nes,
}

impl SDLWrapper {
    pub fn new(nes: Nes) -> Self {
        SDLWrapper{
            nes
        }
    }

    pub fn run(&mut self){
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let window = video_subsystem
            .window("nes-emulator", 256, 240)
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

        loop {
            self.nes.step();
            if self.nes.poll_frame() {
                renderer.clear();
                texture.update(None, &self.nes.frame(), 256 * 3).unwrap();

                let _ = renderer.copy(&texture, None, None);
                renderer.present();
                
            }
            self.handle_input(&mut event_pump);
        }
    }

    fn handle_input(&mut self, event_pump: &mut sdl2::EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | 
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    std::process::exit(0);
                }
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                    println!("Dumping nametables...");
                    if let Err(e) = self.nes.dump_ppu() {
                        println!("Failed to dump nametables: {}", e);
                    } else {
                        println!("Nametables dumped successfully!");
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
                    println!("Reset");
                    self.nes.reset();
                }
                _ => {}
            }
        }
    }

    
}