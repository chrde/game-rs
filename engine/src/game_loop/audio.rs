use sdl2::audio::AudioQueue;
use sdl2::audio::AudioSpecDesired;
use sdl2::audio::AudioStatus;
use sdl2::AudioSubsystem;

struct Sound {
    channels: u8,
    running_sample: usize,
    samples_per_second: u32,
    tone_hz: u32,
    volume: i16,
    data: Vec<i16>,
}

impl Sound {
    fn gen_wave(&mut self, bytes_to_write: u32) {
        let period = self.samples_per_second / self.tone_hz;
        self.data.clear();

        for _ in 0..bytes_to_write {
            let val = if (self.running_sample / period as usize) % 2 == 0 {
                self.volume
            } else {
                -self.volume
            };
            for _ in 0..self.channels {
                self.data.push(val);
            }
            self.running_sample += 1;
        }
    }
}

pub struct Audio {
    device: AudioQueue<i16>,
    sound: Sound,
}

impl Audio {
    fn samples_in_queue(&self) -> u32 {
        //TODO make Audio generic over <i16>
        self.device.size() / std::mem::size_of::<i16>() as u32 / self.sound.channels as u32
    }

    pub fn new(audio_subsystem: AudioSubsystem) -> Result<Self, String> {
        let samples_per_second = 48_000;
        let channels = 2;
        let sound = Sound {
            channels,
            running_sample: 0,
            samples_per_second,
            tone_hz: 256,
            volume: 3_000,
            data: Vec::new(),
        };

        let desired_spec = AudioSpecDesired {
            freq: Some(sound.samples_per_second as i32),
            channels: Some(sound.channels),
            samples: None,
        };

        let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec)?;

        Ok(Self { device, sound })
    }

    pub fn gen_audio(&mut self) {
        let bytes_to_write = self.sound.samples_per_second - self.samples_in_queue();
        if bytes_to_write > 0 {
            self.sound.gen_wave(bytes_to_write);
            assert!(self.device.queue(&self.sound.data));
        }
    }

    pub fn toggle(&mut self) {
        match self.device.status() {
            AudioStatus::Playing => self.device.pause(),
            _ => self.device.resume(),
        }
    }
}
