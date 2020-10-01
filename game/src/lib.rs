use engine::*;

#[no_mangle]
// pub extern "C" fn game_init<'a>(a: u8) -> *mut GameState<'a> {
pub extern "C" fn game_init(a: u8) -> *mut GameState {
    std::ptr::null_mut()
    // unimplemented!()
    // foo();
    // dbg!(a + 1);
    // println!("hello");
    // let game = GameState {
    //     phantom: PhantomData,
    // };
    // Box::into_raw(Box::new(game))
}

#[no_mangle]
pub extern "C" fn game_update(game_state: &mut GameState) -> bool {
// pub extern "C" fn game_update<'a>(game_state: &mut GameState) -> bool {
    let buffer = &mut game_state.offscreen_buffer.buffer;
    let pitch = 3200;
    let width = pitch / 4;
    let height = buffer.len() / pitch;
    for y in 0..height {
        for x in 0..width {
            // B G R A
            let offset = y * pitch + 4 * x;
            buffer[offset + 0] = (x + game_state.blue_offset) as u8;
            buffer[offset + 1] = (y + game_state.green_offset) as u8;
            buffer[offset + 2] = 0;
            buffer[offset + 3] = 0;
        }
    }
    true
}
