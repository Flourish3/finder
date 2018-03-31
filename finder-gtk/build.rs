use std::process::Command;

fn main() {
    //Compile the gresource
    Command::new("glib-compile-resources")
        .args(&["--generate", "resources.xml"])
        .current_dir("res")
        .status()
        .unwrap();
}