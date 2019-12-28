use std::fs;

fn main() {
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(filename) = entry.path().file_name() {
                    println!("{}", filename.to_string_lossy());
                }
            }
        }
    }
}
