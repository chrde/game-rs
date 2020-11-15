pub trait HostApi {
    fn print(&self, val: &str) {
        print!("{}", val)
    }

    fn println(&self, val: &str) {
        println!("{}", val)
    }

    fn update_canvas(&mut self, buffer: &[u8], pitch: usize);

    fn generate_audio(&mut self);

    fn load_bmp(&self, path: &str) -> Bitmap;
}

#[derive(Clone, Default)]
pub struct InputState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub sword: bool,
}

pub struct Input {
    pub old: InputState,
    pub new: InputState,
    pub time_per_frame: f32,
}

pub struct Bitmap {
    pub align_x: u32,
    pub align_y: u32,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}
