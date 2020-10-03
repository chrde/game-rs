#[repr(C)]
pub struct OffscreenBuffer {
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

impl OffscreenBuffer {
    pub fn render_rectangle(&mut self, minx: f32, miny: f32, maxx: f32, maxy: f32, b: bool) {
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

        let maxx = {
            let maxx = maxx.round() as i32;
            if maxx > self.height as i32 {
                self.height as usize
            } else {
                maxx as usize
            }
        };

        let maxy = {
            let maxy = maxy.round() as i32;
            if maxy > self.height as i32 {
                self.height as usize
            } else {
                maxy as usize
            }
        };

        for y in miny..maxy {
            for x in minx..maxx {
                let offset = y * self.pitch() + self.bytes_per_pixel * x;
                if b {
                self.buffer[offset + 0] = 255;
                } else {
                self.buffer[offset + 0] = 0;

                }
                self.buffer[offset + 1] = 0;
                self.buffer[offset + 2] = 255;
                self.buffer[offset + 3] = 0;
            }
        }
    }
}
