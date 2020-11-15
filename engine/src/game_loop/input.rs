use crate::host_api::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn update(input: &mut Input, event: &Event) {
    match event {
        Event::KeyUp { keycode, .. } => {
            if let Some(keycode) = keycode {
                match keycode {
                    Keycode::Up => input.new.up = false,
                    Keycode::Down => input.new.down = false,
                    Keycode::Left => input.new.left = false,
                    Keycode::Right => input.new.right = false,
                    Keycode::S => input.new.sword = false,
                    _ => {}
                }
            }
        }
        Event::KeyDown { keycode, .. } => {
            if let Some(keycode) = keycode {
                match keycode {
                    Keycode::Up => input.new.up = true,
                    Keycode::Down => input.new.down = true,
                    Keycode::Left => input.new.left = true,
                    Keycode::Right => input.new.right = true,
                    Keycode::S => input.new.sword = true,
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

pub fn swap(input: &mut Input) {
    std::mem::swap(&mut input.old, &mut input.new);
    input.new = input.old.clone();
}
