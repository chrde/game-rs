use super::host_api::*;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use std::sync::mpsc::Receiver;
use std::time::Instant;

mod audio;
mod bmp;
mod input;

use super::reloader::*;
use audio::Audio;

fn new_texture(
    creator: &TextureCreator<WindowContext>, width: u32, height: u32,
) -> Result<Texture<'_>, String> {
    creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, width, height)
        .map_err(|e| e.to_string())
}

struct SdlHostApi<'a> {
    texture: Texture<'a>,
    audio: Audio,
}

impl<'a> HostApi for SdlHostApi<'a> {
    fn update_canvas(&mut self, buffer: &[u8], pitch: usize) {
        self.texture.update(None, buffer, pitch).unwrap();
    }

    fn generate_audio(&mut self) {
        self.audio.gen_audio();
    }

    fn load_bmp(&self, path: &str) -> Bitmap {
        bmp::load_from_file(path)
    }
}

pub fn main(reloader: Receiver<()>) -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio = Audio::new(sdl_context.audio()?)?;

    let window = video_subsystem
        .window("rust-sdl2 demo", 1920 / 2, 1080 / 2)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let (width, height) = canvas.window().size();
    let texture = new_texture(&texture_creator, width, height)?;

    let mut host_api = SdlHostApi { texture, audio };

    let mut game = GameLib::new().unwrap();
    let mut api = game.api().unwrap();
    let state = (api.init)(&host_api);

    host_api.audio.toggle();
    let mut input = Input {
        new: Default::default(),
        old: Default::default(),
        time_per_frame: 1.0 / 60.0,
    };
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut start_frame = Instant::now();
    'running: loop {
        if reloader.try_recv().is_ok() {
            println!("===== Reloading =====");
            std::mem::drop(api);
            game = game.reload().unwrap();
            api = game.api().unwrap();
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    println!("===== Restarting =====");
                    (api.restart)(state);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    host_api.audio.toggle();
                }
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(x, y) => {
                        println!("resized {} {}", x, y);
                        let (x, y) = canvas.window().size();
                        println!("logical {} {}", x, y);
                    }
                    _ => {}
                },
                _ => {}
            }
            input::update(&mut input, &event);
        }
        // The rest of the game loop goes here...

        (api.update)(state, &input, &mut host_api);
        canvas.clear();
        let (width, height) = canvas.window().size();
        canvas.copy(
            &host_api.texture,
            None,
            Rect::new(0, 0, width / 2, height / 2),
        )?;
        canvas.present();
        input::swap(&mut input);

        if false {
            dbg!(start_frame.elapsed());
        }
        start_frame = Instant::now();
    }
    Ok(())
}
