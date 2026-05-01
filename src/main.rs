use enigo::{
    Direction::{Click, Press},
    Enigo, Key, Keyboard, Settings,
};
use gilrs::{Button, Event, EventType, Gilrs};
use std::thread::{sleep, spawn};
use std::{env, fs};

// okay we need to take in a pid as an argument
// that's the one to kill/watch and then we need to grab gamepad buttons just like the python
// script
// might also want to do the rofi launch and translation here

fn main() {
    spawn(|| {
        rofi_menu();
    });
    println!("Continuing on...");
    let mut gilrs = Gilrs::new().unwrap();
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;
    let mut down_pressed = false;
    let mut up_pressed = false;
    let mut enter_pressed = false;

    enigo.key(Key::Unicode('a'), Click).unwrap();
    loop {
        // Examine new events
        while let Some(Event {
            id, event, time, ..
        }) = gilrs.next_event()
        {
            println!("{:?} New event from {}: {:?}", time, id, event);
            active_gamepad = Some(id);

            match event {
                EventType::ButtonPressed(Button::DPadDown, _) => {
                    if !down_pressed {
                        enigo.key(Key::DownArrow, Click).unwrap();
                        down_pressed = true;
                    }
                }
                EventType::ButtonReleased(Button::DPadDown, _) => {
                    down_pressed = false;
                }
                EventType::ButtonPressed(Button::DPadUp, _) => {
                    if !up_pressed {
                        enigo.key(Key::UpArrow, Click).unwrap();
                        up_pressed = true;
                    }
                }
                EventType::ButtonReleased(Button::DPadUp, _) => {
                    up_pressed = false;
                }
                EventType::ButtonPressed(Button::South, _) => {
                    if !enter_pressed {
                        enigo.key(Key::Return, Click).unwrap();
                        enter_pressed = true;
                    }
                }
                _ => {}
            }
        }
        /* if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::DPadDown) && !down_pressed {
                enigo.key(Key::DownArrow, Click).unwrap();
                down_pressed = true;
            } else {
                down_pressed = false;
            }
            if gamepad.is_pressed(Button::DPadUp) && !up_pressed {
                enigo.key(Key::UpArrow, Click).unwrap();
            } else {
                up_pressed = false;
            }
        } */
        sleep(std::time::Duration::from_millis(10));
    }
}

fn rofi_menu() {
    let dir_entries = fs::read_dir(env::current_dir().unwrap())
        .unwrap()
        .map(|d| format!("{:?}", d.unwrap().path()))
        .collect::<Vec<String>>();

    match rofi::Rofi::new(&dir_entries).run() {
        Ok(choice) => println!("Choice: {}", choice),
        Err(rofi::Error::Interrupted) => println!("Interrupted"),
        Err(e) => println!("Error: {}", e),
    }
}
