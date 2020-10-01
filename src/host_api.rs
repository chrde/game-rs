pub trait HostApi {
    fn print(&self, val: &str) {
        println!("{}", val)
    }

    fn update_canvas(&mut self, buffer: &[u8], pitch: usize);
    fn generate_audio(&mut self);
}
