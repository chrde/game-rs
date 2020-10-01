use sdl2::event::Event;
use sdl2::keyboard::Keycode;

#[derive(Clone, Default)]
pub struct State {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

#[derive(Default)]
pub struct Input {
    old: State,
    new: State,
}

impl Input {
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.old, &mut self.new);
        self.new = self.old.clone();
    }

    pub fn update(&mut self, event: &Event) {
        match event {
            Event::KeyDown { keycode, .. } => {
                if let Some(keycode) = keycode {
                    match keycode {
                        Keycode::Up => self.new.up = true,
                        Keycode::Down => self.new.down = true,
                        Keycode::Left => self.new.left = true,
                        Keycode::Right => self.new.right = true,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
