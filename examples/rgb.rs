use sliders::{Sliders, Slider, SetterGetter};
use std::error::Error;
use blockish::render;
use crossterm::{cursor::{MoveTo, Show, Hide},terminal::{Clear, ClearType, enable_raw_mode, disable_raw_mode}, ExecutableCommand};
use std::io::stdout;

struct PercentSetterGetter {
    value: u8,
}

impl SetterGetter for PercentSetterGetter {
    fn get(&mut self) -> Result<u8, Box<dyn Error>> {
        Ok(self.value)
    }

    fn set(&mut self, value: u8) -> Result<(), Box<dyn Error>> {
        self.value = value;
        Ok(())
    }

}

fn main() -> Result<(), Box<dyn Error>> {
    let img = image::open("examples/capybara.jpg").unwrap();
    let rgb_image = img.as_rgb8().unwrap();
    let default =  0;
    let mut sliders = Sliders {
        sliders: vec![
        Slider {
            name: String::from("red"),
            setter_getter: Box::new(PercentSetterGetter { value: default }),
            current: default,
        },
        Slider {
            name: String::from("green"),
            setter_getter: Box::new(PercentSetterGetter { value: default }),
            current: default,
        },
        Slider {
            name: String::from("blue"),
            setter_getter: Box::new(PercentSetterGetter { value: default }),
            current: default,
        }
        ],
        coordinates_percent: (50, 0),
        size_percent: (50, 100),
        current: 0
    };
    stdout().execute(Clear(ClearType::All))?;
    stdout().execute(Hide)?;
    loop {
        stdout().execute(Clear(ClearType::All))?;
        stdout().execute(MoveTo(0, 0))?;
        render(rgb_image.width(), rgb_image.height(), &|x, y| {
            let rgb = rgb_image.get_pixel(x, y);
            let r = sliders.sliders[0].current.checked_add(rgb[0]).unwrap_or(254);
            let g = sliders.sliders[1].current.checked_add(rgb[1]).unwrap_or(254);
            let b = sliders.sliders[2].current.checked_add(rgb[2]).unwrap_or(254);
            (r, g, b)
        });
        sliders.draw()?;
        enable_raw_mode()?;
        if !sliders.prompt()? {
            break;
        }
        disable_raw_mode()?;
    }
    disable_raw_mode()?;
    stdout().execute(Show)?;
    Ok(())
}

