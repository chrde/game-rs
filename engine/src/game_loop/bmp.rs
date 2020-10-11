use crate::host_api::*;
use std::convert::TryInto;
use std::fs;

pub fn load_from_file(path: &str) -> Bitmap {
    let pixels = fs::read(path)
        .or_else(|e| {
            let msg = format!("While opening {}: {}", path, e);
            Err(msg)
        })
        .unwrap();

    let header = header(&pixels);
    let alpha_mask = !(header.red_mask | header.green_mask | header.blue_mask);
    let red_shift = header.red_mask.trailing_zeros();
    let green_shift = header.green_mask.trailing_zeros();
    let blue_shift = header.blue_mask.trailing_zeros();
    let alpha_shift = alpha_mask.trailing_zeros();

    let mut result = Vec::with_capacity(header.size_of_bitmap as usize);

    let bytes_per_pixel = header.bits_per_pixel as usize / 8;
    let pitch = header.width as usize * bytes_per_pixel;
    // B G R A
    for y in 0..header.height as usize {
        for x in 0..header.width as usize {
            let offset = header.bitmap_offset as usize + y * pitch + x * bytes_per_pixel;
            let color = u32::from_le_bytes(pixels[offset..offset + 4].try_into().unwrap());
            let good_color = (((color >> alpha_shift) & 0xFF) << 24)
                | (((color >> red_shift) & 0xFF) << 16)
                | (((color >> green_shift) & 0xFF) << 8)
                | (((color >> blue_shift) & 0xFF) << 0);

            for b in &good_color.to_le_bytes() {
                result.push(*b);
            }
        }
    }
    assert_eq!(result.len(), header.size_of_bitmap as usize);

    Bitmap {
        width: header.width as usize,
        height: header.height as usize,
        pixels: result,
    }
}

fn header(buf: &[u8]) -> BmpHeader {
    unsafe { std::ptr::read(buf.as_ptr() as *const _) }
    // let p = buf.as_ptr() as *const _;
    // unsafe { &*p }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct BmpHeader {
    file_type: u16,
    file_size: u32,
    reserved1: u16,
    reserved2: u16,
    bitmap_offset: u32,
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits_per_pixel: u16,
    compression: u32,
    size_of_bitmap: u32,
    horz_resolution: i32,
    vert_resolution: i32,
    colors_used: u32,
    colors_important: u32,
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
}
