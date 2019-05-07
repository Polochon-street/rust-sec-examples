extern crate udev;
extern crate libc;
extern crate input;
extern crate std;
extern crate nix;

use udev::Context;
use input::{Libinput, AsRaw, FromRaw};
use input::Device as DeviceStruct;
use input::Event::Device;
use std::os::unix::io::{AsRawFd, RawFd};
use input::event::device::DeviceEvent::Added;
use std::path::Path;
use std::fs::OpenOptions;
use std::os::unix::io::{IntoRawFd};
use std::os::unix::fs::OpenOptionsExt;
use nix::unistd::close;
use input::ffi::libinput_event_get_device;
use input::ffi::libinput_event;
use input::event::EventTrait;
use input::DeviceCapability::Keyboard;

struct InputInterface;

impl input::LibinputInterface for InputInterface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        let mut options = OpenOptions::new();
        options.read(true).write(true).custom_flags(flags);
        match options.open(path) {
            Ok(f) => {
                Ok(f.into_raw_fd())
            }
            Err(e) => {
                Err(e.raw_os_error().unwrap())
            }
        }
    }

    fn close_restricted(&mut self, fd: RawFd) {
        close(fd).unwrap()
    }
}

fn main() {
    let input_interface = InputInterface { };
    let udev_context = Context::new().unwrap();
    let mut libinput_context = Libinput::new_from_udev(input_interface, &udev_context);
    libinput_context.udev_assign_seat("seat0").unwrap();
    libinput_context.dispatch().unwrap();
    
    for context in libinput_context {
        let device = match context {
            Device(Added(a)) => a.device(),
            _ => panic!("coucou"),
        };
        if device.name().to_lowercase().contains("keyboard") && device.has_capability(Keyboard) {
            println!("Device found!");
            println!("{}", device.name());
            println!("{}", device.sysname());
        }
    }
}
