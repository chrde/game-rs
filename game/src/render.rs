#[repr(C)]
pub struct OffscreenBuffer {
    // B G R A
    pub buffer: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub bytes_per_pixel: usize,
}

impl OffscreenBuffer {
    pub fn pitch(&self) -> usize {
        self.width * self.bytes_per_pixel
    }
}

pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl OffscreenBuffer {
    pub fn reset(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let offset = y * self.pitch() + self.bytes_per_pixel * x;
                // hideous magenta
                self.buffer[offset + 0] = 255;
                self.buffer[offset + 1] = 0;
                self.buffer[offset + 2] = 255;
                self.buffer[offset + 3] = 0;
            }
        }
    }

    pub fn render_rectangle(&mut self, minx: f32, miny: f32, maxx: f32, maxy: f32, color: Color) {
        let miny = {
            let miny = miny.round() as i32;
            if miny < 0 {
                0
            } else {
                miny as usize
            }
        };

        let minx = {
            let minx = minx.round() as i32;
            if minx < 0 {
                0
            } else {
                minx as usize
            }
        };

        let max = {
            let maxx = std::cmp::max(maxx.round() as i32, 0);
            if maxx > self.width as i32 {
                self.width as usize
            } else {
                maxx as usize
            }
        };

        let maxy = {
            let maxy = std::cmp::max(maxy.round() as i32, 0);
            if maxy > self.height as i32 {
                self.height as usize
            } else {
                maxy as usize
            }
        };

        for y in miny..maxy {
            for x in minx..max {
                let offset = y * self.pitch() + self.bytes_per_pixel * x;
                self.buffer[offset + 0] = (color.blue * 255.0).round() as u8;
                self.buffer[offset + 1] = (color.green * 255.0).round() as u8;
                self.buffer[offset + 2] = (color.red * 255.0).round() as u8;
                self.buffer[offset + 3] = 0;
            }
        }
    }
}
