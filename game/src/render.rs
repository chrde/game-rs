use crate::host_api::*;
use crate::V2;

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

#[derive(Copy, Clone, Debug)]
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

    pub fn render_rectangle(&mut self, min: V2, max: V2, color: Color) {
        let miny = {
            let miny = min.y().round() as i32;
            if miny < 0 {
                0
            } else {
                miny as usize
            }
        };

        let minx = {
            let minx = min.x().round() as i32;
            if minx < 0 {
                0
            } else {
                minx as usize
            }
        };

        let maxx = {
            let maxx = std::cmp::max(max.x().round() as i32, 0);
            if maxx > self.width as i32 {
                self.width as usize
            } else {
                maxx as usize
            }
        };

        let maxy = {
            let maxy = std::cmp::max(max.y().round() as i32, 0);
            if maxy > self.height as i32 {
                self.height as usize
            } else {
                maxy as usize
            }
        };

        for y in miny..maxy {
            for x in minx..maxx {
                let offset = y * self.pitch() + self.bytes_per_pixel * x;
                self.buffer[offset + 0] = (color.blue * 255.0).round() as u8;
                self.buffer[offset + 1] = (color.green * 255.0).round() as u8;
                self.buffer[offset + 2] = (color.red * 255.0).round() as u8;
                self.buffer[offset + 3] = 0;
            }
        }
    }

    pub fn render_bitmap(
        &mut self,
        bitmap: &Bitmap,
        xy: V2,
        align_x: i32,
        align_y: i32,
        c_alpha: f32,
    ) {
        let real_x = xy.x() - (align_x as f32);
        let real_y = xy.y() - (align_y as f32);

        let (min_x, source_offset_x) = {
            let min_x = real_x.round() as i32;
            if min_x < 0 {
                (0, -min_x as usize)
            } else {
                (min_x as usize, 0)
            }
        };

        let (min_y, source_offset_y) = {
            let min_y = real_y.round() as i32;
            if min_y < 0 {
                (0, -min_y as usize)
            } else {
                (min_y as usize, 0)
            }
        };

        let max_x = {
            // let max_x = (real_x + bitmap.width as f32).round() as i32;
            let max_x = real_x.round() as i32 + bitmap.width as i32;
            if max_x > self.width as i32 {
                self.width as usize
            } else {
                max_x as usize
            }
        };

        let max_y = {
            // let max_y = (real_y + bitmap.height as f32).round() as i32;
            let max_y = real_y.round() as i32 + bitmap.height as i32;
            if max_y > self.height as i32 {
                self.height as usize
            } else {
                max_y as usize
            }
        };

        let mut source_offset_pixel =
            bitmap.width * (bitmap.height - 1) - (source_offset_y * bitmap.width) + source_offset_x;

        let mut dest_offset_pixel = min_y * self.width + min_x;
        for _ in min_y..max_y {
            let mut source_offset = source_offset_pixel * self.bytes_per_pixel;
            let mut dest_offset = dest_offset_pixel * self.bytes_per_pixel;
            for _ in min_x..max_x {
                let sb = bitmap.pixels[source_offset + 0] as f32;
                let sg = bitmap.pixels[source_offset + 1] as f32;
                let sr = bitmap.pixels[source_offset + 2] as f32;
                let a = c_alpha * (bitmap.pixels[source_offset + 3] as f32 / 255.0);

                let db = self.buffer[dest_offset + 0] as f32;
                let dg = self.buffer[dest_offset + 1] as f32;
                let dr = self.buffer[dest_offset + 2] as f32;

                let r = (1.0 - a) * dr + a * sr;
                let g = (1.0 - a) * dg + a * sg;
                let b = (1.0 - a) * db + a * sb;

                self.buffer[dest_offset + 0] = (b + 0.5) as u8;
                self.buffer[dest_offset + 1] = (g + 0.5) as u8;
                self.buffer[dest_offset + 2] = (r + 0.5) as u8;
                // self.buffer[dest_offset + 3] = 0;

                source_offset += self.bytes_per_pixel;
                dest_offset += self.bytes_per_pixel;
            }
            source_offset_pixel -= bitmap.width;
            dest_offset_pixel += self.width;
        }
    }
}
