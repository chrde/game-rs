use engine::*;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use std::marker::PhantomData;
use std::sync::mpsc::Receiver;
use std::time::Instant;

mod audio;
// mod game;
mod input;

use audio::Audio;
// use game::game_update_and_render;
use input::Input;

fn new_texture(
    creator: &TextureCreator<WindowContext>,
    width: u32,
    height: u32,
) -> Result<Texture<'_>, String> {
    creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, width, height)
        .map_err(|e| e.to_string())
}

pub fn main(reloader: Receiver<()>) -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut audio = Audio::new(sdl_context.audio()?)?;

    let window = video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let (width, height) = canvas.window().size();
    let mut texture = new_texture(&texture_creator, width, height)?;
    let mut offscreen_buffer = OffscreenBuffer {
        buffer: vec![0; 800 * 600 * 4],
    };
    let mut state = GameState {
        offscreen_buffer,
        blue_offset: 0,
        green_offset: 0,
        // phantom: PhantomData,
    };

    let mut game = super::reloader::Game::reload().unwrap();

    // canvas.set_draw_color(Color::RGB(0, 255, 255));
    // canvas.clear();
    // canvas.copy(&offscreen_buffer.texture, None, None)?;
    // canvas.present();
    audio.toggle();
    let mut input = Input::default();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    let mut start_frame = Instant::now();
    'running: loop {
        if reloader.try_recv().is_ok() {
            game = {
                std::mem::drop(game);
                super::reloader::Game::reload().unwrap()
            };
            println!("===== Reloading =====");
        }
        i = (i + 1) % 255;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    audio.toggle();
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
            input.update(&event);
        }
        // The rest of the game loop goes here...

        unsafe {
            assert!((game.api.update)(&mut state));
        }
        // game_update_and_render(&mut offscreen_buffer, &mut audio, &mut input, i);
        canvas.clear();
        texture
            .update(None, &state.offscreen_buffer.buffer, 3200)
            .unwrap();
        canvas.copy(&texture, None, None)?;
        canvas.present();
        input.swap();

        dbg!(start_frame.elapsed());
        start_frame = Instant::now();
    }
    Ok(())
}
