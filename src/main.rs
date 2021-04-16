use std::env;
use std::error::Error;
use std::process::Command;
use crossterm::{execute,cursor::MoveTo,event::{read, Event, KeyCode},terminal::{size, Clear, ClearType, enable_raw_mode, disable_raw_mode}, ExecutableCommand};
use std::io::{stdout, Write};

struct Slider {
    name: String,
    get_command: String,
    set_command: String,
    current: u8,
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

impl Slider {

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
        self.current = value;
        Command::new("sh")
            .arg("-c")
            .arg(self.set_command.replace("%", format!("{}", value).as_str()))
            .output()?;
        Ok(())
    }

    fn inc(&mut self) -> Result<(), Box<dyn Error>> {
        let val = self.get()?;
        if val < 100 { 
            self.set(val + 1)?;
            self.current = val + 1;
        }
        Ok(())
    }

    fn dec(&mut self) -> Result<(), Box<dyn Error>> {
        let val = self.get()?;
        if val > 0 { 
            self.set(val - 1)?; 
            self.current = val - 1;
        }
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        self.current = self.get()?;
        Ok(())
    }
}

fn parse_args() -> Result<Vec<Slider>, Box<dyn Error>> {
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
        sliders.push(Slider {
            name: names[i].clone(),
            get_command: get_commands[i].clone(),
            set_command: set_commands[i].clone(),
            current: 25,
        });
    }

    for slider in &mut sliders {
        slider.initialize()?;
    }

    Ok(sliders)
}

fn clear() -> Result<(), Box<dyn Error>> {
    stdout().execute(Clear(ClearType::All))?;
    Ok(())
}

fn draw(sliders: &Vec<Slider>, current: &mut usize) -> Result<(), Box<dyn Error>> {
    let (cols, rows) = size()?; 
    clear()?;
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

fn read_key() -> Result<KeyCode, Box<dyn Error>> {
    loop {
        if let Event::Key(e) = read()?
        {
            return Ok(e.code);
        }
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut sliders = parse_args()?;
    let mut current = 0;
    enable_raw_mode()?;
    loop {
        draw(&sliders, &mut current)?;
        let slider = &sliders[current];
        match read_key()? {
            KeyCode::Left => if current > 0 { current -= 1 },
            KeyCode::Right => if current < (sliders.len() - 1) { current += 1 },
            KeyCode::Up => if slider.current < 100 { sliders[current].inc()? },
            KeyCode::Down => if slider.current > 0 { sliders[current].dec()? },
            KeyCode::Char('q') => break,
            _ => {},
        }
    }
    disable_raw_mode()?;
    clear()?;
    Ok(())
}
