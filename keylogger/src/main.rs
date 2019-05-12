extern crate input;
extern crate libc;
extern crate nix;
extern crate std;
extern crate udev;

use input::event::device::DeviceEvent::Added;
use input::event::EventTrait;
use input::Event::{Keyboard, Device};
use input::event::keyboard::KeyboardEvent::Key;
use input::event::keyboard::KeyboardEventTrait;
use input::Libinput;
use nix::unistd::close;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::IntoRawFd;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;
use std::process::exit;
use udev::Context;
use std::os::unix::io::FromRawFd;
use std::fs::File;
use nix::poll;

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

fn main() {
    let input_interface = InputInterface {};
    let udev_context = Context::new().unwrap();
    let mut libinput_context = Libinput::new_from_udev(input_interface, &udev_context);
    libinput_context.udev_assign_seat("seat0").unwrap();
    libinput_context.dispatch().unwrap();

    let fd = libinput_context.as_raw_fd();
    let pollfd = poll::PollFd::new(fd, poll::POLLIN);

    while poll::poll(&mut [pollfd], -1).is_ok() {
        libinput_context.dispatch().unwrap();
        let mut events = libinput_context.clone().filter_map(|event| {
            match event {
                Keyboard(Key(key)) => Some(key),
                _ => None,
            }
        });
        for event in events {
            println!("{:?}", event.key());
            println!("{:?}", event.key_state());
        }
    }
}
