use enigo::{Direction::Click, Enigo, Key, Keyboard, Settings};
use gilrs::{Button, Event, EventType, Gilrs};
use log::{debug, error, info, warn};
use procfs::process::Process;
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
            match command.build().spawn() {
                Ok(mut process) => match process.wait() {
                    Ok(status) => {
                        if !status.success() {
                            eprintln!("Command exited with status: {}", status);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to wait on process: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to execute command: {}", e);
                }
            };
        }
    }
}

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let target = {
        let pid = match args.get(1) {
            Some(pid) => pid.parse::<i32>().expect("PID must be a number"),
            None => {
                error!("Usage: {} <pid>", args[0]);
                std::process::exit(1);
            }
        };
        Process::new(pid).expect("Failed to find process with given PID")
    };
    let mut gilrs = Gilrs::new().unwrap();
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        info!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad;
    loop {
        'outer: loop {
            if !target.is_alive() {
                info!("Target process has exited, exiting menu");
                std::process::exit(0);
            }
            while let Some(Event { event, id, .. }) = gilrs.next_event() {
                active_gamepad = Some(id);
                debug!("New event from {}: {:?}", id, event);
                if let EventType::ButtonPressed(Button::Mode, _) = event {
                    break 'outer;
                }
                if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id))
                    && gamepad.is_pressed(Button::Select)
                    && gamepad.is_pressed(Button::LeftTrigger)
                    && gamepad.is_pressed(Button::South)
                {
                    break 'outer;
                }
            }
            sleep(std::time::Duration::from_millis(100));
        }
        let pid = target.pid();
        let mut pause_target = Command::new("kill").arg("-USR1").arg(target.pid().to_string()).spawn().expect("Failed to stop target process");
        pause_target.wait().expect("Failed to wait on pause command");
        spawn(move || {
            debug!("Opening Rofi menu for PID: {:?}", pid);
            rofi_menu(pid);
        });
        'outer: loop {
            if !target.is_alive() {
                info!("Target process has exited, exiting menu");
                std::process::exit(0);
            }
            while let Some(Event { event, .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(Button::DPadDown, _) => {
                        debug!("DPadDown pressed, proxying to keyboard");
                        enigo.key(Key::DownArrow, Click).unwrap();
                    }
                    EventType::ButtonPressed(Button::DPadUp, _) => {
                        debug!("DPadUp pressed, proxying to keyboard");
                        enigo.key(Key::UpArrow, Click).unwrap();
                    }
                    EventType::ButtonPressed(Button::South, _) => {
                        debug!("South, proxying to keyboard");
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

fn rofi_menu(target_pid: i32) {
    let choices = [
        RofiChoice {
            label: "Resume".to_string(),
            command: Some(
                CommandBuilder::new("kill")
                    .arg("-USR2")
                    .arg(&target_pid.to_string()),
            ),
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
                debug!("User selected: {}", choice.label);
                choice.run();
            }
        }
        Err(rofi::Error::Interrupted) => warn!("Interrupted"),
        Err(e) => error!("Error: {}", e),
    }
}
