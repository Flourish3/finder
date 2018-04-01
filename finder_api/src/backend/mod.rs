use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;
use std::sync::mpsc::RecvError;

use std::io;
use std::fs::{self};
use std::path::Path;
use std::str::FromStr;

pub use self::types::Backend;
pub use self::types::BackendData;
pub use self::types::BackendResponse;
pub use self::types::BackendCommand;

use regex::Regex;

mod types;

impl Backend {
    pub fn new(tx : Sender<BackendResponse>) -> Backend {
        let result : Vec<String> = Vec::new();
        let data =  BackendData {
            search_query : "".to_string(),
            result : result,
        };
        Backend {
            tx : tx,
            internal_tx : None,
            data : Arc::new(Mutex::new(data)),
        }
    }

    pub fn run(mut self) -> Sender<BackendCommand> {
        let (apptx, rx) : (Sender<BackendCommand>, Receiver<BackendCommand>) = channel();

        self.internal_tx = Some(apptx.clone());
        thread::spawn(move || loop{
            let cmd = rx.recv();
            if !self.recieve_commad(cmd) {
                break;
            }
        });

        apptx
    }

    pub fn recieve_commad(&mut self, cmd : Result<BackendCommand, RecvError>) -> bool {

        match cmd {
            Ok(BackendCommand::Search(query)) => {
                self.process_query(query);
            }
            Ok(BackendCommand::Reset) => {
                println!( "Backend recieved reset");
            }
            Err(_) => {
                return false;
            }
        }
        true
    }

    pub fn process_query( &mut self, _query : String ) {
        //Clear result list
        self.data.lock().unwrap().result.clear();
        
        //Selects current path and start search
        let path = Path::new("./");
        self.visit_dirs(path).unwrap();

        //Send result list to application
        let tx = self.tx.clone();
        tx.send(BackendResponse::SearchResult(self.data.lock().unwrap().result.clone())).unwrap();
    }

    fn visit_dirs(&self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                self.filter_file(&path);
                if path.is_dir() {
                    self.visit_dirs(&path)?;
                } 
            }
        }
        Ok(())
    }

    fn filter_file(&self, path : &Path) {
        let query : String = "api".to_string();
        lazy_static! {
            static ref  RE: Regex = Regex::new(r"^.+\.(?i)(rs|toml)$").unwrap();
        }
        //let file_name = String::from_str(path.file_stem().unwrap().to_str().unwrap());
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if RE.is_match(file_name) {
            println!( "Regex match" );
            let fname = String::from_str(file_name).unwrap();
            self.data.lock().unwrap().result.push(format!("{}", fname));
        }

    }
}
