extern crate gtk;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::sync::mpsc::{ Sender, Receiver };
use std::sync::mpsc::RecvError;
use std::thread;

use std::str::FromStr;

use gio::ApplicationExt;
use gio::SimpleActionExt;
use gio::ActionMapExt;
use glib;
use gio;
use self::gio::prelude::*;
use self::gtk::prelude::*;

use backend::Backend;
use backend::BackendCommand;
use backend::BackendResponse;
use backend;

const APP_ID : &'static str = "org.Finder";

static mut OP : Option<Arc<Mutex<AppOp>>> = None;

macro_rules! APPOP {
    ( $fn: ident, ($($x:ident),*) ) => {{
        if let Some(ctx) = glib::MainContext::default() {
            ctx.invoke(move || {
                $( let $x = $x.clone(); )*
                if let Some(op) = AppOp::def() {
                    op.lock().unwrap().$fn($($x),*);
                }
            });
        }
    }};
    ($fn: ident) => {{
        APPOP!($fn, ( ) );
    }}
}

#[derive(Debug, Clone)]
pub enum AppState {
    Initializing,
    Loading,
    Running,
}

pub struct AppOp {
    pub gtk_builder : gtk::Builder,
    pub gtk_app : gtk::Application,
    pub state : AppState,
    pub backend : Sender<backend::BackendCommand>,
}


impl AppOp{
    pub fn def() -> Option<Arc<Mutex<AppOp>>> {
        unsafe {
            match OP {
                Some(ref m) => Some(m.clone()),
                None => None,
            }
        }
    }
    pub fn new(app : gtk::Application,
               builder : gtk::Builder,
               tx : Sender<backend::BackendCommand>) -> AppOp {
        AppOp {
            gtk_builder : builder,
            gtk_app : app,
            state : AppState::Initializing,
            backend : tx,
        }
    }

    pub fn init(&mut self) {
        self.set_state(AppState::Loading);
        /* Init stuff */
        self.set_state(AppState::Running);
    }

    pub fn activate(&self) {
        let window : gtk::Window = self.gtk_builder
            .get_object("main_window")
            .expect("Couldn't find main window(AppOp activate)");
        window.show();
        window.present();
    }

    pub fn quit(&self) {
        println!( "Closing application" );
        self.gtk_app.quit();
    }

    pub fn set_state( &mut self, new_state : AppState ) {
        self.state = new_state;
    }

    pub fn search_changed( &self, text : Option<String> ) {
        self.backend.send(BackendCommand::Search(text.unwrap())).unwrap();
    }

    pub fn process_result(&self, list : Vec<String>) {
        //Remove old labels
        let lb = self.gtk_builder
            .get_object::<gtk::ListBox>("result_list")
            .expect("Couldn't find result_list in UI");
         
        for label in lb.get_children().iter() {
            lb.remove(label);
        }

        for item in list.iter() {
            let lb = self.gtk_builder
                .get_object::<gtk::ListBox>("result_list")
                .expect("Couldn't find result_list in UI");
            
           

            let mes = gtk::Box::new( gtk::Orientation::Horizontal, 5 );
            mes.set_margin_top(2);
            mes.set_margin_bottom(2);
            //Content
            let content = gtk::Box::new( gtk::Orientation::Horizontal, 0);
            let name = String::from_str(item);

            let msg = gtk::Label::new( "" );
            msg.set_markup(&format!("<b>{}</b>",
                name.unwrap()));
            content.add(&msg);
            mes.pack_start(&content, true, true, 55);
            mes.show_all();
            lb.add(&mes);
        }
    }

    pub fn print(&self) {
        println!( "Activated listbox item" );
    }
}

pub struct App {
    gtk_builder : gtk::Builder,
    op : Arc<Mutex<AppOp>>,
}

impl App {
    pub fn new() {
            let gtk_app = gtk::Application::new(Some(APP_ID), gio::ApplicationFlags::empty())
                .expect("Failed to initiate GtkApplication");
            
            gtk_app.connect_startup(move |gtk_app| {
                let gtk_builder = gtk::Builder::new_from_resource("/org/Finder/main_window.glade");
                let window : gtk::Window = gtk_builder
                    .get_object("main_window")
                    .expect("Couldn't find main window in .ui");
                
                window.set_application(gtk_app);
                
                //Tx will be given to Backend, rx will be used in loop/thread for App
                let ( tx, rx ) : ( Sender<BackendResponse>, Receiver<BackendResponse> ) = channel();

                //Backend will return a Sender used by app
                let bk = Backend::new(tx);
                let apptx = bk.run();

                backend_loop(rx);
                
                let op = Arc::new(Mutex::new(
                    AppOp::new(gtk_app.clone(), gtk_builder.clone(), apptx)
                ));

                unsafe {
                    OP = Some(op.clone());
                }

                let app = App {
                    gtk_builder : gtk_builder,
                    op : op.clone(),
                };

                gtk_app.connect_activate(move |_| {op.lock().unwrap().activate() });

                app.connect_gtk();
                app.run();

            });

            gtk_app.run(&[]);
    }

    pub fn connect_gtk(&self) {
        let window : gtk::Window = self.gtk_builder
            .get_object("main_window")
            .expect("Couldn't find main window in ui (connect-gtk)");

        window.set_title("Finder");
        window.show_all();

        let op = self.op.clone();
        window.connect_delete_event(move |_,_| {
            op.lock().unwrap().quit();
            Inhibit(false)
        });

        self.connect_text_field();
    }

    pub fn connect_text_field(&self) {
        let op = &self.op;
        let e = self.gtk_builder
            .get_object::<gtk::SearchEntry>("search_entry")
            .expect("Couldn't find search_entry in UI");
        e.connect_search_changed(clone!(op => move |entry| {
            op.lock().unwrap().search_changed(entry.get_text());
        }));
    }

    pub fn run(&self) {
        self.op.lock().unwrap().init();
    }
}

fn backend_loop( rx : Receiver<BackendResponse> ) {
    thread::spawn(move || {
        loop {
            let recv = rx.recv();

            if let Err(RecvError) = recv {
                break;
            }

            match recv {
                Err( RecvError ) => { break; }
                Ok( BackendResponse::SearchResult(list) ) => {
                    //println!( "Result from backend!" );
                    APPOP!( process_result, (list) );
                }
            };
        }
    });
}