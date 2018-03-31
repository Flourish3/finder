#![deny(unused_extern_crates)]
extern crate glib;
extern crate gio;

#[macro_use]
extern crate finder_api;
use finder_api::backend;

mod static_resources;
mod app;

use app::App;

fn main() {
    print!("Loading static resources...");
    static_resources::init().expect("Gresource initialization failed");
    println!("Done!");
    println!("Launching application...");
    App::new();
}
