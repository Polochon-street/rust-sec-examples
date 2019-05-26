extern crate xkbcommon;

use nix::unistd::close;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{IntoRawFd, RawFd};
use std::path::Path;
use xkbcommon::xkb;
use xkbcommon::xkb::keysyms;

pub const MODIFIERS_KEYS: [u32; 6] = [
    keysyms::KEY_Control_L,
    keysyms::KEY_Control_R,
    keysyms::KEY_Alt_L,
    keysyms::KEY_Alt_R,
    keysyms::KEY_Super_L,
    keysyms::KEY_Super_R,
];

pub const NON_PRINT_KEYS: [u32; 31] = [
    keysyms::KEY_Delete,
    keysyms::KEY_DeleteChar,
    keysyms::KEY_DeleteLine,
    keysyms::KEY_BackSpace,
    keysyms::KEY_Tab,
    keysyms::KEY_Tab,
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

pub struct ModifiersState {
    pub ctrl: bool,
    pub alt: bool,
    pub windows: bool,
}

impl ModifiersState {
    pub fn new() -> ModifiersState {
        ModifiersState {
            ctrl: false,
            alt: false,
            windows: false,
        }
    }

    pub fn update_with(&mut self, state: &xkb::State) -> bool {
        let mut modified = false;
        let ctrl = state.mod_name_is_active(&xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE);
        let alt = state.mod_name_is_active(&xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE);
        let windows = state.mod_name_is_active(&xkb::MOD_NAME_LOGO, xkb::STATE_MODS_EFFECTIVE);
        if ctrl != self.ctrl || alt != self.alt || windows != self.windows {
            modified = true;
        }

        self.ctrl = ctrl;
        self.alt = alt;
        self.windows = windows;
        return modified;
    }

    pub fn get_modifiers_string(&mut self) -> String {
        let mut modifier_str = String::new();
        if self.ctrl {
            modifier_str.push_str("CTRL + ");
        }
        if self.alt {
            modifier_str.push_str("ALT + ");
        }
        if self.windows {
            modifier_str.push_str("LOGO + ");
        }
        return modifier_str;
    }
}

pub struct InputInterface;

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

pub fn get_kbd_variant() -> (String, String) {
    let vconsole_file = File::open("/etc/vconsole.conf");
    let mut contents = String::new();

    let mut handle = match vconsole_file {
        Ok(f) => f,
        Err(_) => return (String::from(""), String::from("")),
    };
    match handle.read_to_string(&mut contents) {
        Err(_) => return (String::from(""), String::from("")),
        _ => (),
    }
    let mut iter_split = contents.split("KEYMAP=").last().unwrap().split("-");
    let layout = iter_split.next().unwrap_or("");
    let variant = iter_split.next().unwrap_or("");

    (layout.to_string(), variant.to_string())
}
