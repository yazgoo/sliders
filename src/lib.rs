use std::env;
use std::error::Error;
use std::process::Command;
use crossterm::{cursor::MoveTo,event::{read, Event, KeyCode, KeyModifiers},terminal::{size, Clear, ClearType, enable_raw_mode, disable_raw_mode}, ExecutableCommand};
use std::io::stdout;

trait SetterGetter {
    fn get(&self) -> Result<u8, Box<dyn Error>>;
    fn set(&self, value: u8) -> Result<(), Box<dyn Error>>;
}

struct Slider {
    name: String,
    setter_getter: Box<dyn SetterGetter>,
    current: u8,
}

impl Slider {

    fn get(&mut self) -> Result<u8, Box<dyn Error>> {
        self.setter_getter.get()
    }

    fn set(&mut self, value: u8) -> Result<(), Box<dyn Error>> {
        self.current = value;
        self.setter_getter.set(value)
    }

    fn inc(&mut self, n: u8) -> Result<(), Box<dyn Error>> {
        let val = self.get()?;
        if (val + n) <= 100 { 
            self.set(val + n)?;
            self.current = val + n;
        }
        Ok(())
    }

    fn dec(&mut self, n: u8) -> Result<(), Box<dyn Error>> {
        let val = self.get()?;
        if val >= n { 
            self.set(val - n)?; 
            self.current = val - n;
        }
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        self.current = self.get()?;
        Ok(())
    }
}

struct CommandLineSetterGetter {
    get_command: String,
    set_command: String,
}

impl SetterGetter for CommandLineSetterGetter {

    fn get(&self) -> Result<u8, Box<dyn Error>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(self.get_command.clone())
            .output()?;
        let mut contents = String::from_utf8_lossy(&output.stdout).to_string();
        if contents.ends_with('\n') { contents.pop(); }
        let res = contents.parse()?;
        Ok(res)
    }

    fn set(&self, value: u8) -> Result<(), Box<dyn Error>> {
        Command::new("sh")
            .arg("-c")
            .arg(self.set_command.replace("{}", format!("{}", value).as_str()))
            .output()?;
        Ok(())
    }

}

fn command_line_slider(name: String, get_command: String, set_command: String) -> Slider {

    Slider {
        name,
        setter_getter: Box::new(CommandLineSetterGetter { get_command, set_command }),
        current: 25,
    }
}

pub struct Sliders {
    sliders: Vec<Slider>
}

impl Sliders {

    fn clear() -> Result<(), Box<dyn Error>> {
        stdout().execute(Clear(ClearType::All))?;
        Ok(())
    }

    fn draw(sliders: &Vec<Slider>, current: &mut usize) -> Result<(), Box<dyn Error>> {
        let (cols, rows) = size()?; 
        Sliders::clear()?;
        let vertical_margin = 4;
        let spaces_count = (cols as usize / sliders.len() - 5) / 2;
        let spaces = format!("{:width$}", "", width=spaces_count);
        for y in 0..(rows - 1) {
            stdout() .execute(MoveTo(0, y))?;
            for (i, slider) in sliders.iter().enumerate() {
                let value = slider.current as u16;
                let start_y = (rows - vertical_margin) * (100 - value) / 100;
                if y > vertical_margin && y < (rows - vertical_margin) {
                    print!("{}", spaces);
                    if y > start_y {
                        print!("│ █ │");
                    }
                    else {
                        print!("│   │");
                    }
                    print!("{}", spaces);
                }
                else if y == (rows - vertical_margin) {
                    print!("{}", spaces);
                    print!("╰───╯");
                    print!("{}", spaces);
                }
                else if y == vertical_margin {
                    print!("{}", spaces);
                    print!("╭───╮");
                    print!("{}", spaces);
                }
                else if y == (rows - vertical_margin + 1) {
                    let title = &slider.name;
                    let spaces_count = (cols as usize / sliders.len() - title.len() - 2) / 2;
                    let spaces = format!("{:width$}", "", width=spaces_count);
                    print!("{}", spaces);
                    print!("{}", if i == *current { "<" } else { " " });
                    print!("{}", title);
                    print!("{}", if i == *current { ">" } else { " " });
                    print!("{}", spaces);
                }
            }
        }
        Ok(())
    }

    fn read_key() -> Result<(KeyCode, KeyModifiers), Box<dyn Error>> {
        loop {
            if let Event::Key(e) = read()?
            {
                return Ok((e.code, e.modifiers));
            }
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut current = 0;
        enable_raw_mode()?;
        loop {
            Sliders::draw(&self.sliders, &mut current)?;
            match Sliders::read_key()? {
                (KeyCode::Char('h'), _) | (KeyCode::Left, _) => if current > 0 { current -= 1 },
                (KeyCode::Char('l'), _) | (KeyCode::Right, _) => if current < (self.sliders.len() - 1) { current += 1 },
                (KeyCode::Char('k'), _) | (KeyCode::Up, _) => self.sliders[current].inc(1)?,
                (KeyCode::Char('j'), _) | (KeyCode::Down , _)=> self.sliders[current].dec(1)?,
                (KeyCode::Char('g'), _) => self.sliders[current].set(0)?,
                (KeyCode::Char('G'), _) => self.sliders[current].set(100)?,
                (KeyCode::Char('m'), _) => self.sliders[current].set(50)?,
                (KeyCode::Char('q'), _) => break,
                (KeyCode::Char('u'), x) if x.contains(KeyModifiers::CONTROL) => self.sliders[current].inc(10)?,
                (KeyCode::Char('d'), x) if x.contains(KeyModifiers::CONTROL) => self.sliders[current].dec(10)?,
                _ => {},
            }
        }
        disable_raw_mode()?;
        Sliders::clear()?;
        Ok(())
    }

    pub fn from_args() -> Result<Sliders, Box<dyn Error>> {
        let mut names = vec![];
        let mut get_commands = vec![];
        let mut set_commands = vec![];

        let mut i = 1;
        let args : Vec<String> = env::args().collect();
        while i < args.len()  {
            match args[i].as_str() {
                "--name" => names.push(args[i + 1].clone()),
                "--get" => get_commands.push(args[i + 1].clone()),
                "--set" => set_commands.push(args[i + 1].clone()),
                _ => {},
            }
            i += 2;
        }

        let mut sliders = vec![];

        for i in 0..names.len() {
            let get_command = get_commands[i].clone();
            let set_command = set_commands[i].clone();
            sliders.push(command_line_slider(
                    names[i].clone(),
                    get_command,
                    set_command));
        }

        for slider in &mut sliders {
            slider.initialize()?;
        }

        Ok(Sliders { sliders })
    }

}
