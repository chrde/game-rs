pub trait HostApi {
    fn print(&self, val: &str) {
        print!("{}", val)
    }

    fn println(&self, val: &str) {
        println!("{}", val)
    }

    fn update_canvas(&mut self, buffer: &[u8], pitch: usize);
    fn generate_audio(&mut self);
}

#[derive(Clone, Default)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

pub struct Input {
    pub old: InputState,
    pub new: InputState,
    pub time_per_frame: f32,
}
