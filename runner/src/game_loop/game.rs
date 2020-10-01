use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use super::OffscreenBuffer;
use super::audio::Audio;
use super::input::Input;

pub fn game_update_and_render(
    offscreen_buffer: &mut OffscreenBuffer,
    audio: &mut Audio,
    input: &mut Input,
    i: u8,
) {
    // canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
    // render_weird_gradient(offscreen_buffer, i as usize, i as usize).unwrap();
    audio.gen_audio();
}

// fn render_weird_gradient(
//     offscreen_buffer: &mut OffscreenBuffer,
//     blue_offset: usize,
//     green_offset: usize,
// ) -> Result<(), String> {
//     offscreen_buffer.texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
//         dbg!(pitch);
//         let width = pitch / 4;
//         let height = buffer.len() / pitch;
//         for y in 0..height {
//             for x in 0..width {
//                 // B G R A
//                 let offset = y * pitch + 4 * x;
//                 buffer[offset + 0] = (x + blue_offset) as u8;
//                 buffer[offset + 1] = (y + green_offset) as u8;
//                 buffer[offset + 2] = 0;
//                 buffer[offset + 3] = 0;
//             }
//         }
//     })
// }
