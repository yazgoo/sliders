use sliders::{Sliders, Slider, SetterGetter};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use rodio::{Decoder, OutputStream, Sink};
use rodio::source::{SineWave, Source};
use std::error::Error;

struct SoundSetterGetter {
    sink: Sink,
    value: u8,
}

impl SoundSetterGetter {

    fn new() -> Result<SoundSetterGetter, Box<dyn Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(SoundSetterGetter {
            sink,
            value: 50,
        })
    }
}


impl SetterGetter for SoundSetterGetter {
    fn get(&mut self) -> Result<u8, Box<dyn Error>> {
        Ok(self.value)
    }

    fn set(&mut self, value: u8) -> Result<(), Box<dyn Error>> {
        self.value = value;
        let source = SineWave::new(440).take_duration(Duration::from_secs_f32(0.25)).amplify(0.50);
        self.sink.append(source);
        Ok(())
    }

}

fn main() -> Result<(), Box<dyn Error>> {
    Sliders {
        sliders: vec![
        Slider {
            name: String::from("music"),
            setter_getter: Box::new(SoundSetterGetter::new()?),
            current: 50,
        }],
        coordinates_percent: (0, 0),
        size_percent: (100, 100),
        current: 0
    }.run()?;
    Ok(())
}
