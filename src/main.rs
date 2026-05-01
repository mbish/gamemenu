use enigo::{Direction::Click, Enigo, Key, Keyboard, Settings};
use gilrs::{Button, Event, EventType, Gilrs};
use std::env;
use std::{
    process::Command,
    thread::{sleep, spawn},
};

struct CommandBuilder {
    command: String,
    args: Vec<String>,
}

impl CommandBuilder {
    pub fn new(command: &str) -> Self {
        CommandBuilder {
            command: command.to_string(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn build(&self) -> Command {
        let mut cmd = Command::new(self.command.clone());
        for arg in self.args.iter() {
            cmd.arg(arg);
        }
        cmd
    }
}

struct RofiChoice {
    label: String,
    command: Option<CommandBuilder>,
}

impl RofiChoice {
    pub fn run(self) {
        if let Some(command) = self.command {
            let mut process = command.build().spawn().expect("Failed to execute command");
            process.wait().expect("Failed to wait on process");
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let target = match args.get(1) {
        Some(pid) => pid.parse::<u32>().expect("PID must be a number"),
        None => {
            eprintln!("Usage: {} <pid>", args[0]);
            std::process::exit(1);
        }
    };
    let mut gilrs = Gilrs::new().unwrap();
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    loop {
        'outer: loop {
            while let Some(Event { event, .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(Button::Mode, _) => {
                        spawn(move || {
                            rofi_menu(target);
                        });
                        break 'outer;
                    }
                    _ => {}
                }
            }
        }
        'outer: loop {
            while let Some(Event { event, .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(Button::DPadDown, _) => {
                        enigo.key(Key::DownArrow, Click).unwrap();
                    }
                    EventType::ButtonPressed(Button::DPadUp, _) => {
                        enigo.key(Key::UpArrow, Click).unwrap();
                    }
                    EventType::ButtonPressed(Button::South, _) => {
                        enigo.key(Key::Return, Click).unwrap();
                        break 'outer;
                    }
                    _ => {}
                }
            }
            sleep(std::time::Duration::from_millis(10));
        }
    }
}

fn rofi_menu(target_pid: u32) {
    let choices = [
        RofiChoice {
            label: "Resume".to_string(),
            command: None,
        },
        RofiChoice {
            label: "Close".to_string(),
            command: Some(
                CommandBuilder::new("kill")
                    .arg("-HUP")
                    .arg(&target_pid.to_string()),
            ),
        },
        RofiChoice {
            label: "Force Quit".to_string(),
            command: Some(
                CommandBuilder::new("kill")
                    .arg("-KILL")
                    .arg(&target_pid.to_string()),
            ),
        },
    ];
    let labels = choices.iter().map(|c| c.label.clone()).collect::<Vec<_>>();

    match rofi::Rofi::new(&labels).run_index() {
        Ok(choice) => {
            if let Some(choice) = choices.into_iter().nth(choice) {
                choice.run();
            }
        }
        Err(rofi::Error::Interrupted) => println!("Interrupted"),
        Err(e) => println!("Error: {}", e),
    }
}
