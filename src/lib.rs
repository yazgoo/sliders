use std::env;
use std::error::Error;
use std::process::Command;
use crossterm::{cursor::{Show,Hide,MoveTo},event::{read, Event, KeyCode, KeyModifiers},terminal::{size, Clear, ClearType, enable_raw_mode, disable_raw_mode}, ExecutableCommand};
use std::io::stdout;

pub trait SetterGetter {
    fn get(&mut self) -> Result<u8, Box<dyn Error>>;
    fn set(&mut self, value: u8) -> Result<(), Box<dyn Error>>;
}

pub struct Slider {
    pub name: String,
    pub setter_getter: Box<dyn SetterGetter>,
    pub current: u8,
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

    fn get(&mut self) -> Result<u8, Box<dyn Error>> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(self.get_command.clone())
            .output()?;
        let mut contents = String::from_utf8_lossy(&output.stdout).to_string();
        if contents.ends_with('\n') { contents.pop(); }
        let res = contents.parse()?;
        Ok(res)
    }

    fn set(&mut self, value: u8) -> Result<(), Box<dyn Error>> {
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
    pub sliders: Vec<Slider>,
    pub coordinates_percent: (u16, u16),
    pub size_percent: (u16, u16),
    pub current: usize,
}

impl Sliders {

    fn clear() -> Result<(), Box<dyn Error>> {
        stdout().execute(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn draw(&self) -> Result<(), Box<dyn Error>> {
        let (total_cols, total_rows) = size()?; 
        let (cols, rows) = (total_cols * self.size_percent.0 / 100, total_rows * self.size_percent.1 / 100);
        let (x0, y0) = (total_cols * self.coordinates_percent.0 / 100, total_rows * self.coordinates_percent.1 / 100);
        let vertical_margin = 10 * self.size_percent.1 / 100;
        let spaces_count = (cols as usize / self.sliders.len() - 5) / 2;
        let spaces = format!("{:width$}", "", width=spaces_count);
        for y in 0..(rows - 1) {
            stdout().execute(MoveTo(x0, y0 + y))?;
            for (i, slider) in self.sliders.iter().enumerate() {
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
                    let spaces_count = (cols as usize / self.sliders.len() - title.len() - 2) / 2;
                    let spaces = format!("{:width$}", "", width=spaces_count);
                    print!("{}", spaces);
                    print!("{}", if i == self.current { "<" } else { " " });
                    print!("{}", title);
                    print!("{}", if i == self.current { ">" } else { " " });
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

    fn print_help() -> Result<(), Box<dyn Error>> {
        disable_raw_mode()?;
        stdout() .execute(MoveTo(4, 4))?;
        println!(r#"
      ╭─────────────────────────────────────╮
      │ h, left arrow    previous slider    │
      │ l, right arrow   next slider        │
      │ k, up arrow      increment slider   │
      │ j, down arrow    decrement slider   │
      │ g                set slider to 0    │
      │ G                set slider to 100  │
      │ m                set slider to 50   │
      │ ?                prints this help   │
      │ q                exit               │
      │ ctrl+u           increment 10       │
      │ ctrl+d           decrement 10       │
      ╰─────────────────────────────────────╯
        "#);
        enable_raw_mode()?;
        Sliders::read_key()?;
        Sliders::clear()?;
        Ok(())
    }

    pub fn prompt(&mut self) -> Result<bool, Box<dyn Error>> {
        match Sliders::read_key()? {
            (KeyCode::Char('h'), _) | (KeyCode::Left, _) => if self.current > 0 { self.current -= 1 },
            (KeyCode::Char('l'), _) | (KeyCode::Right, _) => if self.current < (self.sliders.len() - 1) { self.current += 1 },
            (KeyCode::Char('k'), _) | (KeyCode::Up, _) => self.sliders[self.current].inc(1)?,
            (KeyCode::Char('j'), _) | (KeyCode::Down , _)=> self.sliders[self.current].dec(1)?,
            (KeyCode::Char('g'), _) => self.sliders[self.current].set(0)?,
            (KeyCode::Char('G'), _) => self.sliders[self.current].set(100)?,
            (KeyCode::Char('m'), _) => self.sliders[self.current].set(50)?,
            (KeyCode::Char('?'), _) => Sliders::print_help()?,
            (KeyCode::Char('q'), _) => return Ok(false),
            (KeyCode::Char('u'), x) if x.contains(KeyModifiers::CONTROL) => self.sliders[self.current].inc(10)?,
            (KeyCode::Char('d'), x) if x.contains(KeyModifiers::CONTROL) => self.sliders[self.current].dec(10)?,
            _ => {},
        };
        Ok(true)
    }


    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        stdout().execute(Hide)?;
        Sliders::clear()?;
        enable_raw_mode()?;
        loop {
            self.draw()?;
            if !self.prompt()? {
                break;
            }
        }
        disable_raw_mode()?;
        stdout().execute(Show)?;
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

        Ok(Sliders { sliders, coordinates_percent: (0, 0), size_percent: (100, 100), current: 0 })
    }

}
